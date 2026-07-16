use serde_json::{Value as JsonValue, json};
use unode_sdk::prelude::{
    self as ui, ActionIntent, ActionRef, ActionType, CoreActionType, IntoNode,
    PluginDispatchOutcome, PluginDispatchRequest, PluginDispatchResponse, PluginLoadRequest,
    PluginManifestEnvelope, PluginRenderRequest, ScreenNode, TextRole, Tone,
    UNODE_PLUGIN_ABI_VERSION, route_group,
};

#[cfg(test)]
use unode_sdk::prelude::{StringOrExpr, UiNode};

const PLUGIN_ID: &str = "dev.unode.sanity-check";
const PLUGIN_NAME: &str = "Sanity Check";
const ROUTE_PATH: &str = "/plugins/sanity-check";
const INSPECT_ROUTE_PATH: &str = "/plugins/sanity-check/inspect";

fn manifest_envelope() -> PluginManifestEnvelope {
    PluginManifestEnvelope {
        abi_version: UNODE_PLUGIN_ABI_VERSION.to_string(),
        manifest: unode_sdk::plugin_manifest(PLUGIN_ID, PLUGIN_NAME)
            .version("0.1.0")
            .description("Runtime-loaded WASM sanity-check plugin for the MGN TUI shell.")
            .author("Unode")
            // One plugin, two screens: the host registers both routes and
            // dispatches matching navigations back through `plugin_render`.
            // The group asks for tabs; the renderer decides whether to honor
            // it (tab bar) or present the routes as separate screens.
            .route_group(route_group("main").tabs())
            .routes([
                unode_sdk::route(ROUTE_PATH)
                    .screen_kind(format!("{PLUGIN_ID}.overview"))
                    .group("main")
                    .label("Overview")
                    .badge("wasm"),
                unode_sdk::route(INSPECT_ROUTE_PATH)
                    .screen_kind(format!("{PLUGIN_ID}.inspect"))
                    .group("main")
                    .label("Inspect"),
            ])
            .build(),
    }
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
    // Branch on the matched route pattern to pick the screen to render.
    match request.route.pattern.as_str() {
        INSPECT_ROUTE_PATH => render_inspect_screen(request),
        _ => render_overview_screen(request),
    }
}

fn render_inspect_screen(request: &PluginRenderRequest) -> ScreenNode {
    let locale = request.locale.as_deref().unwrap_or("en");
    let query_summary = if request.route.query.is_empty() {
        "none".to_string()
    } else {
        request
            .route
            .query
            .iter()
            .map(|(key, value)| format!("{key}={value}"))
            .collect::<Vec<_>>()
            .join(", ")
    };
    let state_summary = if request.state_snapshot.is_empty() {
        "empty".to_string()
    } else {
        request
            .state_snapshot
            .keys()
            .cloned()
            .collect::<Vec<_>>()
            .join(", ")
    };

    ui::screen()
        .id("sanity-check.inspect-screen")
        .title(format!("{PLUGIN_NAME} - Inspect"))
        .subtitle(format!("route={} locale={locale}", request.route.pattern))
        .children([
            ui::text("This is a second screen rendered by the same plugin.")
                .role(TextRole::Body)
                .tone(Tone::Info)
                .into_node(),
            ui::text(format!("Query params: {query_summary}"))
                .role(TextRole::Caption)
                .tone(Tone::Muted)
                .into_node(),
            ui::text(format!("State snapshot keys: {state_summary}"))
                .role(TextRole::Caption)
                .tone(Tone::Muted)
                .into_node(),
            ui::actions()
                .id("sanity-check.inspect.actions")
                .children([ui::action(
                    "Back to overview",
                    ActionRef {
                        r#type: ActionType::Core(CoreActionType::Navigate),
                        params: Some(std::collections::BTreeMap::from([(
                            "to".to_string(),
                            json!(ROUTE_PATH),
                        )])),
                        confirm: None,
                    },
                )
                .id("sanity-check.inspect.back")
                .intent(ActionIntent::Primary)])
                .into_node(),
        ])
        .initial_focus("sanity-check.inspect.back")
        .build()
}

fn render_overview_screen(request: &PluginRenderRequest) -> ScreenNode {
    let locale = request.locale.as_deref().unwrap_or("en");
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

    ui::screen()
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
                        "Open inspect screen",
                        ActionRef {
                            r#type: ActionType::Core(CoreActionType::Navigate),
                            params: Some(std::collections::BTreeMap::from([(
                                "to".to_string(),
                                json!(INSPECT_ROUTE_PATH),
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
        .build()
}

fn encode_action_response(request: &PluginDispatchRequest) -> PluginDispatchResponse {
    match &request.action.r#type {
        ActionType::Custom(action) if action == "sanity.refresh" => PluginDispatchResponse {
            handled: true,
            outcome: PluginDispatchOutcome::RefreshCurrentScreen,
            message: Some(format!(
                "Plugin requested refresh for {}",
                request.route.pattern
            )),
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
        other => lines.push(format!(
            "{prefix}[node] {}",
            serde_json::to_string(other).unwrap_or_default()
        )),
    }
}

#[cfg(test)]
fn render_text_value(value: &StringOrExpr) -> String {
    match value {
        StringOrExpr::Value(text) => text.clone(),
        StringOrExpr::Expr(expr) => {
            serde_json::to_string(expr).unwrap_or_else(|_| "<expr>".to_string())
        }
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
    use super::{ROUTE_PATH, flatten_lines, manifest_envelope, render_screen};
    use serde_json::json;
    use std::collections::BTreeMap;
    use unode_sdk::PluginRenderRequest;
    use unode_sdk::prelude::ResolvedRoute;

    #[test]
    fn manifest_has_expected_plugin_identity() {
        let manifest = manifest_envelope();
        assert_eq!(manifest.manifest.id, "dev.unode.sanity-check");
        assert_eq!(manifest.manifest.name, "Sanity Check");
    }

    #[test]
    fn manifest_declares_both_screens_as_routes() {
        let manifest = manifest_envelope().manifest;
        assert!(manifest.validate().is_ok());
        assert_eq!(manifest.routes.len(), 2);
        assert_eq!(manifest.routes[0].pattern, ROUTE_PATH);
        assert_eq!(manifest.routes[1].pattern, super::INSPECT_ROUTE_PATH);
        assert_eq!(
            manifest.routes[1].screen_kind.as_deref(),
            Some("dev.unode.sanity-check.inspect")
        );
        assert_eq!(manifest.route_groups.len(), 1);
        assert_eq!(manifest.routes[0].group.as_deref(), Some("main"));
        assert_eq!(manifest.routes[1].group.as_deref(), Some("main"));
    }

    #[test]
    fn render_screen_branches_on_route_pattern() {
        let inspect = render_screen(&PluginRenderRequest {
            route: ResolvedRoute {
                pattern: super::INSPECT_ROUTE_PATH.to_string(),
                params: BTreeMap::new(),
                query: BTreeMap::new(),
            },
            data: json!({}),
            state_snapshot: BTreeMap::new(),
            locale: None,
        });

        let lines = flatten_lines(&inspect);
        assert!(
            lines
                .iter()
                .any(|line| line.contains("second screen rendered by the same plugin"))
        );
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
        assert!(
            lines
                .iter()
                .any(|line| line.contains("Loaded through Wasmtime"))
        );
        assert!(
            lines
                .iter()
                .any(|line| line.contains("Rust plugin compiled to WebAssembly"))
        );
    }
}
