use std::collections::BTreeMap;

use serde_json::{Value as JsonValue, json};
use unode_plugin_sdk::prelude::{
    self as ui, ActionIntent, ActionRef, ActionType, IntoNode, PluginDispatchOutcome,
    PluginDispatchRequest, PluginDispatchResponse, PluginLoadRequest, PluginManifestEnvelope,
    PluginRenderRequest, ScreenNode, TextRole, Tone, UNODE_PLUGIN_ABI_VERSION, expr, perm,
};

const PLUGIN_ID: &str = "dev.unode.playground.split-host";
const PLUGIN_NAME: &str = "Split Screen Host";
const HOST_COUNT_PATH: &str = "split.hostRefreshes";
const HOST_LABEL_PATH: &str = "split.hostLabel";
const CONTRIBUTOR_COUNT_PATH: &str = "split.contributorApprovals";
const CONTRIBUTOR_LABEL_PATH: &str = "split.contributorLabel";

fn manifest_envelope() -> PluginManifestEnvelope {
    ui::plugin_manifest(PLUGIN_ID, PLUGIN_NAME)
        .version("0.1.0")
        .description("Owns a split screen and exposes SlotNode anchors for other plugins.")
        .author("unode")
        .permission(
            perm("screen.refresh")
                .required(true)
                .reason("Refresh the host-owned pane."),
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

fn host_label(count: i64) -> String {
    format!("Host refreshes: {count}")
}

fn contributor_label(count: i64) -> String {
    format!("Contributor approvals: {count}")
}

fn load_response(request: &PluginLoadRequest) -> JsonValue {
    json!({ "loaded": true, "pluginId": PLUGIN_ID, "route": request.route.pattern })
}

fn render_screen(_request: &PluginRenderRequest) -> ScreenNode {
    ui::screen()
        .id("playground-split-host.screen")
        .title(PLUGIN_NAME)
        .subtitle("Plugin A owns the layout. Plugin B is injected into named slots.")
        .initial_state(BTreeMap::from([
            (HOST_COUNT_PATH.to_string(), json!(0)),
            (HOST_LABEL_PATH.to_string(), json!(host_label(0))),
            (CONTRIBUTOR_COUNT_PATH.to_string(), json!(0)),
            (
                CONTRIBUTOR_LABEL_PATH.to_string(),
                json!(contributor_label(0)),
            ),
        ]))
        .children(ui::nodes![
            ui::grid()
                .id("playground-split-host.grid")
                .max_columns(2)
                .children(ui::nodes![
                    ui::section()
                        .id("playground-split-host.left")
                        .title("Host-owned pane")
                        .description("These actions dispatch to the host plugin.")
                        .children(ui::nodes![
                            ui::text(expr::binding::<String>(HOST_LABEL_PATH))
                                .id("playground-split-host.label")
                                .role(TextRole::Title)
                                .tone(Tone::Info),
                            ui::action("Refresh host pane", custom("split-host.refresh"))
                                .id("playground-split-host.refresh")
                                .intent(ActionIntent::Primary),
                        ])
                        .into_node(),
                    ui::section()
                        .id("playground-split-host.right")
                        .title("Contributed pane")
                        .description("Actions rendered here must dispatch to the contributor.")
                        .children(ui::nodes![
                            ui::slot("playground.split:right")
                                .id("playground-split-host.right-slot")
                                .fallback(
                                    ui::text("No contributor responded to playground.split:right.")
                                        .role(TextRole::Caption)
                                        .tone(Tone::Warning),
                                ),
                        ])
                        .into_node(),
                ])
                .into_node(),
            ui::section()
                .id("playground-split-host.footer")
                .title("Footer slot")
                .children(ui::nodes![
                    ui::slot("playground.split:footer")
                        .id("playground-split-host.footer-slot")
                        .fallback(ui::text("Footer slot fallback.").tone(Tone::Muted)),
                ])
                .into_node(),
        ])
        .initial_focus("playground-split-host.refresh")
        .build()
}

fn dispatch_response(request: &PluginDispatchRequest) -> PluginDispatchResponse {
    match &request.action.r#type {
        ActionType::Custom(action) if action == "split-host.refresh" => {
            let count = request
                .state_snapshot
                .get(HOST_COUNT_PATH)
                .and_then(JsonValue::as_i64)
                .unwrap_or(0)
                + 1;
            ui::host::state_set(HOST_COUNT_PATH, json!(count));
            ui::host::state_set(HOST_LABEL_PATH, json!(host_label(count)));
            PluginDispatchResponse {
                handled: true,
                outcome: PluginDispatchOutcome::None,
                message: Some(format!("host refresh -> {count}")),
                data: None,
            }
        }
        _ => PluginDispatchResponse {
            handled: false,
            outcome: PluginDispatchOutcome::None,
            message: Some("split-host ignored action".to_string()),
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
    fn render_contains_slot_anchors() {
        let encoded = serde_json::to_string(&render_screen(&PluginRenderRequest {
            route: Default::default(),
            data: json!({}),
            state_snapshot: Default::default(),
            locale: Some("en".to_string()),
        }))
        .unwrap();
        assert!(encoded.contains("playground.split:right"));
        assert!(encoded.contains("playground.split:footer"));
    }
}
