use serde_json::{Value as JsonValue, json};
use unode_plugin_sdk::prelude::{
    self as ui, IntoNode, ListDensity, PluginDispatchOutcome, PluginDispatchRequest,
    PluginDispatchResponse, PluginLoadRequest, PluginManifestEnvelope, PluginRenderRequest,
    ScreenNode, TextRole, Tone, UNODE_PLUGIN_ABI_VERSION, ValueFormat, perm,
};

const PLUGIN_ID: &str = "dev.unode.playground.complex-layout";
const PLUGIN_NAME: &str = "Complex Layout";

fn manifest_envelope() -> PluginManifestEnvelope {
    ui::plugin_manifest(PLUGIN_ID, PLUGIN_NAME)
        .version("0.1.0")
        .description("Dense semantic layout demo with metrics, sections, badges, and lists.")
        .author("unode")
        .permission(
            perm("layout.read")
                .required(true)
                .reason("Read static demo layout data."),
        )
        .envelope()
}

fn load_response(request: &PluginLoadRequest) -> JsonValue {
    json!({ "loaded": true, "pluginId": PLUGIN_ID, "route": request.route.pattern })
}

fn metric(label: &str, value: i64, tone: Tone) -> ui::UiNode {
    ui::section()
        .title(label)
        .children(ui::nodes![
            ui::value(json!(value), ValueFormat::Number)
                .role(TextRole::Title)
                .tone(tone),
        ])
        .into_node()
}

fn render_screen(_request: &PluginRenderRequest) -> ScreenNode {
    ui::screen()
        .id("playground-complex-layout.screen")
        .title(PLUGIN_NAME)
        .subtitle("A bigger screen that stays pure semantic UI and renderer agnostic.")
        .children(ui::nodes![
            ui::grid()
                .id("playground-complex-layout.metrics")
                .max_columns(4)
                .children(vec![
                    metric("Slots", 4, Tone::Info),
                    metric("Routes", 3, Tone::Success),
                    metric("Patches", 12, Tone::Warning),
                    metric("Denied", 0, Tone::Danger),
                ])
                .into_node(),
            ui::grid()
                .id("playground-complex-layout.body")
                .max_columns(2)
                .children(ui::nodes![
                    ui::section()
                        .id("playground-complex-layout.queue")
                        .title("Work queue")
                        .description("Repeated rows are semantic list items, not host-specific table code.")
                        .children(ui::nodes![
                            ui::list(vec![
                                ui::item("normalize", ui::text("Normalize plugin IR"))
                                    .secondary_child(ui::text("Stable ids and action origin").tone(Tone::Muted)),
                                ui::item("track", ui::text("Track reactivity paths"))
                                    .secondary_child(ui::text("Patch only affected nodes").tone(Tone::Muted)),
                                ui::item("render", ui::text("Render host slots"))
                                    .secondary_child(ui::text("Map semantic action nodes to host buttons").tone(Tone::Muted)),
                            ])
                            .id("playground-complex-layout.queue-list")
                            .density(ListDensity::Comfortable),
                        ])
                        .into_node(),
                    ui::section()
                        .id("playground-complex-layout.inspector")
                        .title("Renderer notes")
                        .description("The host decides visual density, component recipes, and host-slot components.")
                        .children(ui::nodes![
                            ui::text("The same ScreenNode can be mounted by React, Svelte, or a TUI renderer.")
                                .role(TextRole::Body)
                                .tone(Tone::Info),
                            ui::inline()
                                .id("playground-complex-layout.badges")
                                .children(ui::nodes![
                                    ui::badge("serializable").tone(Tone::Success),
                                    ui::badge("sandboxed").tone(Tone::Info),
                                    ui::badge("renderer-free").tone(Tone::Muted),
                                ])
                                .into_node(),
                        ])
                        .into_node(),
                ])
                .into_node(),
        ])
        .build()
}

fn dispatch_response(_request: &PluginDispatchRequest) -> PluginDispatchResponse {
    PluginDispatchResponse {
        handled: false,
        outcome: PluginDispatchOutcome::None,
        message: Some("complex-layout is static".to_string()),
        data: None,
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
    fn render_has_layout_sections() {
        let encoded = serde_json::to_string(&render_screen(&PluginRenderRequest {
            route: Default::default(),
            data: json!({}),
            state_snapshot: Default::default(),
            locale: Some("en".to_string()),
        }))
        .unwrap();
        assert!(encoded.contains("Work queue"));
        assert!(encoded.contains("Renderer notes"));
    }
}
