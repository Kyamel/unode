use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::Duration;

use anyhow::{Context, Result};
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use serde_json::{json, Value as JsonValue};
use unode_runtime::{
    CommandResult, DeferredText, RegisteredCommand, RegisteredNavigationItem, RegisteredRoute,
    ShellContext,
};
use unode_sdk::{
    PluginDispatchOutcome, PluginDispatchRequest, PluginDispatchResponse, PluginLoadRequest,
    PluginRenderRequest,
};
use unode_sdk::prelude::{ActionRef, ActionType, CoreActionType, PermissionProfile, ResolvedRoute, ScreenNode};
use unode_tui_runtime::{CachedTuiPlugin, PluginSession, TuiHostCallDispatcher, TuiRuntime};
use tui_renderer::{
    collect_screen_interactions, render_tui_shell, TuiCommandBar, TuiFocusedPane,
    TuiInteractiveElement, TuiInteractiveKind, TuiMainContent, TuiMainPanel, TuiNavItem,
    TuiScreenView, TuiShellView,
};

fn main() -> Result<()> {
    let mut app = App::new()?;
    app.run()
}

struct LoadedPlugin {
    runtime_plugin: CachedTuiPlugin,
    route: String,
    display_source: String,
    source_newer_than_wasm: bool,
}

struct ActivePluginSession {
    plugin_id: String,
    route: String,
    session: PluginSession,
}

struct App {
    runtime: TuiRuntime<()>,
    shell: ShellContext,
    current_route: String,
    selected_nav: usize,
    command_mode: bool,
    command_input: String,
    status: String,
    main_panel: TuiMainContent,
    focused_pane: TuiFocusedPane,
    main_interactions: Vec<TuiInteractiveElement>,
    selected_main_interaction: Option<usize>,
    plugins: Vec<LoadedPlugin>,
    active_plugin_session: Option<ActivePluginSession>,
}

impl App {
    fn new() -> Result<Self> {
        let profile = PermissionProfile {
            plugin_id: "mgn.shell".to_string(),
            grants: vec![],
        };
        let mut runtime = TuiRuntime::new(profile);

        register_builtin_shell(&mut runtime);
        let (plugins, plugin_messages) = load_runtime_plugins(&mut runtime)?;

        let current_route = "/home".to_string();
        let shell = ShellContext {
            route: runtime.inner.routes.resolve(&current_route),
            locale: Some("en".to_string()),
            plugin_id: None,
            screen_kind: None,
        };

        let mut app = Self {
            runtime,
            shell,
            current_route,
            selected_nav: 0,
            command_mode: false,
            command_input: String::new(),
            status: if plugin_messages.is_empty() {
                "TUI runtime ready".to_string()
            } else {
                plugin_messages.join(" | ")
            },
            focused_pane: TuiFocusedPane::Navigation,
            main_interactions: vec![],
            selected_main_interaction: None,
            main_panel: TuiMainContent::Panel(TuiMainPanel {
                title: "Loading".to_string(),
                subtitle: None,
                lines: vec![],
                footer: None,
            }),
            plugins,
            active_plugin_session: None,
        };
        app.refresh_main_panel();
        Ok(app)
    }

    fn run(&mut self) -> Result<()> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        let result = self.event_loop(&mut terminal);

        disable_raw_mode()?;
        execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
        terminal.show_cursor()?;

        result
    }

    fn event_loop(&mut self, terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
        loop {
            terminal.draw(|frame| {
                let view = self.view();
                render_tui_shell(frame, &view);
            })?;

            if !event::poll(Duration::from_millis(150))? {
                continue;
            }

            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }

                if self.command_mode {
                    if self.handle_command_key(key.code)? {
                        break;
                    }
                } else if self.handle_shell_key(key.code, key.modifiers)? {
                    break;
                }
            }
        }

        Ok(())
    }

    fn handle_shell_key(&mut self, code: KeyCode, modifiers: KeyModifiers) -> Result<bool> {
        match code {
            KeyCode::Char('q') => return Ok(true),
            KeyCode::Tab => {
                if modifiers.contains(KeyModifiers::SHIFT) {
                    self.focus_previous_pane();
                } else {
                    self.focus_next_pane();
                }
            }
            KeyCode::BackTab => self.focus_previous_pane(),
            KeyCode::Up | KeyCode::Left => match self.focused_pane {
                TuiFocusedPane::Navigation => {
                    self.selected_nav = self.selected_nav.saturating_sub(1);
                }
                TuiFocusedPane::Main => self.move_main_focus(-1),
            },
            KeyCode::Down | KeyCode::Right => match self.focused_pane {
                TuiFocusedPane::Navigation => {
                    let nav_len = self.nav_items().len();
                    if nav_len > 0 {
                        self.selected_nav = (self.selected_nav + 1).min(nav_len - 1);
                    }
                }
                TuiFocusedPane::Main => self.move_main_focus(1),
            },
            KeyCode::Enter => match self.focused_pane {
                TuiFocusedPane::Navigation => {
                    if let Some(item) = self.nav_items().get(self.selected_nav) {
                        self.navigate_to(item.route.clone());
                    }
                }
                TuiFocusedPane::Main => self.activate_main_interaction()?,
            },
            KeyCode::Char(':') => {
                self.command_mode = true;
                self.command_input.clear();
                self.status = "Command mode".to_string();
            }
            _ => {}
        }

        Ok(false)
    }

    fn handle_command_key(&mut self, code: KeyCode) -> Result<bool> {
        match code {
            KeyCode::Esc => {
                self.command_mode = false;
                self.command_input.clear();
                self.status = "Command cancelled".to_string();
            }
            KeyCode::Enter => {
                let command_id = self.command_input.trim().to_string();
                self.command_mode = false;
                self.command_input.clear();
                self.execute_command(command_id)?;
            }
            KeyCode::Backspace => {
                self.command_input.pop();
            }
            KeyCode::Char('q') if self.command_input.is_empty() => return Ok(true),
            KeyCode::Char(ch) => {
                self.command_input.push(ch);
            }
            _ => {}
        }

        Ok(false)
    }

    fn execute_command(&mut self, id: String) -> Result<()> {
        if id.is_empty() {
            self.status = "Empty command".to_string();
            return Ok(());
        }

        match self.runtime.inner.commands.run(&id, &self.shell, &()) {
            Ok(CommandResult::Navigate(route)) => {
                self.navigate_to(route);
                self.status = format!("Executed command `{id}`");
            }
            Ok(CommandResult::RefreshCurrentScreen) => {
                self.refresh_main_panel();
                self.status = format!("Refresh requested by `{id}`");
            }
            Ok(CommandResult::Invalidate(keys)) => {
                self.refresh_main_panel();
                self.status = format!("Invalidation requested: {}", keys.join(", "));
            }
            Ok(CommandResult::None) => {
                self.status = format!("Executed command `{id}`");
            }
            Err(err) => {
                self.status = format!("Command error: {err}");
            }
        }

        Ok(())
    }

    fn navigate_to(&mut self, route: String) {
        if self.current_route != route {
            self.active_plugin_session = None;
        }
        self.current_route = route.clone();
        let parsed = parse_route(&route);
        self.shell.route = self.runtime.inner.routes.resolve(&parsed.pathname);
        self.shell.plugin_id = self.shell.route.as_ref().map(|route| route.plugin_id.clone());
        self.shell.screen_kind = self.shell.route.as_ref().map(|route| route.screen_kind.clone());
        self.status = format!("Navigated to {}", self.current_route);
        self.refresh_main_panel();
        if !self.main_interactions.is_empty() {
            self.focused_pane = TuiFocusedPane::Main;
        }
    }

    fn nav_items(&self) -> Vec<TuiNavItem> {
        self.runtime
            .inner
            .navigation
            .get_available(&self.shell)
            .into_iter()
            .map(|item| TuiNavItem {
                id: item.id,
                label: item.label,
                route: item.to,
            })
            .collect()
    }

    fn refresh_main_panel(&mut self) {
        self.main_panel = match self.render_plugin_panel() {
            Some(panel) => panel,
            None => TuiMainContent::Panel(self.render_shell_panel()),
        };
        self.main_interactions = match &self.main_panel {
            TuiMainContent::Screen(view) => collect_screen_interactions(&view.screen),
            TuiMainContent::Panel(_) => vec![],
        };
        self.selected_main_interaction = self.resolve_main_focus();
        if self.focused_pane == TuiFocusedPane::Main && self.main_interactions.is_empty() {
            self.focused_pane = TuiFocusedPane::Navigation;
        }
    }

    fn render_plugin_panel(&mut self) -> Option<TuiMainContent> {
        let route = self.current_route.clone();
        let locale = self.shell.locale.clone();
        let parsed = parse_route(&route);
        let plugin_index = self
            .plugins
            .iter()
            .position(|plugin| plugin.route == parsed.pathname)?;
        let plugin = &self.plugins[plugin_index];
        let manifest = plugin.runtime_plugin.manifest().clone();
        let display_source = plugin.display_source.clone();
        let source_newer_than_wasm = plugin.source_newer_than_wasm;
        let request = PluginRenderRequest {
            route: ResolvedRoute {
                pattern: parsed.pathname,
                params: self
                    .shell
                    .route
                    .as_ref()
                    .map(|resolved| resolved.params.clone())
                    .unwrap_or_default(),
                query: parsed.query,
            },
            data: json!({
                "title": "Smoke test",
                "hostMessage": format!("Loaded from {}", display_source),
            }),
            state_snapshot: BTreeMap::new(),
            locale,
        };

        let session = match self.ensure_plugin_session(
            plugin_index,
            &manifest.manifest.id,
            &route,
            &request.route,
            request.locale.clone(),
        ) {
            Ok(session) => session,
            Err(err) => {
                return Some(TuiMainContent::Panel(TuiMainPanel {
                    title: format!("Plugin error: {}", manifest.manifest.name),
                    subtitle: Some(manifest.manifest.id.clone()),
                    lines: vec![
                        "The plugin loaded, but the runtime could not start a plugin session.".to_string(),
                        err.to_string(),
                    ],
                    footer: Some("Check the wasm artifact and runtime session lifecycle.".to_string()),
                }))
            }
        };

        Some(match session.render::<ScreenNode>(&request) {
            Ok(screen) => TuiMainContent::Screen(TuiScreenView {
                plugin_id: manifest.manifest.id.clone(),
                source: display_source,
                screen,
                focused_interaction: None,
            }),
            Err(err) => TuiMainContent::Panel(TuiMainPanel {
                title: format!("Plugin error: {}", manifest.manifest.name),
                subtitle: Some(manifest.manifest.id.clone()),
                lines: {
                    let mut lines = vec![
                    "The plugin loaded, but `plugin_render` failed.".to_string(),
                    err.to_string(),
                    ];
                    if source_newer_than_wasm {
                        lines.push(String::new());
                        lines.push(
                            "The plugin source looks newer than the compiled `.wasm` artifact.".to_string(),
                        );
                        lines.push(
                            "Rebuild inside `nix-shell` so the runtime stops loading an outdated binary."
                                .to_string(),
                        );
                    }
                    lines
                },
                footer: Some(
                    "Check the plugin ABI exports, render payload, and rebuild the wasm artifact if needed."
                        .to_string(),
                ),
            }),
        })
    }

    fn ensure_plugin_session(
        &mut self,
        plugin_index: usize,
        plugin_id: &str,
        route: &str,
        resolved_route: &ResolvedRoute,
        locale: Option<String>,
    ) -> Result<&mut PluginSession> {
        let reuse_existing = self
            .active_plugin_session
            .as_ref()
            .map(|session| session.plugin_id == plugin_id && session.route == route)
            .unwrap_or(false);

        if !reuse_existing {
            let request = PluginLoadRequest {
                route: resolved_route.clone(),
                state_snapshot: BTreeMap::new(),
                locale,
            };
            let session = self.plugins[plugin_index]
                .runtime_plugin
                .start_session(&request)
                .with_context(|| format!("failed to activate `{plugin_id}` for route `{route}`"))?;
            self.active_plugin_session = Some(ActivePluginSession {
                plugin_id: plugin_id.to_string(),
                route: route.to_string(),
                session,
            });
        }

        Ok(&mut self
            .active_plugin_session
            .as_mut()
            .expect("plugin session should exist after activation")
            .session)
    }

    fn render_shell_panel(&self) -> TuiMainPanel {
        let route = self.current_route.clone();
        let plugin_id = self
            .shell
            .route
            .as_ref()
            .map(|route| route.plugin_id.clone())
            .unwrap_or_else(|| "shell".to_string());
        let screen_kind = self
            .shell
            .route
            .as_ref()
            .map(|route| route.screen_kind.clone())
            .unwrap_or_else(|| "shell.index".to_string());

        let available_commands = self
            .runtime
            .inner
            .commands
            .get_available(&self.shell)
            .into_iter()
            .map(|cmd| cmd.id)
            .collect::<Vec<_>>();

        TuiMainPanel {
            title: format!("Plugin shell for {}", route),
            subtitle: Some(format!("pluginId={} • screenKind={}", plugin_id, screen_kind)),
            lines: vec![
                "This shell is ready for runtime-loaded WASM plugins.".to_string(),
                "The sidebar is fed by the generic navigation registry.".to_string(),
                "The command line is fed by the generic command registry.".to_string(),
                "The main area is where plugin screens are mounted.".to_string(),
                String::new(),
                "Available commands:".to_string(),
                available_commands
                    .into_iter()
                    .map(|id| format!("  - {}", id))
                    .collect::<Vec<_>>()
                    .join("\n"),
            ],
            footer: Some(
                "Keys: Tab switches pane, arrows move focus, Enter activates, : command mode, q quit"
                    .to_string(),
            ),
        }
    }

    fn view(&self) -> TuiShellView {
        let main = match &self.main_panel {
            TuiMainContent::Screen(screen) => {
                let mut screen = screen.clone();
                screen.focused_interaction = if self.focused_pane == TuiFocusedPane::Main {
                    self.selected_main_interaction
                } else {
                    None
                };
                TuiMainContent::Screen(screen)
            }
            TuiMainContent::Panel(panel) => TuiMainContent::Panel(panel.clone()),
        };

        TuiShellView {
            title: "MGN Test Shell".to_string(),
            status: self.status.clone(),
            nav_items: self.nav_items(),
            selected_nav: self.selected_nav,
            focused_pane: self.focused_pane,
            main,
            command_bar: TuiCommandBar {
                prompt: ":".to_string(),
                input: self.command_input.clone(),
                active: self.command_mode,
                hint: Some("type a command id like `open.dev.mugens.sanity-check`".to_string()),
            },
        }
    }

    fn focus_next_pane(&mut self) {
        self.focused_pane = if self.focused_pane == TuiFocusedPane::Navigation && !self.main_interactions.is_empty() {
            TuiFocusedPane::Main
        } else {
            TuiFocusedPane::Navigation
        };
    }

    fn focus_previous_pane(&mut self) {
        self.focus_next_pane();
    }

    fn move_main_focus(&mut self, delta: isize) {
        if self.main_interactions.is_empty() {
            return;
        }

        let current = self.selected_main_interaction.unwrap_or(0) as isize;
        let len = self.main_interactions.len() as isize;
        let next = (current + delta).rem_euclid(len) as usize;
        self.selected_main_interaction = Some(next);
    }

    fn resolve_main_focus(&self) -> Option<usize> {
        let TuiMainContent::Screen(view) = &self.main_panel else {
            return None;
        };

        if self.main_interactions.is_empty() {
            return None;
        }

        if let Some(current) = self.selected_main_interaction {
            if current < self.main_interactions.len() {
                return Some(current);
            }
        }

        if let Some(initial_focus) = view.screen.initial_focus.as_deref() {
            if let Some(index) = self
                .main_interactions
                .iter()
                .position(|interaction| interaction.node_id.as_deref() == Some(initial_focus))
            {
                return Some(index);
            }
        }

        Some(0)
    }

    fn activate_main_interaction(&mut self) -> Result<()> {
        let Some(index) = self.selected_main_interaction else {
            self.status = "No interactive element selected".to_string();
            return Ok(());
        };
        let Some(interaction) = self.main_interactions.get(index).cloned() else {
            self.status = "Focused element is out of bounds".to_string();
            return Ok(());
        };

        match interaction.kind {
            TuiInteractiveKind::RouteTab { to } => {
                self.navigate_to(to);
                self.status = format!("Opened tab `{}`", interaction.label);
            }
            TuiInteractiveKind::Action { action } | TuiInteractiveKind::ListItem { action } => {
                self.dispatch_action(action, &interaction.label)?;
            }
        }

        Ok(())
    }

    fn dispatch_action(&mut self, action: ActionRef, label: &str) -> Result<()> {
        if self.handle_builtin_action(&action, label)? {
            return Ok(());
        }

        let Some(plugin_id) = self.shell.plugin_id.clone() else {
            self.status = format!("No plugin bound to action `{label}`");
            return Ok(());
        };

        let parsed = parse_route(&self.current_route);
        let request = PluginDispatchRequest {
            route: ResolvedRoute {
                pattern: parsed.pathname,
                params: self
                    .shell
                    .route
                    .as_ref()
                    .map(|resolved| resolved.params.clone())
                    .unwrap_or_default(),
                query: parsed.query,
            },
            action,
            state_snapshot: BTreeMap::new(),
            locale: self.shell.locale.clone(),
        };

        let plugin = self
            .plugins
            .iter()
            .position(|plugin| plugin.runtime_plugin.manifest().manifest.id == plugin_id)
            .with_context(|| format!("plugin `{plugin_id}` not loaded"))?;
        let session = self
            .ensure_plugin_session(plugin, &plugin_id, &self.current_route.clone(), &request.route, request.locale.clone())
            .with_context(|| format!("failed to start plugin session for `{plugin_id}`"))?;
        let response = session
            .dispatch::<PluginDispatchResponse>(&request)
            .with_context(|| format!("plugin_dispatch failed for `{plugin_id}`"))?;
        self.apply_dispatch_response(response, label);
        Ok(())
    }

    fn handle_builtin_action(&mut self, action: &ActionRef, label: &str) -> Result<bool> {
        let ActionType::Core(core) = &action.r#type else {
            return Ok(false);
        };

        match core {
            CoreActionType::Navigate => {
                if let Some(to) = action
                    .params
                    .as_ref()
                    .and_then(|params| params.get("to"))
                    .and_then(JsonValue::as_str)
                {
                    self.navigate_to(to.to_string());
                    self.status = format!("Navigated from `{label}`");
                    return Ok(true);
                }
                self.status = format!("Action `{label}` is missing `to`");
                Ok(true)
            }
            CoreActionType::Refresh => {
                self.refresh_main_panel();
                self.status = format!("Refreshed from `{label}`");
                Ok(true)
            }
            CoreActionType::Dismiss => {
                self.status = format!("Dismiss requested by `{label}`");
                Ok(true)
            }
            CoreActionType::LoadMore | CoreActionType::Submit => Ok(false),
        }
    }

    fn apply_dispatch_response(&mut self, response: PluginDispatchResponse, label: &str) {
        match response.outcome {
            PluginDispatchOutcome::None => {}
            PluginDispatchOutcome::RefreshCurrentScreen => self.refresh_main_panel(),
            PluginDispatchOutcome::Navigate { to } => self.navigate_to(to),
        }

        self.status = response.message.unwrap_or_else(|| {
            if response.handled {
                format!("Handled action `{label}`")
            } else {
                format!("Plugin ignored action `{label}`")
            }
        });
    }
}

#[derive(Debug, Clone)]
struct ParsedRoute {
    pathname: String,
    query: BTreeMap<String, String>,
}

fn parse_route(route: &str) -> ParsedRoute {
    let (pathname, raw_query) = route.split_once('?').map_or((route, ""), |(path, query)| (path, query));
    let query = raw_query
        .split('&')
        .filter(|pair| !pair.is_empty())
        .map(|pair| {
            let (key, value) = pair.split_once('=').map_or((pair, ""), |(key, value)| (key, value));
            (key.to_string(), value.to_string())
        })
        .collect::<BTreeMap<_, _>>();

    ParsedRoute {
        pathname: pathname.to_string(),
        query,
    }
}

fn register_builtin_shell(runtime: &mut TuiRuntime<()>) {
    runtime.inner.routes.register(RegisteredRoute {
        plugin_id: "org.mugens.core.home".to_string(),
        pattern: "/home".to_string(),
        screen_kind: "org.mugens.core.home".to_string(),
        priority: 100,
    });
    runtime.inner.routes.register(RegisteredRoute {
        plugin_id: "org.mugens.core.mangas.hot".to_string(),
        pattern: "/mangas/hot".to_string(),
        screen_kind: "org.mugens.core.mangas.hot".to_string(),
        priority: 100,
    });
    runtime.inner.routes.register(RegisteredRoute {
        plugin_id: "org.mugens.core.mangas.recent".to_string(),
        pattern: "/mangas/recent".to_string(),
        screen_kind: "org.mugens.core.mangas.recent".to_string(),
        priority: 100,
    });

    runtime.inner.navigation.register(RegisteredNavigationItem {
        id: "nav.home".to_string(),
        plugin_id: "org.mugens.core.home".to_string(),
        label: DeferredText::from("Home"),
        short_label: None,
        to: "/home".to_string(),
        icon: None,
        section: Some("main".to_string()),
        priority: 300,
        when: None,
    });
    runtime.inner.navigation.register(RegisteredNavigationItem {
        id: "nav.mangas.hot".to_string(),
        plugin_id: "org.mugens.core.mangas.hot".to_string(),
        label: DeferredText::from("Mangas Hot"),
        short_label: None,
        to: "/mangas/hot".to_string(),
        icon: None,
        section: Some("main".to_string()),
        priority: 200,
        when: None,
    });
    runtime.inner.navigation.register(RegisteredNavigationItem {
        id: "nav.mangas.recent".to_string(),
        plugin_id: "org.mugens.core.mangas.recent".to_string(),
        label: DeferredText::from("Mangas Recent"),
        short_label: None,
        to: "/mangas/recent".to_string(),
        icon: None,
        section: Some("main".to_string()),
        priority: 100,
        when: None,
    });

    runtime.inner.commands.register(RegisteredCommand {
        id: "goto.home".to_string(),
        plugin_id: "org.mugens.core.home".to_string(),
        title: DeferredText::from("Go to Home"),
        category: Some(DeferredText::from("Navigation")),
        keywords: vec!["home".to_string()],
        when: None,
        run: std::sync::Arc::new(|_| CommandResult::Navigate("/home".to_string())),
    });
    runtime.inner.commands.register(RegisteredCommand {
        id: "goto.mangas.hot".to_string(),
        plugin_id: "org.mugens.core.mangas.hot".to_string(),
        title: DeferredText::from("Go to Mangas Hot"),
        category: Some(DeferredText::from("Navigation")),
        keywords: vec!["mangas".to_string(), "hot".to_string()],
        when: None,
        run: std::sync::Arc::new(|_| CommandResult::Navigate("/mangas/hot".to_string())),
    });
    runtime.inner.commands.register(RegisteredCommand {
        id: "goto.mangas.recent".to_string(),
        plugin_id: "org.mugens.core.mangas.recent".to_string(),
        title: DeferredText::from("Go to Mangas Recent"),
        category: Some(DeferredText::from("Navigation")),
        keywords: vec!["mangas".to_string(), "recent".to_string()],
        when: None,
        run: std::sync::Arc::new(|_| CommandResult::Navigate("/mangas/recent".to_string())),
    });
}

fn load_runtime_plugins(runtime: &mut TuiRuntime<()>) -> Result<(Vec<LoadedPlugin>, Vec<String>)> {
    let mut messages = Vec::new();
    let mut plugins = Vec::new();

    let plugin_root = workspace_root().join("plugins/sanity-check");
    let Some(wasm_path) = find_plugin_wasm(&plugin_root) else {
        messages.push(
            "Sanity plugin not built yet. Run `cargo build --manifest-path plugins/sanity-check/Cargo.toml --target wasm32-unknown-unknown`.".to_string(),
        );
        return Ok((plugins, messages));
    };
    let source_newer_than_wasm = plugin_source_is_newer_than_wasm(&plugin_root, &wasm_path);

    let mut dispatcher = TuiHostCallDispatcher::new();
    dispatcher.register("system.ping", |_| Ok(json!({ "pong": true })));
    dispatcher.register("navigation.navigate", |params| {
        Ok(json!({
            "ok": true,
            "to": params.get("to").cloned().unwrap_or(JsonValue::Null)
        }))
    });

    let runtime_plugin = CachedTuiPlugin::from_wasm_file(&wasm_path, dispatcher)
        .with_context(|| format!("failed to instantiate plugin at {}", wasm_path.display()))?;
    let manifest = runtime_plugin.manifest().clone();

    let route = format!("/plugins/{}", plugin_slug(&manifest.manifest.id));

    runtime.inner.routes.register(RegisteredRoute {
        plugin_id: manifest.manifest.id.clone(),
        pattern: route.clone(),
        screen_kind: format!("{}.screen", manifest.manifest.id),
        priority: 500,
    });
    runtime.inner.navigation.register(RegisteredNavigationItem {
        id: format!("nav.{}", manifest.manifest.id),
        plugin_id: manifest.manifest.id.clone(),
        label: DeferredText::from(manifest.manifest.name.clone()),
        short_label: None,
        to: route.clone(),
        icon: None,
        section: Some("plugins".to_string()),
        priority: 400,
        when: None,
    });
    runtime.inner.commands.register(RegisteredCommand {
        id: format!("open.{}", manifest.manifest.id),
        plugin_id: manifest.manifest.id.clone(),
        title: DeferredText::from(format!("Open {}", manifest.manifest.name)),
        category: Some(DeferredText::from("Plugins")),
        keywords: vec!["plugin".to_string(), plugin_slug(&manifest.manifest.id)],
        when: None,
        run: {
            let route = route.clone();
            std::sync::Arc::new(move |_| CommandResult::Navigate(route.clone()))
        },
    });

    let display_source = display_path_for_ui(&wasm_path);
    messages.push(format!(
        "Loaded {} from {}",
        manifest.manifest.name,
        display_source
    ));
    if source_newer_than_wasm {
        messages.push(format!(
            "Warning: {} source changed after the wasm build. Rebuild the plugin inside `nix-shell`.",
            manifest.manifest.name
        ));
    }
    plugins.push(LoadedPlugin {
        runtime_plugin,
        route,
        display_source,
        source_newer_than_wasm,
    });

    Ok((plugins, messages))
}

fn find_plugin_wasm(plugin_root: &Path) -> Option<PathBuf> {
    let candidates = [
        plugin_root.join("target/wasm32-unknown-unknown/debug/sanity_check_plugin.wasm"),
        plugin_root.join("target/wasm32-unknown-unknown/release/sanity_check_plugin.wasm"),
    ];

    candidates
        .into_iter()
        .filter_map(|path| {
            let modified = fs::metadata(&path).ok()?.modified().ok()?;
            Some((modified, path))
        })
        .max_by_key(|(modified, _)| *modified)
        .map(|(_, path)| path)
}

fn plugin_source_is_newer_than_wasm(plugin_root: &Path, wasm_path: &Path) -> bool {
    let Ok(wasm_modified) = fs::metadata(wasm_path).and_then(|metadata| metadata.modified()) else {
        return false;
    };

    latest_modified_in(plugin_root.join("src"))
        .into_iter()
        .chain(
            [
                plugin_root.join("Cargo.toml"),
                plugin_root.join("Cargo.lock"),
            ]
            .into_iter()
            .filter_map(|path| fs::metadata(path).ok())
            .filter_map(|metadata| metadata.modified().ok()),
        )
        .any(|modified| modified > wasm_modified)
}

fn latest_modified_in(path: PathBuf) -> Vec<std::time::SystemTime> {
    let Ok(metadata) = fs::metadata(&path) else {
        return vec![];
    };

    if metadata.is_file() {
        return metadata.modified().ok().into_iter().collect();
    }

    let Ok(entries) = fs::read_dir(path) else {
        return vec![];
    };

    entries
        .filter_map(Result::ok)
        .flat_map(|entry| latest_modified_in(entry.path()))
        .collect()
}

fn plugin_slug(plugin_id: &str) -> String {
    plugin_id
        .split('.')
        .next_back()
        .unwrap_or(plugin_id)
        .to_string()
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn display_path_for_ui(path: &Path) -> String {
    path.strip_prefix(workspace_root())
        .unwrap_or(path)
        .display()
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::{find_plugin_wasm, plugin_slug, App, TuiFocusedPane};
    use std::collections::BTreeMap;
    use std::fs;
    use std::path::PathBuf;
    use std::thread::sleep;
    use std::time::Duration;

    use serde_json::{json, Value as JsonValue};
    use unode_sdk::{
        PluginDispatchRequest, PluginDispatchResponse, PluginLoadRequest, PluginRenderRequest,
    };
    use unode_sdk::prelude::{ActionRef, ActionType, PermissionProfile, ResolvedRoute, ScreenNode};
    use unode_tui_runtime::{TuiHostCallDispatcher, WasmtimeGuest};
    use tui_renderer::TuiMainContent;

    fn route_for(plugin_id: &str) -> String {
        format!("/plugins/{}", plugin_slug(plugin_id))
    }

    fn test_dispatcher() -> TuiHostCallDispatcher {
        let mut dispatcher = TuiHostCallDispatcher::new();
        dispatcher.register("system.ping", |_| Ok(json!({ "pong": true })));
        dispatcher.register("navigation.navigate", |params| {
            Ok(json!({
                "ok": true,
                "to": params.get("to").cloned().unwrap_or(JsonValue::Null)
            }))
        });
        dispatcher
    }

    #[test]
    fn sanity_plugin_survives_render_dispatch_render_sequence() {
        let plugin_root = PathBuf::from("plugins/sanity-check");
        let Some(wasm_path) = find_plugin_wasm(&plugin_root) else {
            return;
        };

        let mut bridge = WasmtimeGuest::from_wasm_file(&wasm_path, test_dispatcher()).expect("instantiate wasm");
        let manifest = bridge.call_plugin_manifest().expect("manifest");
        let route = route_for(&manifest.manifest.id);

        bridge
            .call_plugin_load::<_, JsonValue>(&PluginLoadRequest {
                route: ResolvedRoute {
                    pattern: route.clone(),
                    params: BTreeMap::new(),
                    query: BTreeMap::new(),
                },
                state_snapshot: BTreeMap::new(),
                locale: Some("en".to_string()),
            })
            .expect("load");

        let overview = bridge
            .call_plugin_render::<_, ScreenNode>(&PluginRenderRequest {
                route: ResolvedRoute {
                    pattern: route.clone(),
                    params: BTreeMap::new(),
                    query: BTreeMap::new(),
                },
                data: json!({
                    "title": "Smoke test",
                    "hostMessage": format!("Loaded from {}", wasm_path.display()),
                }),
                state_snapshot: BTreeMap::new(),
                locale: Some("en".to_string()),
            })
            .expect("overview render");
        assert!(overview.title.is_some());

        let inspect = bridge
            .call_plugin_render::<_, ScreenNode>(&PluginRenderRequest {
                route: ResolvedRoute {
                    pattern: route.clone(),
                    params: BTreeMap::new(),
                    query: BTreeMap::from([("view".to_string(), "inspect".to_string())]),
                },
                data: json!({
                    "title": "Smoke test",
                    "hostMessage": format!("Loaded from {}", wasm_path.display()),
                }),
                state_snapshot: BTreeMap::new(),
                locale: Some("en".to_string()),
            })
            .expect("inspect render");
        assert!(inspect.subtitle.is_some());

        let dispatch = bridge
            .call_plugin_dispatch::<PluginDispatchResponse>(&PluginDispatchRequest {
                route: ResolvedRoute {
                    pattern: route.clone(),
                    params: BTreeMap::new(),
                    query: BTreeMap::from([("view".to_string(), "inspect".to_string())]),
                },
                action: ActionRef {
                    r#type: ActionType::Custom("sanity.go-home".to_string()),
                    params: None,
                    confirm: None,
                },
                state_snapshot: BTreeMap::new(),
                locale: Some("en".to_string()),
            })
            .expect("dispatch");
        assert!(dispatch.handled);

        let rerender = bridge
            .call_plugin_render::<_, ScreenNode>(&PluginRenderRequest {
                route: ResolvedRoute {
                    pattern: route,
                    params: BTreeMap::new(),
                    query: BTreeMap::new(),
                },
                data: json!({
                    "title": "Smoke test",
                    "hostMessage": format!("Loaded from {}", wasm_path.display()),
                }),
                state_snapshot: BTreeMap::new(),
                locale: Some("en".to_string()),
            })
            .expect("rerender after dispatch");
        assert!(rerender.title.is_some());
    }

    #[test]
    fn warns_when_plugin_source_is_newer_than_wasm() {
        let plugin_root = PathBuf::from("plugins/sanity-check");
        let Some(wasm_path) = find_plugin_wasm(&plugin_root) else {
            return;
        };

        let _ = PermissionProfile {
            plugin_id: "mgn.shell".to_string(),
            grants: vec![],
        };
        assert!(wasm_path.exists());
    }

    #[test]
    fn prefers_newest_wasm_artifact() {
        let plugin_root = std::env::temp_dir().join(format!("mgn-test-{}", std::process::id()));
        let _ = fs::remove_dir_all(&plugin_root);
        let debug_path = plugin_root.join("target/wasm32-unknown-unknown/debug/sanity_check_plugin.wasm");
        let release_path = plugin_root.join("target/wasm32-unknown-unknown/release/sanity_check_plugin.wasm");

        fs::create_dir_all(debug_path.parent().expect("debug parent")).expect("debug dir");
        fs::create_dir_all(release_path.parent().expect("release parent")).expect("release dir");

        fs::write(&debug_path, b"debug").expect("write debug");
        sleep(Duration::from_millis(10));
        fs::write(&release_path, b"release").expect("write release");

        let selected = find_plugin_wasm(&plugin_root).expect("selected wasm");
        assert_eq!(selected, release_path);
        let _ = fs::remove_dir_all(&plugin_root);
    }

    #[test]
    fn app_survives_three_full_plugin_navigation_cycles() {
        let mut app = match App::new() {
            Ok(app) => app,
            Err(_) => return,
        };

        let plugin_route = app
            .plugins
            .first()
            .map(|plugin| plugin.route.clone())
            .expect("sanity plugin route");

        for _ in 0..3 {
            app.navigate_to(plugin_route.clone());
            app.focused_pane = TuiFocusedPane::Main;
            if app.main_interactions.is_empty() {
                match &app.main_panel {
                    TuiMainContent::Panel(panel) => panic!("plugin panel fallback: {:?}", panel.lines),
                    TuiMainContent::Screen(screen) => panic!("screen without interactions: {:?}", screen.screen),
                }
            }

            let inspect_index = app
                .main_interactions
                .iter()
                .position(|interaction| interaction.label.contains("Inspect"))
                .unwrap_or_else(|| panic!("inspect interaction not found: {:?}", app.main_interactions));
            app.selected_main_interaction = Some(inspect_index);
            app.activate_main_interaction().expect("open inspect tab");
            assert_eq!(app.current_route, format!("{plugin_route}?view=inspect"));

            let go_home_index = app
                .main_interactions
                .iter()
                .position(|interaction| interaction.label.contains("Go home via plugin dispatch"))
                .unwrap_or_else(|| panic!("go-home interaction not found: {:?}", app.main_interactions));
            app.selected_main_interaction = Some(go_home_index);
            app.activate_main_interaction().expect("go home");
            assert_eq!(app.current_route, "/home");
        }

        app.navigate_to(plugin_route.clone());
        app.focused_pane = TuiFocusedPane::Main;
        if app.main_interactions.is_empty() {
            match &app.main_panel {
                TuiMainContent::Panel(panel) => panic!("plugin panel fallback: {:?}", panel.lines),
                TuiMainContent::Screen(screen) => panic!("screen without interactions: {:?}", screen.screen),
            }
        }
        let inspect_index = app
            .main_interactions
            .iter()
            .position(|interaction| interaction.label.contains("Inspect"))
            .unwrap_or_else(|| panic!("inspect interaction not found: {:?}", app.main_interactions));
        app.selected_main_interaction = Some(inspect_index);
        app.activate_main_interaction().expect("open inspect tab 4th");
        assert_eq!(app.current_route, format!("{plugin_route}?view=inspect"));
    }
}
