//! `host-slot` — one capability: **host components**.
//!
//! The plugin emits semantic `action` nodes and nothing else about their
//! looks. The host's renderer maps them to its own native components via
//! `hostSlot("Button")` (a React/Vue/Svelte/Solid component on the web, a
//! ratatui painter on the TUI). The lesson: plugins declare intent
//! (`primary`, `danger`, ...), hosts own appearance.

use std::collections::BTreeMap;

use serde_json::{Value as JsonValue, json};
use unode_plugin_sdk::prelude::{
    self as ui, ActionIntent, ActionRef, ActionType, IntoNode, PluginDispatchOutcome,
    PluginDispatchRequest, PluginDispatchResponse, PluginLoadRequest, PluginManifestEnvelope,
    PluginRenderRequest, ScreenNode, TextRole, Tone, expr, perm,
};

const PLUGIN_ID: &str = "dev.unode.host-components";
const PLUGIN_NAME: &str = "Host Components";
const COUNT_PATH: &str = "hostSlot.count";
const LABEL_PATH: &str = "hostSlot.label";

fn manifest_envelope() -> PluginManifestEnvelope {
    ui::plugin_manifest(PLUGIN_ID, PLUGIN_NAME)
        .version("0.1.0")
        .description("Demonstrates host-rendered Button slots driven by plugin action nodes.")
        .author("unode")
        .permission(
            perm("host-components.button")
                .required(true)
                .reason("Render native playground buttons for action nodes."),
        )
        .envelope()
}

fn custom(action: &str) -> ActionRef {
    ActionRef {
        r#type: ActionType::Custom(action.to_string()),
        params: None,
        confirm: None,
    }
}

fn label_for(count: i64) -> String {
    format!("Host slot clicks: {count}")
}

fn load_response(request: &PluginLoadRequest) -> JsonValue {
    json!({ "loaded": true, "pluginId": PLUGIN_ID, "route": request.route.pattern })
}

fn render_screen(_request: &PluginRenderRequest) -> ScreenNode {
    ui::screen()
        .id("host-components.screen")
        .title(PLUGIN_NAME)
        .subtitle("One capability: host components. Semantic action nodes; the host maps them to its native Button.")
        .initial_state(BTreeMap::from([
            (COUNT_PATH.to_string(), json!(0)),
            (LABEL_PATH.to_string(), json!(label_for(0))),
        ]))
        .children(ui::nodes![
            ui::text(expr::binding::<String>(LABEL_PATH))
                .id("host-components.count")
                .role(TextRole::Title)
                .tone(Tone::Info),
            ui::text("The plugin only declares intents (primary, secondary, ghost, danger). What a button looks like is decided by whichever host renders this screen.")
                .id("host-components.note")
                .role(TextRole::Caption)
                .tone(Tone::Muted),
            ui::actions()
                .id("host-components.actions")
                .children([
                    ui::action("Primary", custom("host-slot.primary"))
                        .id("host-components.primary")
                        .intent(ActionIntent::Primary),
                    ui::action("Secondary", custom("host-slot.secondary"))
                        .id("host-components.secondary")
                        .intent(ActionIntent::Secondary),
                    ui::action("Ghost", custom("host-slot.ghost"))
                        .id("host-components.ghost")
                        .intent(ActionIntent::Ghost),
                    ui::action("Danger", custom("host-slot.danger"))
                        .id("host-components.danger")
                        .intent(ActionIntent::Danger),
                ])
                .into_node(),
        ])
        .initial_focus("host-components.primary")
        .build()
}

fn current_count(request: &PluginDispatchRequest) -> i64 {
    request
        .state_snapshot
        .get(COUNT_PATH)
        .and_then(JsonValue::as_i64)
        .unwrap_or(0)
}

fn dispatch_response(request: &PluginDispatchRequest) -> PluginDispatchResponse {
    match &request.action.r#type {
        ActionType::Custom(action) if action.starts_with("host-slot.") => {
            let count = if action == "host-slot.danger" {
                0
            } else {
                current_count(request) + 1
            };
            ui::host::state_set(COUNT_PATH, json!(count));
            ui::host::state_set(LABEL_PATH, json!(label_for(count)));
            PluginDispatchResponse {
                handled: true,
                outcome: PluginDispatchOutcome::None,
                message: Some(format!("{action} -> {count}")),
                data: None,
            }
        }
        _ => PluginDispatchResponse {
            handled: false,
            outcome: PluginDispatchOutcome::None,
            message: Some("host-slot ignored action".to_string()),
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

    #[test]
    fn manifest_requests_host_slot_permission() {
        let manifest = manifest_envelope().manifest;
        assert_eq!(manifest.id, PLUGIN_ID);
        assert_eq!(manifest.permissions[0].permission, "host-components.button");
    }
}
