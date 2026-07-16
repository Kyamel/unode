use std::io;
use std::time::Duration;

use anyhow::{Context, Result};
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use serde_json::{Value as JsonValue, json};
use tui_renderer::{
    TuiCommandBar, TuiFocusedPane, TuiInteractiveElement, TuiInteractiveKind, TuiMainContent,
    TuiMainPanel, TuiNavItem, TuiScreenView, TuiShellView, collect_screen_interactions,
    render_tui_shell,
};
use unode_runtime::{CommandResult, ShellContext};
use unode_sdk::prelude::{
    ActionRef, ActionType, CoreActionType, PermissionProfile, ResolvedRoute, ScreenNode,
    route_tabs_view,
};
use unode_sdk::{
    PluginDispatchOutcome, PluginDispatchRequest, PluginDispatchResponse, PluginLoadRequest,
    PluginRenderRequest,
};
use unode_tui_runtime::{PluginSession, TuiRuntime};

mod plugin_registry;
mod route;
mod shell_registry;
#[cfg(test)]
mod tests;

use plugin_registry::{LoadedPlugin, load_runtime_plugins, resolve_screen_state};
use route::parse_route;
use shell_registry::register_builtin_shell;

fn main() -> Result<()> {
    let mut app = App::new()?;
    app.run()
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
        self.shell.plugin_id = self
            .shell
            .route
            .as_ref()
            .map(|route| route.plugin_id.clone());
        self.shell.screen_kind = self
            .shell
            .route
            .as_ref()
            .map(|route| route.screen_kind.clone());
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
            TuiMainContent::Screen(view) => {
                collect_screen_interactions(&view.screen, view.route_tabs.as_ref())
            }
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
        let resolved = self.runtime.inner.routes.resolve(&parsed.pathname)?;
        let plugin_index = self.plugins.iter().position(|plugin| {
            plugin.runtime_plugin.manifest().manifest.id == resolved.plugin_id
        })?;
        let plugin = &self.plugins[plugin_index];
        let manifest = plugin.runtime_plugin.manifest().clone();
        let display_source = plugin.display_source.clone();
        let source_newer_than_wasm = plugin.source_newer_than_wasm;
        let state_snapshot = plugin.state.snapshot();
        let request = PluginRenderRequest {
            route: ResolvedRoute {
                pattern: resolved.pattern.clone(),
                params: resolved.params.clone(),
                query: parsed.query,
            },
            data: json!({
                "title": "Smoke test",
                "hostMessage": format!("Loaded from {}", display_source),
            }),
            state_snapshot,
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
                        "The plugin loaded, but the runtime could not start a plugin session."
                            .to_string(),
                        err.to_string(),
                    ],
                    footer: Some(
                        "Check the wasm artifact and runtime session lifecycle.".to_string(),
                    ),
                }));
            }
        };

        Some(match session.render::<ScreenNode>(&request) {
            Ok(mut screen) => {
                resolve_screen_state(&mut screen, &self.plugins[plugin_index].state);
                // Chrome is host-derived: tabs come from the manifest's route
                // groups, resolved against the freshest state snapshot.
                let route_tabs = route_tabs_view(
                    &manifest.manifest,
                    &request.route.pattern,
                    &self.plugins[plugin_index].state.snapshot(),
                );
                TuiMainContent::Screen(TuiScreenView {
                    plugin_id: manifest.manifest.id.clone(),
                    source: display_source,
                    screen,
                    route_tabs,
                    focused_interaction: None,
                })
            }
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
                state_snapshot: self.plugins[plugin_index].state.snapshot(),
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
                hint: Some("type a command id like `open.dev.unode.sanity-check`".to_string()),
            },
        }
    }

    fn focus_next_pane(&mut self) {
        self.focused_pane = if self.focused_pane == TuiFocusedPane::Navigation
            && !self.main_interactions.is_empty()
        {
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
                pattern: self
                    .shell
                    .route
                    .as_ref()
                    .map(|resolved| resolved.pattern.clone())
                    .filter(|pattern| !pattern.is_empty())
                    .unwrap_or(parsed.pathname),
                params: self
                    .shell
                    .route
                    .as_ref()
                    .map(|resolved| resolved.params.clone())
                    .unwrap_or_default(),
                query: parsed.query,
            },
            action,
            state_snapshot: self
                .plugins
                .iter()
                .find(|plugin| plugin.runtime_plugin.manifest().manifest.id == plugin_id)
                .map(|plugin| plugin.state.snapshot())
                .unwrap_or_default(),
            locale: self.shell.locale.clone(),
        };

        let plugin = self
            .plugins
            .iter()
            .position(|plugin| plugin.runtime_plugin.manifest().manifest.id == plugin_id)
            .with_context(|| format!("plugin `{plugin_id}` not loaded"))?;
        let session = self
            .ensure_plugin_session(
                plugin,
                &plugin_id,
                &self.current_route.clone(),
                &request.route,
                request.locale.clone(),
            )
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

        if response.handled {
            self.refresh_main_panel();
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
