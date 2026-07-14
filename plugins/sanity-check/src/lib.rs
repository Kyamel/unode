use serde_json::{json, Value as JsonValue};
use unode_sdk::prelude::{
    self as ui, create_route_tabs_meta, ActionIntent, ActionRef, ActionType, CoreActionType,
    IntoNode, PluginDispatchOutcome, PluginDispatchRequest, PluginDispatchResponse,
    PluginLoadRequest, PluginManifestEnvelope, PluginRenderRequest, ScreenNode, ScreenRouteTab,
    TextRole, Tone, UNODE_PLUGIN_ABI_VERSION, with_route_tabs,
};

#[cfg(test)]
use unode_sdk::prelude::{StringOrExpr, UiNode};

const PLUGIN_ID: &str = "dev.mugens.sanity-check";
const PLUGIN_NAME: &str = "Sanity Check";
const ROUTE_PATH: &str = "/plugins/sanity-check";

fn manifest_envelope() -> PluginManifestEnvelope {
    PluginManifestEnvelope {
        abi_version: UNODE_PLUGIN_ABI_VERSION.to_string(),
        manifest: unode_sdk::plugin_manifest(PLUGIN_ID, PLUGIN_NAME)
            .version("0.1.0")
            .description("Runtime-loaded WASM sanity-check plugin for the MGN TUI shell.")
            .author("Codex")
            .build(),
    }
}

fn route_tabs() -> Vec<ScreenRouteTab> {
    vec![
        ScreenRouteTab {
            id: "overview".to_string(),
            label: "Overview".to_string(),
            to: ROUTE_PATH.to_string(),
            badge: Some("wasm".to_string()),
        },
        ScreenRouteTab {
            id: "inspect".to_string(),
            label: "Inspect".to_string(),
            to: format!("{ROUTE_PATH}?view=inspect"),
            badge: None,
        },
    ]
}

fn load_response(request: &PluginLoadRequest) -> JsonValue {
    json!({
        "loaded": true,
        "pluginId": PLUGIN_ID,
        "route": request.route.pattern,
        "locale": request.locale,
    })
}

fn render_screen(request: &PluginRenderRequest) -> ScreenNode {
    let locale = request.locale.as_deref().unwrap_or("en");
    let active_tab = request
        .route
        .query
        .get("view")
        .map(String::as_str)
        .unwrap_or("overview");
    let title = request
        .data
        .get("title")
        .and_then(JsonValue::as_str)
        .unwrap_or("Runtime-loaded plugin");
    let host_message = request
        .data
        .get("hostMessage")
        .and_then(JsonValue::as_str)
        .unwrap_or("Host data not provided.");

    let params_summary = if request.route.params.is_empty() {
        "none".to_string()
    } else {
        request
            .route
            .params
            .iter()
            .map(|(key, value)| format!("{key}={value}"))
            .collect::<Vec<_>>()
            .join(", ")
    };

    let screen = ui::screen()
        .id("sanity-check.screen")
        .title(format!("{PLUGIN_NAME} - {title}"))
        .subtitle(format!("route={} locale={locale}", request.route.pattern))
        .children([
            ui::text("This screen is rendered from a Rust plugin compiled to WebAssembly.")
                .role(TextRole::Body)
                .tone(Tone::Info)
                .into_node(),
            ui::text(format!("Host message: {host_message}"))
                .role(TextRole::Body)
                .into_node(),
            ui::text(format!("Route params: {params_summary}"))
                .role(TextRole::Caption)
                .tone(Tone::Muted)
                .into_node(),
            ui::stack()
                .id("sanity-check.details")
                .children([
                    ui::text("The plugin is isolated behind the unode WASM ABI.")
                        .role(TextRole::Body)
                        .into_node(),
                    ui::text("The TUI runtime is responsible for sandboxing and host calls.")
                        .role(TextRole::Body)
                        .into_node(),
                ])
                .into_node(),
            ui::actions()
                .id("sanity-check.actions")
                .children([
                    ui::action(
                        "Refresh screen",
                        ActionRef {
                            r#type: ActionType::Custom("sanity.refresh".to_string()),
                            params: None,
                            confirm: None,
                        },
                    )
                    .id("sanity-check.refresh")
                    .intent(ActionIntent::Primary),
                    ui::action(
                        "Open inspect tab",
                        ActionRef {
                            r#type: ActionType::Core(CoreActionType::Navigate),
                            params: Some(std::collections::BTreeMap::from([(
                                "to".to_string(),
                                json!(format!("{ROUTE_PATH}?view=inspect")),
                            )])),
                            confirm: None,
                        },
                    )
                    .id("sanity-check.inspect")
                    .intent(ActionIntent::Secondary),
                    ui::action(
                        "Go home via plugin dispatch",
                        ActionRef {
                            r#type: ActionType::Custom("sanity.go-home".to_string()),
                            params: None,
                            confirm: None,
                        },
                    )
                    .id("sanity-check.go-home")
                    .intent(ActionIntent::Ghost),
                ])
                .into_node(),
        ])
        .initial_focus("sanity-check.refresh")
        .build();

    with_route_tabs(
        screen,
        create_route_tabs_meta(active_tab, route_tabs())
            .swipe_enabled(true)
            .swipe_threshold(48.0),
    )
}

fn encode_action_response(request: &PluginDispatchRequest) -> PluginDispatchResponse {
    match &request.action.r#type {
        ActionType::Custom(action) if action == "sanity.refresh" => PluginDispatchResponse {
            handled: true,
            outcome: PluginDispatchOutcome::RefreshCurrentScreen,
            message: Some(format!("Plugin requested refresh for {}", request.route.pattern)),
            data: None,
        },
        ActionType::Custom(action) if action == "sanity.go-home" => PluginDispatchResponse {
            handled: true,
            outcome: PluginDispatchOutcome::Navigate {
                to: "/home".to_string(),
            },
            message: Some("Plugin requested navigation to /home".to_string()),
            data: None,
        },
        _ => PluginDispatchResponse {
            handled: false,
            outcome: PluginDispatchOutcome::None,
            message: Some(format!(
                "Plugin ignored action on {}",
                request.route.pattern
            )),
            data: Some(json!({
                "pluginId": PLUGIN_ID,
                "actionType": request.action.r#type,
            })),
        },
    }
}

#[cfg(test)]
fn flatten_lines(screen: &ScreenNode) -> Vec<String> {
    let mut lines = Vec::new();
    if let Some(title) = &screen.title {
        lines.push(format!("title: {}", render_text_value(title)));
    }
    if let Some(subtitle) = &screen.subtitle {
        lines.push(format!("subtitle: {}", render_text_value(subtitle)));
    }
    for child in &screen.children {
        collect_node_lines(child, 0, &mut lines);
    }
    lines
}

#[cfg(test)]
fn collect_node_lines(node: &UiNode, depth: usize, lines: &mut Vec<String>) {
    let prefix = "  ".repeat(depth);
    match node {
        UiNode::Text(text) => lines.push(format!("{prefix}- {}", render_text_value(&text.content))),
        UiNode::Stack(stack) => {
            lines.push(format!("{prefix}[stack]"));
            for child in &stack.children {
                collect_node_lines(child, depth + 1, lines);
            }
        }
        UiNode::Section(section) => {
            let title = section
                .title
                .as_ref()
                .map(render_text_value)
                .unwrap_or_else(|| "section".to_string());
            lines.push(format!("{prefix}[section] {title}"));
            for child in &section.children {
                collect_node_lines(child, depth + 1, lines);
            }
        }
        other => lines.push(format!("{prefix}[node] {}", serde_json::to_string(other).unwrap_or_default())),
    }
}

#[cfg(test)]
fn render_text_value(value: &StringOrExpr) -> String {
    match value {
        StringOrExpr::Value(text) => text.clone(),
        StringOrExpr::Expr(expr) => serde_json::to_string(expr).unwrap_or_else(|_| "<expr>".to_string()),
    }
}

unode_sdk::export_plugin! {
    manifest: manifest_envelope,
    load: load_response,
    render: render_screen,
    dispatch: encode_action_response,
}

#[cfg(test)]
mod tests {
    use super::{flatten_lines, manifest_envelope, render_screen, ROUTE_PATH};
    use serde_json::json;
    use std::collections::BTreeMap;
    use unode_sdk::PluginRenderRequest;
    use unode_sdk::prelude::ResolvedRoute;

    #[test]
    fn manifest_has_expected_plugin_identity() {
        let manifest = manifest_envelope();
        assert_eq!(manifest.manifest.id, "dev.mugens.sanity-check");
        assert_eq!(manifest.manifest.name, "Sanity Check");
    }

    #[test]
    fn render_screen_contains_sanity_lines() {
        let screen = render_screen(&PluginRenderRequest {
            route: ResolvedRoute {
                pattern: ROUTE_PATH.to_string(),
                params: BTreeMap::new(),
                query: BTreeMap::new(),
            },
            data: json!({
                "title": "Smoke test",
                "hostMessage": "Loaded through Wasmtime"
            }),
            state_snapshot: BTreeMap::new(),
            locale: Some("pt-BR".to_string()),
        });

        let lines = flatten_lines(&screen);
        assert!(lines.iter().any(|line| line.contains("Loaded through Wasmtime")));
        assert!(lines.iter().any(|line| line.contains("Rust plugin compiled to WebAssembly")));
    }
}
