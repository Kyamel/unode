use serde_json::{Value as JsonValue, json};
use unode_sdk::prelude::{
    self as ui, ActionIntent, ActionRef, ActionType, CoreActionType, IntoNode,
    PluginDispatchOutcome, PluginDispatchRequest, PluginDispatchResponse, PluginLoadRequest,
    PluginManifestEnvelope, PluginRenderRequest, ScreenNode, ScreenRouteTab, TextRole, Tone,
    UNODE_PLUGIN_ABI_VERSION, create_route_tabs_meta, permission, with_route_tabs,
};

const PLUGIN_ID: &str = "dev.unode.playground.route-tabs";
const PLUGIN_NAME: &str = "Route Tabs";
const ROUTE_PATH: &str = "/playground/route-tabs";

fn manifest_envelope() -> PluginManifestEnvelope {
    PluginManifestEnvelope {
        abi_version: UNODE_PLUGIN_ABI_VERSION.to_string(),
        manifest: ui::plugin_manifest(PLUGIN_ID, PLUGIN_NAME)
            .version("0.1.0")
            .description("Demonstrates route tab metadata and navigation outcomes.")
            .author("unode")
            .permission(
                permission("route.navigate")
                    .required(true)
                    .reason("Switch playground route tabs."),
            )
            .build(),
    }
}

fn route_tabs() -> Vec<ScreenRouteTab> {
    vec![
        ScreenRouteTab {
            id: "compose".to_string(),
            label: "Compose".to_string(),
            to: ROUTE_PATH.to_string(),
            badge: Some("slot".to_string()),
        },
        ScreenRouteTab {
            id: "review".to_string(),
            label: "Review".to_string(),
            to: format!("{ROUTE_PATH}?tab=review"),
            badge: None,
        },
        ScreenRouteTab {
            id: "ship".to_string(),
            label: "Ship".to_string(),
            to: format!("{ROUTE_PATH}?tab=ship"),
            badge: Some("3".to_string()),
        },
    ]
}

fn active_tab(request: &PluginRenderRequest) -> &str {
    request
        .route
        .query
        .get("tab")
        .map(String::as_str)
        .unwrap_or("compose")
}

fn navigate_action(tab: &str) -> ActionRef {
    ActionRef {
        r#type: ActionType::Core(CoreActionType::Navigate),
        params: Some(std::collections::BTreeMap::from([(
            "to".to_string(),
            json!(if tab == "compose" {
                ROUTE_PATH.to_string()
            } else {
                format!("{ROUTE_PATH}?tab={tab}")
            }),
        )])),
        confirm: None,
    }
}

fn load_response(request: &PluginLoadRequest) -> JsonValue {
    json!({ "loaded": true, "pluginId": PLUGIN_ID, "route": request.route.pattern })
}

fn render_screen(request: &PluginRenderRequest) -> ScreenNode {
    let active = active_tab(request);
    let title = match active {
        "review" => "Review route",
        "ship" => "Ship route",
        _ => "Compose route",
    };
    let screen = ui::screen()
        .id("playground-route-tabs.screen")
        .title(PLUGIN_NAME)
        .subtitle("Tabs are semantic screen chrome, not React-only state.")
        .children(ui::nodes![
            ui::section()
                .id("playground-route-tabs.panel")
                .title(title)
                .description(format!("Active tab id: {active}"))
                .children(ui::nodes![
                    ui::text("The host can render these tabs as browser tabs, TUI tabs, or another native pattern.")
                        .role(TextRole::Body)
                        .tone(Tone::Info),
                    ui::actions()
                        .id("playground-route-tabs.actions")
                        .children([
                            ui::action("Compose", navigate_action("compose"))
                                .id("playground-route-tabs.compose")
                                .intent(ActionIntent::Primary),
                            ui::action("Review", navigate_action("review"))
                                .id("playground-route-tabs.review")
                                .intent(ActionIntent::Secondary),
                            ui::action("Ship", navigate_action("ship"))
                                .id("playground-route-tabs.ship")
                                .intent(ActionIntent::Secondary),
                        ])
                        .into_node(),
                ])
                .into_node(),
        ])
        .initial_focus("playground-route-tabs.compose")
        .build();

    with_route_tabs(
        screen,
        create_route_tabs_meta(active, route_tabs())
            .swipe_enabled(true)
            .swipe_threshold(48.0),
    )
}

fn dispatch_response(request: &PluginDispatchRequest) -> PluginDispatchResponse {
    match &request.action.r#type {
        ActionType::Core(CoreActionType::Navigate) => PluginDispatchResponse {
            handled: true,
            outcome: PluginDispatchOutcome::Navigate {
                to: request
                    .action
                    .params
                    .as_ref()
                    .and_then(|params| params.get("to"))
                    .and_then(JsonValue::as_str)
                    .unwrap_or(ROUTE_PATH)
                    .to_string(),
            },
            message: Some("route tab navigation requested".to_string()),
            data: None,
        },
        _ => PluginDispatchResponse {
            handled: false,
            outcome: PluginDispatchOutcome::None,
            message: Some("route-tabs ignored action".to_string()),
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

    #[test]
    fn render_exposes_three_tabs() {
        let screen = render_screen(&PluginRenderRequest {
            route: unode_sdk::prelude::ResolvedRoute {
                pattern: ROUTE_PATH.to_string(),
                params: Default::default(),
                query: Default::default(),
            },
            data: json!({}),
            state_snapshot: Default::default(),
            locale: Some("en".to_string()),
        });
        assert_eq!(screen.route_tabs.unwrap().tabs.len(), 3);
    }
}
