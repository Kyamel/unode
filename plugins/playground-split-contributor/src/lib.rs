use serde_json::{Value as JsonValue, json};
use unode_sdk::prelude::{
    self as ui, ActionIntent, ActionRef, ActionType, IntoNode, PluginDispatchOutcome,
    PluginDispatchRequest, PluginDispatchResponse, PluginLoadRequest, PluginManifestEnvelope,
    PluginRenderRequest, PluginRenderSlotRequest, PluginRenderSlotResponse, ScreenNode,
    SlotContributionDecl, TextRole, Tone, UNODE_PLUGIN_ABI_VERSION, expr, permission,
};

const PLUGIN_ID: &str = "dev.unode.playground.split-contributor";
const PLUGIN_NAME: &str = "Split Contributor";
const COUNT_PATH: &str = "split.contributorApprovals";
const LABEL_PATH: &str = "split.contributorLabel";
const HOST_COUNT_PATH: &str = "split.hostRefreshes";
const HOST_LABEL_PATH: &str = "split.hostLabel";

fn manifest_envelope() -> PluginManifestEnvelope {
    PluginManifestEnvelope {
        abi_version: UNODE_PLUGIN_ABI_VERSION.to_string(),
        manifest: ui::plugin_manifest(PLUGIN_ID, PLUGIN_NAME)
            .version("0.1.0")
            .description("Contributes UI into Split Screen Host slots.")
            .author("unode")
            .permission(
                permission("slot.contribute:playground.split")
                    .required(true)
                    .reason("Render UI into playground split slots."),
            )
            .permission(
                permission("contributor.approve")
                    .required(true)
                    .reason("Handle approval actions from contributed UI."),
            )
            .slot_contributions([
                SlotContributionDecl {
                    id: "right-approval".to_string(),
                    target: "playground.split:right".to_string(),
                    priority: 100,
                    when: None,
                },
                SlotContributionDecl {
                    id: "footer-status".to_string(),
                    target: "playground.split:footer".to_string(),
                    priority: 50,
                    when: None,
                },
            ])
            .build(),
    }
}

fn custom(action: &str) -> ActionRef {
    ActionRef {
        r#type: ActionType::Custom(action.to_string()),
        params: None,
        confirm: None,
    }
}

fn label_for(count: i64) -> String {
    format!("Contributor approvals: {count}")
}

fn host_label_for(count: i64) -> String {
    format!("Host refreshes: {count}")
}

fn load_response(request: &PluginLoadRequest) -> JsonValue {
    json!({ "loaded": true, "pluginId": PLUGIN_ID, "route": request.route.pattern })
}

fn render_screen(_request: &PluginRenderRequest) -> ScreenNode {
    ui::screen()
        .id("playground-split-contributor.screen")
        .title(PLUGIN_NAME)
        .subtitle("Select Split Screen Host to see this plugin rendered inside another plugin.")
        .children(ui::nodes![
            ui::text("This plugin declares slot contributions in its manifest.")
                .role(TextRole::Body)
                .tone(Tone::Info),
            ui::text("The action button is more interesting when injected into the host plugin.")
                .role(TextRole::Caption)
                .tone(Tone::Muted),
        ])
        .build()
}

fn render_slot(request: &PluginRenderSlotRequest) -> PluginRenderSlotResponse {
    if request.contribution_id == "footer-status" {
        return PluginRenderSlotResponse {
            nodes: vec![
                ui::inline()
                    .id("footer-status.inline")
                    .children(ui::nodes![
                        ui::badge("contributed by Split Contributor").tone(Tone::Info),
                        ui::text(format!("route={}", request.route.pattern))
                            .role(TextRole::Caption)
                            .tone(Tone::Muted),
                    ])
                    .into_node(),
            ],
        };
    }

    PluginRenderSlotResponse {
        nodes: vec![
            ui::stack()
                .id("right-approval.stack")
                .children(ui::nodes![
                    ui::text(expr::binding::<String>(LABEL_PATH))
                        .id("right-approval.label")
                        .role(TextRole::Title)
                        .tone(Tone::Success),
                    ui::text("This button is visible inside the host plugin, but dispatches to the contributor plugin.")
                        .id("right-approval.note")
                        .role(TextRole::Caption)
                        .tone(Tone::Muted),
                    ui::action("Set host refreshes to 3", custom("contributor.approve"))
                        .id("right-approval.approve")
                        .intent(ActionIntent::Primary),
                ])
                .into_node(),
        ],
    }
}

fn dispatch_response(request: &PluginDispatchRequest) -> PluginDispatchResponse {
    match &request.action.r#type {
        ActionType::Custom(action) if action == "contributor.approve" => {
            let count = request
                .state_snapshot
                .get(COUNT_PATH)
                .and_then(JsonValue::as_i64)
                .unwrap_or(0)
                + 1;
            ui::host::state_set(COUNT_PATH, json!(count));
            ui::host::state_set(LABEL_PATH, json!(label_for(count)));
            ui::host::state_set(HOST_COUNT_PATH, json!(3));
            ui::host::state_set(HOST_LABEL_PATH, json!(host_label_for(3)));
            PluginDispatchResponse {
                handled: true,
                outcome: PluginDispatchOutcome::None,
                message: Some(format!(
                    "contributor approval -> {count}; host refreshes -> 3"
                )),
                data: None,
            }
        }
        _ => PluginDispatchResponse {
            handled: false,
            outcome: PluginDispatchOutcome::None,
            message: Some("split-contributor ignored action".to_string()),
            data: None,
        },
    }
}

unode_sdk::export_plugin! {
    manifest: manifest_envelope,
    load: load_response,
    render: render_screen,
    dispatch: dispatch_response,
    render_slot: render_slot,
}

#[cfg(test)]
mod tests {
    use super::*;
    use unode_sdk::prelude::ResolvedRoute;

    #[test]
    fn manifest_declares_slot_contributions() {
        let manifest = manifest_envelope().manifest;
        assert_eq!(manifest.slot_contributions.len(), 2);
        assert_eq!(manifest.slot_contributions[0].target, "playground.split:right");
    }

    #[test]
    fn approve_sets_host_refresh_counter_to_three() {
        ui::host::clear_recorded_host_calls();
        let response = dispatch_response(&PluginDispatchRequest {
            route: ResolvedRoute::default(),
            action: custom("contributor.approve"),
            state_snapshot: Default::default(),
            locale: Some("en".to_string()),
        });

        assert!(response.handled);
        let calls = ui::host::recorded_host_calls();
        assert!(calls.iter().any(|call| {
            call.operation == "state.set"
                && call.params["path"] == HOST_COUNT_PATH
                && call.params["value"] == json!(3)
        }));
        assert!(calls.iter().any(|call| {
            call.operation == "state.set"
                && call.params["path"] == HOST_LABEL_PATH
                && call.params["value"] == json!(host_label_for(3))
        }));
    }
}
