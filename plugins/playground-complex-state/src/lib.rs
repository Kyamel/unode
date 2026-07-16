use std::collections::BTreeMap;

use serde_json::{Value as JsonValue, json};
use unode_sdk::prelude::{
    self as ui, ActionIntent, ActionRef, ActionType, IntoNode, PluginDispatchOutcome,
    PluginDispatchRequest, PluginDispatchResponse, PluginLoadRequest, PluginManifestEnvelope,
    PluginRenderRequest, ScreenNode, TextRole, Tone, UNODE_PLUGIN_ABI_VERSION, expr, permission,
};

const PLUGIN_ID: &str = "dev.unode.playground.complex-state";
const PLUGIN_NAME: &str = "Complex State";

fn manifest_envelope() -> PluginManifestEnvelope {
    PluginManifestEnvelope {
        abi_version: UNODE_PLUGIN_ABI_VERSION.to_string(),
        manifest: ui::plugin_manifest(PLUGIN_ID, PLUGIN_NAME)
            .version("0.1.0")
            .description("Structured host state demo with derived labels and item-level actions.")
            .author("unode")
            .permission(
                permission("state.write:tasks")
                    .required(true)
                    .reason("Cycle task state in the playground board."),
            )
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

fn labels(done: i64, doing: i64, todo: i64) -> BTreeMap<String, JsonValue> {
    BTreeMap::from([
        ("board.doneLabel".to_string(), json!(format!("Done: {done}"))),
        ("board.doingLabel".to_string(), json!(format!("Doing: {doing}"))),
        ("board.todoLabel".to_string(), json!(format!("Todo: {todo}"))),
    ])
}

fn load_response(request: &PluginLoadRequest) -> JsonValue {
    json!({ "loaded": true, "pluginId": PLUGIN_ID, "route": request.route.pattern })
}

fn extra_count(request: &PluginRenderRequest) -> i64 {
    request
        .state_snapshot
        .get("board.extraCount")
        .and_then(JsonValue::as_i64)
        .unwrap_or(0)
        .max(0)
}

fn render_screen(request: &PluginRenderRequest) -> ScreenNode {
    let extra_count = extra_count(request);
    let mut initial = labels(1, 1, 1);
    initial.insert("board.step".to_string(), json!(0));
    initial.insert("board.extraCount".to_string(), json!(0));

    let mut items = vec![
        ui::item("abi", ui::text("Stabilize render-slot ABI"))
            .secondary_child(ui::text("done").tone(Tone::Success)),
        ui::item("trust", ui::text("Preserve contributor action origin"))
            .secondary_child(ui::text("doing").tone(Tone::Warning)),
        ui::item("docs", ui::text("Document playground examples"))
            .secondary_child(ui::text("todo").tone(Tone::Info)),
    ];
    for index in 1..=extra_count {
        items.push(
            ui::item(
                format!("generated-{index}"),
                ui::text(format!("Generated task #{index}")),
            )
            .secondary_child(ui::text("added by Advance board").tone(Tone::Info)),
        );
    }

    ui::screen()
        .id("playground-complex-state.screen")
        .title(PLUGIN_NAME)
        .subtitle("A structured state snapshot drives several aligned labels.")
        .initial_state(initial)
        .children(ui::nodes![
            ui::grid()
                .id("playground-complex-state.metrics")
                .max_columns(3)
                .children(ui::nodes![
                    ui::text(expr::binding::<String>("board.doneLabel"))
                        .id("playground-complex-state.done")
                        .role(TextRole::Title)
                        .tone(Tone::Success),
                    ui::text(expr::binding::<String>("board.doingLabel"))
                        .id("playground-complex-state.doing")
                        .role(TextRole::Title)
                        .tone(Tone::Warning),
                    ui::text(expr::binding::<String>("board.todoLabel"))
                        .id("playground-complex-state.todo")
                        .role(TextRole::Title)
                        .tone(Tone::Info),
                ])
                .into_node(),
            ui::section()
                .id("playground-complex-state.board")
                .title("Task board")
                .description("Each action writes a small set of derived host-state paths.")
                .children(ui::nodes![
                    ui::list(items),
                    ui::actions()
                        .id("playground-complex-state.actions")
                        .children([
                            ui::action("Advance board", custom("board.advance"))
                                .id("playground-complex-state.advance")
                                .intent(ActionIntent::Primary),
                            ui::action("Reset board", custom("board.reset"))
                                .id("playground-complex-state.reset")
                                .intent(ActionIntent::Ghost),
                        ])
                        .into_node(),
                ])
                .into_node(),
        ])
        .initial_focus("playground-complex-state.advance")
        .build()
}

fn write_labels(done: i64, doing: i64, todo: i64) {
    for (path, value) in labels(done, doing, todo) {
        ui::host::state_set(&path, value);
    }
}

fn dispatch_response(request: &PluginDispatchRequest) -> PluginDispatchResponse {
    match &request.action.r#type {
        ActionType::Custom(action) if action == "board.advance" => {
            let step = request
                .state_snapshot
                .get("board.step")
                .and_then(JsonValue::as_i64)
                .unwrap_or(0)
                + 1;
            let done = 1 + (step % 3);
            let doing = 1 + ((step + 1) % 3);
            let todo = 3 - (done + doing - 2).min(2);
            ui::host::state_set("board.step", json!(step));
            ui::host::state_set("board.extraCount", json!(step));
            write_labels(done, doing, todo);
            PluginDispatchResponse {
                handled: true,
                outcome: PluginDispatchOutcome::RefreshCurrentScreen,
                message: Some(format!("board step -> {step}; added Generated task #{step}")),
                data: None,
            }
        }
        ActionType::Custom(action) if action == "board.reset" => {
            ui::host::state_set("board.step", json!(0));
            ui::host::state_set("board.extraCount", json!(0));
            write_labels(1, 1, 1);
            PluginDispatchResponse {
                handled: true,
                outcome: PluginDispatchOutcome::RefreshCurrentScreen,
                message: Some("board reset".to_string()),
                data: None,
            }
        }
        _ => PluginDispatchResponse {
            handled: false,
            outcome: PluginDispatchOutcome::None,
            message: Some("complex-state ignored action".to_string()),
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

    fn render_req_with_extra(extra_count: i64) -> PluginRenderRequest {
        PluginRenderRequest {
            route: Default::default(),
            data: json!({}),
            state_snapshot: BTreeMap::from([("board.extraCount".to_string(), json!(extra_count))]),
            locale: Some("en".to_string()),
        }
    }

    #[test]
    fn manifest_requests_state_permission() {
        assert_eq!(
            manifest_envelope().manifest.permissions[0].permission,
            "state.write:tasks"
        );
    }

    #[test]
    fn render_adds_generated_task_items_from_snapshot() {
        let encoded = serde_json::to_string(&render_screen(&render_req_with_extra(2))).unwrap();
        assert!(encoded.contains("Generated task #1"));
        assert!(encoded.contains("Generated task #2"));
    }
}
