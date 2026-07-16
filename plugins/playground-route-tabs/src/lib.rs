use serde_json::{Value as JsonValue, json};
use unode_sdk::prelude::{
    self as ui, ActionIntent, ActionRef, ActionType, CoreActionType, IntoNode,
    PluginDispatchOutcome, PluginDispatchRequest, PluginDispatchResponse, PluginLoadRequest,
    PluginManifestEnvelope, PluginRenderRequest, ScreenNode, TextRole, Tone,
    UNODE_PLUGIN_ABI_VERSION, expr, permission, route_group,
};

const PLUGIN_ID: &str = "dev.unode.playground.route-tabs";
const PLUGIN_NAME: &str = "Route Tabs";
const COMPOSE_PATH: &str = "/playground/route-tabs";
const REVIEW_PATH: &str = "/playground/route-tabs/review";
const SHIP_PATH: &str = "/playground/route-tabs/ship";
const SHIP_COUNT_STATE: &str = "routeTabs.shipCount";

fn manifest_envelope() -> PluginManifestEnvelope {
    PluginManifestEnvelope {
        abi_version: UNODE_PLUGIN_ABI_VERSION.to_string(),
        manifest: ui::plugin_manifest(PLUGIN_ID, PLUGIN_NAME)
            .version("0.1.0")
            .description("Demonstrates manifest route groups with a tabs intent.")
            .author("unode")
            .permission(
                permission("route.navigate")
                    .required(true)
                    .reason("Switch playground route tabs."),
            )
            // Three screens grouped with a tabs intent. The host derives the
            // tab bar (and the active tab) from the matched route; the ship
            // badge is a state binding, so it updates with plugin state.
            .route_group(route_group("flow").tabs())
            .routes([
                unode_sdk::route(COMPOSE_PATH)
                    .group("flow")
                    .label("Compose")
                    .badge("slot"),
                unode_sdk::route(REVIEW_PATH).group("flow").label("Review"),
                unode_sdk::route(SHIP_PATH)
                    .group("flow")
                    .label("Ship")
                    .badge(expr::binding::<String>(SHIP_COUNT_STATE)),
            ])
            .build(),
    }
}

fn navigate_action(to: &str) -> ActionRef {
    ActionRef {
        r#type: ActionType::Core(CoreActionType::Navigate),
        params: Some(std::collections::BTreeMap::from([(
            "to".to_string(),
            json!(to),
        )])),
        confirm: None,
    }
}

fn load_response(request: &PluginLoadRequest) -> JsonValue {
    json!({ "loaded": true, "pluginId": PLUGIN_ID, "route": request.route.pattern })
}

fn render_screen(request: &PluginRenderRequest) -> ScreenNode {
    let (active, title) = match request.route.pattern.as_str() {
        REVIEW_PATH => ("review", "Review route"),
        SHIP_PATH => ("ship", "Ship route"),
        _ => ("compose", "Compose route"),
    };

    ui::screen()
        .id("playground-route-tabs.screen")
        .title(PLUGIN_NAME)
        .subtitle("Tabs come from manifest route groups; the renderer decides how to show them.")
        .initial_state(std::collections::BTreeMap::from([(
            SHIP_COUNT_STATE.to_string(),
            json!("3"),
        )]))
        .children(ui::nodes![
            ui::section()
                .id("playground-route-tabs.panel")
                .title(title)
                .description(format!("Active route: {}", request.route.pattern))
                .children(ui::nodes![
                    ui::text(format!(
                        "The host can render the `flow` group as browser tabs, TUI tabs, or plain routes. Active tab id: {active}."
                    ))
                    .role(TextRole::Body)
                    .tone(Tone::Info),
                    ui::actions()
                        .id("playground-route-tabs.actions")
                        .children([
                            ui::action("Compose", navigate_action(COMPOSE_PATH))
                                .id("playground-route-tabs.compose")
                                .intent(ActionIntent::Primary),
                            ui::action("Review", navigate_action(REVIEW_PATH))
                                .id("playground-route-tabs.review")
                                .intent(ActionIntent::Secondary),
                            ui::action("Ship", navigate_action(SHIP_PATH))
                                .id("playground-route-tabs.ship")
                                .intent(ActionIntent::Secondary),
                        ])
                        .into_node(),
                ])
                .into_node(),
        ])
        .initial_focus("playground-route-tabs.compose")
        .build()
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
                    .unwrap_or(COMPOSE_PATH)
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
    use std::collections::BTreeMap;
    use unode_sdk::prelude::route_tabs_view;

    #[test]
    fn manifest_groups_three_routes_as_tabs() {
        let manifest = manifest_envelope().manifest;
        assert!(manifest.validate().is_ok());
        assert_eq!(manifest.routes.len(), 3);
        assert_eq!(manifest.route_groups.len(), 1);

        // Host-side derivation: active comes from the matched route and the
        // ship badge resolves against the state snapshot.
        let state = BTreeMap::from([(SHIP_COUNT_STATE.to_string(), serde_json::json!("3"))]);
        let view = route_tabs_view(&manifest, REVIEW_PATH, &state).expect("tabs");
        assert_eq!(view.active, REVIEW_PATH);
        assert_eq!(view.tabs.len(), 3);
        assert_eq!(view.tabs[0].label, "Compose");
        assert_eq!(view.tabs[2].badge.as_deref(), Some("3"));

        // Without state the dynamic badge is simply omitted.
        let view = route_tabs_view(&manifest, SHIP_PATH, &BTreeMap::new()).expect("tabs");
        assert_eq!(view.tabs[2].badge, None);
    }

    #[test]
    fn render_branches_on_route_pattern() {
        let screen = render_screen(&PluginRenderRequest {
            route: unode_sdk::prelude::ResolvedRoute {
                pattern: SHIP_PATH.to_string(),
                params: Default::default(),
                query: Default::default(),
            },
            data: json!({}),
            state_snapshot: Default::default(),
            locale: Some("en".to_string()),
        });
        let title = screen.title.expect("title");
        assert!(format!("{title:?}").contains("Route Tabs"));
    }
}
