//! `actions` — one capability: **dispatch outcomes**.
//!
//! Every button is an `ActionRef` (a name plus optional params — never a
//! callback; closures don't survive the sandbox boundary). The host sends it
//! to `plugin_dispatch`, and the *outcome* in the response tells the host
//! what to do next:
//!
//! - `None`             — nothing beyond the message (and any state writes)
//! - `RefreshCurrentScreen` — re-render this screen
//! - `Navigate { to }`  — go somewhere else
//!
//! The last button shows that one dispatch can do several things at once:
//! write state (a capability call) *and* return a `Navigate` outcome — one
//! click, two effects, still a single request/response.

use std::collections::BTreeMap;

use serde_json::{Value as JsonValue, json};
use unode_plugin_sdk::prelude::{
    self as ui, ActionIntent, ActionRef, ActionType, IntoNode, PluginDispatchOutcome,
    PluginDispatchRequest, PluginDispatchResponse, PluginLoadRequest, PluginManifestEnvelope,
    PluginRenderRequest, ScreenNode, StateKey, TextRole, Tone, builtin, perm,
};

const PLUGIN_ID: &str = "dev.unode.actions";
const PLUGIN_NAME: &str = "Actions & Outcomes";
const ROUTE_PATH: &str = "/plugins/actions";
const HOME_PATH: &str = "/home";

/// Counts how many times any outcome button was pressed — written by the
/// dispatch handler, read by a binding on the screen.
const PRESSES: StateKey<u32> = StateKey::new("actions.presses");
const LAST_OUTCOME: StateKey<String> = StateKey::new("actions.lastOutcome");

fn manifest_envelope() -> PluginManifestEnvelope {
    ui::plugin_manifest(PLUGIN_ID, PLUGIN_NAME)
        .version("0.1.0")
        .description(
            "Didactic tour of dispatch outcomes: message, refresh, navigate, and combined effects.",
        )
        .author("unode")
        .permission(
            perm(builtin::NAVIGATION_WRITE)
                .required(false)
                .reason("The navigate buttons return Navigate outcomes."),
        )
        .route(unode_plugin_sdk::route(ROUTE_PATH).label("Actions"))
        .envelope()
}

fn load_response(request: &PluginLoadRequest) -> JsonValue {
    json!({ "loaded": true, "pluginId": PLUGIN_ID, "route": request.route.pattern })
}

fn custom(action: &str) -> ActionRef {
    ActionRef {
        r#type: ActionType::Custom(action.to_string()),
        params: None,
        confirm: None,
    }
}

fn render_screen(_request: &PluginRenderRequest) -> ScreenNode {
    ui::screen()
        .id("actions.screen")
        .title(PLUGIN_NAME)
        .subtitle("A button is an ActionRef; what happens next is the dispatch outcome.")
        .initial_state(BTreeMap::from([
            (PRESSES.path().to_string(), json!(0)),
            (LAST_OUTCOME.path().to_string(), json!("none yet")),
        ]))
        .children(ui::nodes![
            ui::text(
                "Plugins never receive callbacks — a click crosses the sandbox as a named \
                 action, and the response's outcome tells the host what to do next.",
            )
            .role(TextRole::Body)
            .tone(Tone::Info),
            ui::text(PRESSES.bind_text())
                .id("actions.presses")
                .role(TextRole::Caption)
                .tone(Tone::Muted),
            ui::text(LAST_OUTCOME.bind_text())
                .id("actions.last")
                .role(TextRole::Caption)
                .tone(Tone::Muted),
            ui::actions()
                .id("actions.buttons")
                .children([
                    ui::action("Outcome: None (message only)", custom("outcome.none"))
                        .id("actions.none")
                        .intent(ActionIntent::Secondary),
                    ui::action("Outcome: Refresh", custom("outcome.refresh"))
                        .id("actions.refresh")
                        .intent(ActionIntent::Secondary),
                    ui::action("Outcome: Navigate home", custom("outcome.navigate"))
                        .id("actions.navigate")
                        .intent(ActionIntent::Secondary),
                    ui::action("Write state + navigate (one click)", custom("outcome.both"))
                        .id("actions.both")
                        .intent(ActionIntent::Primary),
                ])
                .into_node(),
        ])
        .initial_focus("actions.none")
        .build()
}

/// Bumps the press counter and records which outcome the button produced —
/// state writes are capability calls, independent from the outcome value.
fn record(request: &PluginDispatchRequest, outcome: &str) -> u32 {
    let next = PRESSES.get_or(&request.state_snapshot, 0) + 1;
    PRESSES.set(next);
    LAST_OUTCOME.set(format!("last outcome: {outcome}"));
    next
}

fn dispatch_response(request: &PluginDispatchRequest) -> PluginDispatchResponse {
    match &request.action.r#type {
        ActionType::Custom(action) if action == "outcome.none" => {
            let presses = record(request, "None");
            PluginDispatchResponse {
                handled: true,
                outcome: PluginDispatchOutcome::None,
                message: Some(format!("press #{presses}: nothing else happens")),
                data: None,
            }
        }
        ActionType::Custom(action) if action == "outcome.refresh" => {
            let presses = record(request, "RefreshCurrentScreen");
            PluginDispatchResponse {
                handled: true,
                outcome: PluginDispatchOutcome::RefreshCurrentScreen,
                message: Some(format!("press #{presses}: host re-renders this screen")),
                data: None,
            }
        }
        ActionType::Custom(action) if action == "outcome.navigate" => {
            let presses = record(request, "Navigate");
            PluginDispatchResponse {
                handled: true,
                outcome: PluginDispatchOutcome::Navigate {
                    to: HOME_PATH.to_string(),
                },
                message: Some(format!("press #{presses}: host navigates away")),
                data: None,
            }
        }
        // One click, two effects: capability calls (state writes) AND a
        // Navigate outcome in the same dispatch.
        ActionType::Custom(action) if action == "outcome.both" => {
            let presses = record(request, "state writes + Navigate");
            PluginDispatchResponse {
                handled: true,
                outcome: PluginDispatchOutcome::Navigate {
                    to: ROUTE_PATH.to_string(),
                },
                message: Some(format!("press #{presses}: wrote state and navigated")),
                data: None,
            }
        }
        _ => PluginDispatchResponse {
            handled: false,
            outcome: PluginDispatchOutcome::None,
            message: Some("actions plugin ignored an unknown action".to_string()),
            data: None,
        },
    }
}

unode_plugin_sdk::export_plugin! {
    manifest: manifest_envelope,
    load: load_response,
    render: render_screen,
    dispatch: dispatch_response,
}

#[cfg(test)]
mod tests {
    use super::*;
    use unode_plugin_sdk::prelude::ResolvedRoute;

    fn dispatch_req(action: &str, presses: u32) -> PluginDispatchRequest {
        PluginDispatchRequest {
            route: ResolvedRoute {
                pattern: ROUTE_PATH.to_string(),
                params: BTreeMap::new(),
                query: BTreeMap::new(),
            },
            action: custom(action),
            state_snapshot: BTreeMap::from([(PRESSES.path().to_string(), json!(presses))]),
            locale: Some("en".to_string()),
        }
    }

    #[test]
    fn manifest_declares_route_and_permission() {
        let manifest = manifest_envelope().manifest;
        assert!(manifest.validate().is_ok());
        assert_eq!(manifest.routes[0].pattern, ROUTE_PATH);
    }

    #[test]
    fn each_button_maps_to_its_outcome() {
        ui::host::clear_recorded_host_calls();

        let none = dispatch_response(&dispatch_req("outcome.none", 0));
        assert!(matches!(none.outcome, PluginDispatchOutcome::None));

        let refresh = dispatch_response(&dispatch_req("outcome.refresh", 1));
        assert!(matches!(
            refresh.outcome,
            PluginDispatchOutcome::RefreshCurrentScreen
        ));

        let navigate = dispatch_response(&dispatch_req("outcome.navigate", 2));
        assert!(
            matches!(navigate.outcome, PluginDispatchOutcome::Navigate { to } if to == HOME_PATH)
        );

        // The combined button issues state writes AND navigates.
        ui::host::clear_recorded_host_calls();
        let both = dispatch_response(&dispatch_req("outcome.both", 3));
        assert!(matches!(both.outcome, PluginDispatchOutcome::Navigate { to } if to == ROUTE_PATH));
        let writes = ui::host::recorded_host_calls();
        assert_eq!(writes.len(), 2, "presses + lastOutcome: {writes:?}");
    }
}
