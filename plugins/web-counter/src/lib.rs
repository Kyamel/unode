//! `web-counter` — the reactive plugin behind the web vertical slice.
//!
//! It renders one reactive line bound to `ui.countLabel` plus three actions.
//! On dispatch it reads the current count from the `state_snapshot` the host
//! passed in, computes the next value, and requests state writes through the SDK
//! host-call helper. The host applies them to its store, which produces a single
//! patch op re-rendering only the bound line.
//!
//! State never lives inside the plugin's linear memory — it is owned by the
//! host store and handed back each dispatch. That is the sandbox boundary: the
//! plugin only declares intent and returns data.

use std::collections::BTreeMap;

use serde_json::{json, Value as JsonValue};

use unode_sdk::prelude::{
    self as ui, expr, ActionIntent, ActionRef, ActionType, IntoNode, PluginDispatchOutcome,
    PluginDispatchRequest, PluginDispatchResponse, PluginLoadRequest, PluginManifestEnvelope,
    PluginRenderRequest, ScreenNode, TextRole, Tone, UNODE_PLUGIN_ABI_VERSION,
};

const PLUGIN_ID: &str = "dev.unode.web-counter";
const PLUGIN_NAME: &str = "Web Counter";
#[cfg(test)]
const ROUTE_PATH: &str = "/plugins/web-counter";
const COUNT_PATH: &str = "ui.count";
const LABEL_PATH: &str = "ui.countLabel";

fn manifest_envelope() -> PluginManifestEnvelope {
    PluginManifestEnvelope {
        abi_version: UNODE_PLUGIN_ABI_VERSION.to_string(),
        manifest: unode_sdk::plugin_manifest(PLUGIN_ID, PLUGIN_NAME)
            .version("0.1.0")
            .description("Reactive counter proving the unode web runtime slice.")
            .author("unode")
            .build(),
    }
}

fn load_response(request: &PluginLoadRequest) -> JsonValue {
    json!({
        "loaded": true,
        "pluginId": PLUGIN_ID,
        "route": request.route.pattern,
    })
}

fn label_for(count: i64) -> String {
    format!("Count: {count}")
}

/// A plain Custom action with no params.
fn custom(action: &str) -> ActionRef {
    ActionRef {
        r#type: ActionType::Custom(action.to_string()),
        params: None,
        confirm: None,
    }
}

fn render_screen(_request: &PluginRenderRequest) -> ScreenNode {
    ui::screen()
        .id("web-counter.screen")
        .title(format!("{PLUGIN_NAME}"))
        .subtitle("Rendered from a Rust plugin compiled to WebAssembly.")
        .initial_state(BTreeMap::from([
            (COUNT_PATH.to_string(), json!(0)),
            (LABEL_PATH.to_string(), json!(label_for(0))),
        ]))
        .children(ui::nodes![
            // The one reactive node: its content is a binding, so the host
            // tracks it and patches only this line when `ui.countLabel` changes.
            ui::text(expr::binding::<String>(LABEL_PATH))
                .id("web-counter.value")
                .role(TextRole::Title)
                .tone(Tone::Info),
            ui::text("The number above is host state; the buttons dispatch intents.")
                .id("web-counter.hint")
                .role(TextRole::Caption)
                .tone(Tone::Muted),
            ui::actions()
                .id("web-counter.actions")
                .children([
                    ui::action("Increment", custom("counter.inc"))
                        .id("web-counter.inc")
                        .intent(ActionIntent::Primary),
                    ui::action("Decrement", custom("counter.dec"))
                        .id("web-counter.dec")
                        .intent(ActionIntent::Secondary),
                    ui::action("Reset", custom("counter.reset"))
                        .id("web-counter.reset")
                        .intent(ActionIntent::Ghost),
                ])
                .into_node(),
        ])
        .initial_focus("web-counter.inc")
        .build()
}

fn current_count(request: &PluginDispatchRequest) -> i64 {
    request
        .state_snapshot
        .get(COUNT_PATH)
        .and_then(JsonValue::as_i64)
        .unwrap_or(0)
}

/// Compute the next count for a known action, or `None` if unhandled.
fn next_count(request: &PluginDispatchRequest) -> Option<i64> {
    match &request.action.r#type {
        ActionType::Custom(a) if a == "counter.inc" => Some(current_count(request) + 1),
        ActionType::Custom(a) if a == "counter.dec" => Some(current_count(request) - 1),
        ActionType::Custom(a) if a == "counter.reset" => Some(0),
        _ => None,
    }
}

fn dispatch_response(request: &PluginDispatchRequest) -> PluginDispatchResponse {
    match next_count(request) {
        Some(count) => {
            // Writes cross the sandbox boundary as capability calls — the plugin
            // never returns UI state in its response.
            ui::host::state_set(COUNT_PATH, json!(count));
            ui::host::state_set(LABEL_PATH, json!(label_for(count)));
            PluginDispatchResponse {
                handled: true,
                outcome: PluginDispatchOutcome::None,
                message: Some(format!("count -> {count}")),
                data: None,
            }
        }
        None => PluginDispatchResponse {
            handled: false,
            outcome: PluginDispatchOutcome::None,
            message: Some("web-counter ignored action".to_string()),
            data: None,
        },
    }
}

unode_sdk::export_plugin! {
    manifest: manifest_envelope,
    load: load_response,
    render: render_screen,
    dispatch: dispatch_response,
}

#[cfg(test)]
mod tests {
    use super::*;
    use unode_sdk::prelude::ResolvedRoute;

    fn dispatch_req(action: &str, count: i64) -> PluginDispatchRequest {
        PluginDispatchRequest {
            route: ResolvedRoute {
                pattern: ROUTE_PATH.to_string(),
                params: BTreeMap::new(),
                query: BTreeMap::new(),
            },
            action: custom(action),
            state_snapshot: BTreeMap::from([(COUNT_PATH.to_string(), json!(count))]),
            locale: Some("en".to_string()),
        }
    }

    #[test]
    fn manifest_identity() {
        assert_eq!(manifest_envelope().manifest.id, PLUGIN_ID);
    }

    fn recorded_state_sets() -> Vec<(String, JsonValue)> {
        ui::host::recorded_host_calls()
            .iter()
            .filter(|env| env.operation == "state.set")
            .map(|env| {
                (
                    env.params["path"].as_str().unwrap_or_default().to_string(),
                    env.params["value"].clone(),
                )
            })
            .collect()
    }

    fn clear_recorded() {
        ui::host::clear_recorded_host_calls();
    }

    #[test]
    fn increment_issues_state_set_host_calls() {
        clear_recorded();
        let resp = dispatch_response(&dispatch_req("counter.inc", 4));
        assert!(resp.handled);
        assert!(resp.data.is_none(), "no UI state returned in the response");

        let sets = recorded_state_sets();
        assert!(sets.contains(&(COUNT_PATH.to_string(), json!(5))), "sets: {sets:?}");
        assert!(sets.contains(&(LABEL_PATH.to_string(), json!("Count: 5"))), "sets: {sets:?}");
    }

    #[test]
    fn reset_sets_zero() {
        clear_recorded();
        dispatch_response(&dispatch_req("counter.reset", 99));
        let sets = recorded_state_sets();
        assert!(sets.contains(&(COUNT_PATH.to_string(), json!(0))), "sets: {sets:?}");
    }

    #[test]
    fn unknown_action_is_unhandled_and_writes_nothing() {
        clear_recorded();
        let resp = dispatch_response(&dispatch_req("counter.spin", 1));
        assert!(!resp.handled);
        assert!(recorded_state_sets().is_empty());
    }
}
