//! The demo host app — the TUI counterpart of the web examples' `App.tsx` /
//! `App.svelte`: it declares the renderer (defaults + the host Button),
//! starts a plugin session, and runs the render/dispatch loop.

use std::io;
use std::path::Path;
use std::time::Duration;

use anyhow::{Context, Result};
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use serde_json::json;
use unode::core::ast::ScreenNode;
use unode_plugin_sdk::prelude::ResolvedRoute;
use unode_plugin_sdk::{
    PluginDispatchRequest, PluginDispatchResponse, PluginLoadRequest, PluginRenderRequest,
};
use unode_ratatui_renderer::{
    TuiInteractiveKind, TuiRenderer, TuiScreenView, collect_screen_interactions, ratatui_renderer,
    render_tui_screen,
};
use unode_tui_runtime::{
    CachedTuiPlugin, PluginSession, PluginState, TuiHostCallDispatcher, resolve_screen_state,
};

use crate::button::button_recipe;

const ROUTE: &str = "/counter";
const LOCALE: &str = "en";

pub struct App {
    session: PluginSession,
    /// Host-side plugin state, fed by the plugin's `state.set` host calls.
    state: PluginState,
    renderer: TuiRenderer,
    plugin_id: String,
    route: ResolvedRoute,
    focused: Option<usize>,
}

impl App {
    pub fn new(wasm_path: &Path) -> Result<Self> {
        let state = PluginState::default();
        let plugin = CachedTuiPlugin::from_wasm_file(wasm_path, dispatcher(state.clone()))
            .context("instantiate counter plugin")?;
        let plugin_id = plugin.manifest().manifest.id.clone();

        // The renderer declaration: ratatui defaults for every node, with
        // `action` backed by the host's Button (see `button.rs`) — the same
        // shape as `defineRenderer().recipe("action", ...)` on the web.
        let renderer = ratatui_renderer().recipes([button_recipe()]).build();

        let route = ResolvedRoute {
            pattern: ROUTE.to_string(),
            params: Default::default(),
            query: Default::default(),
        };
        let session = plugin.start_session(&PluginLoadRequest {
            route: route.clone(),
            state_snapshot: state.snapshot(),
            locale: Some(LOCALE.to_string()),
        })?;

        Ok(Self {
            session,
            state,
            renderer,
            plugin_id,
            route,
            focused: Some(0),
        })
    }

    pub fn run(&mut self, terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
        loop {
            // Render through the sandbox, resolve bindings against host state.
            let mut screen: ScreenNode = self.session.render(&PluginRenderRequest {
                route: self.route.clone(),
                data: json!({}),
                state_snapshot: self.state.snapshot(),
                locale: Some(LOCALE.to_string()),
            })?;
            resolve_screen_state(&mut screen, &self.state);

            let interactions = collect_screen_interactions(&screen, None);
            self.focused = match (self.focused, interactions.len()) {
                (_, 0) => None,
                (Some(index), len) => Some(index.min(len - 1)),
                (None, _) => Some(0),
            };

            let view = TuiScreenView {
                plugin_id: self.plugin_id.clone(),
                source: "examples/tui-ratatui".to_string(),
                screen,
                route_tabs: None,
                focused_interaction: self.focused,
            };
            terminal.draw(|frame| {
                render_tui_screen(frame, frame.area(), &view, &self.renderer);
            })?;

            if !event::poll(Duration::from_millis(150))? {
                continue;
            }
            let Event::Key(key) = event::read()? else {
                continue;
            };
            if key.kind != KeyEventKind::Press {
                continue;
            }

            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => break,
                KeyCode::Left | KeyCode::Up => {
                    self.focused = self.focused.map(|index| index.saturating_sub(1));
                }
                KeyCode::Right | KeyCode::Down => {
                    self.focused = self
                        .focused
                        .map(|index| (index + 1).min(interactions.len().saturating_sub(1)));
                }
                KeyCode::Enter => {
                    let Some(interaction) = self.focused.and_then(|index| interactions.get(index))
                    else {
                        continue;
                    };
                    let action = match &interaction.kind {
                        TuiInteractiveKind::Action { action }
                        | TuiInteractiveKind::ListItem { action } => action.clone(),
                        TuiInteractiveKind::RouteTab { .. } => continue,
                    };
                    // The dispatch mutates host state through `state.set`
                    // host calls; the next iteration re-renders with it.
                    let _response: PluginDispatchResponse =
                        self.session.dispatch(&PluginDispatchRequest {
                            route: self.route.clone(),
                            action,
                            state_snapshot: self.state.snapshot(),
                            locale: Some(LOCALE.to_string()),
                        })?;
                }
                _ => {}
            }
        }

        Ok(())
    }
}

fn dispatcher(state: PluginState) -> TuiHostCallDispatcher {
    let mut dispatcher = TuiHostCallDispatcher::new();
    dispatcher.register("state.set", move |params| {
        let path = params
            .get("path")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| unode_tui_runtime::TuiHostCallError::Handler {
                operation: "state.set".to_string(),
                message: "missing string `path`".to_string(),
            })?;
        state.set(
            path.to_string(),
            params.get("value").cloned().unwrap_or(json!(null)),
        );
        Ok(json!({ "ok": true }))
    });
    dispatcher
}
