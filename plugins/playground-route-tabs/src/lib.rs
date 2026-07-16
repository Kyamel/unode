use serde_json::{Value as JsonValue, json};
use unode_plugin_sdk::prelude::{
    self as ui, ActionIntent, ActionRef, ActionType, CoreActionType, IntoNode,
    PluginDispatchOutcome, PluginDispatchRequest, PluginDispatchResponse, PluginLoadRequest,
    PluginManifestEnvelope, PluginRenderRequest, ScreenNode, StateKey, TextRole, Tone,
    UNODE_PLUGIN_ABI_VERSION, perm, route_group,
};

const PLUGIN_ID: &str = "dev.unode.playground.route-tabs";
const PLUGIN_NAME: &str = "Route Tabs";
const COMPOSE_PATH: &str = "/playground/route-tabs";
const REVIEW_PATH: &str = "/playground/route-tabs/review";
const SHIP_PATH: &str = "/playground/route-tabs/ship";
// Typed state keys: the path is stated once; reads, writes, and bindings
// all agree on the value type.
const SHIP_COUNT: StateKey<u32> = StateKey::new("routeTabs.shipCount");
const LAST_SHIP: StateKey<String> = StateKey::new("routeTabs.lastShip");

fn manifest_envelope() -> PluginManifestEnvelope {
    ui::plugin_manifest(PLUGIN_ID, PLUGIN_NAME)
        .version("0.1.0")
        .description("Demonstrates manifest route groups with a tabs intent.")
        .author("unode")
        .permission(
            perm("route.navigate")
                .required(true)
                .reason("Switch playground route tabs."),
        )
        // Three screens grouped with a tabs intent. The host derives the
        // tab bar (and the active tab) from the matched route; the ship
        // badge is a state binding, so it updates with plugin state.
        .route_group(route_group("flow").tabs())
        .routes([
            unode_plugin_sdk::route(COMPOSE_PATH)
                .group("flow")
                .label("Compose")
                .badge("slot"),
            unode_plugin_sdk::route(REVIEW_PATH)
                .group("flow")
                .label("Review"),
            unode_plugin_sdk::route(SHIP_PATH)
                .group("flow")
                .label("Ship")
                .badge_bind(SHIP_COUNT.path()),
        ])
        .envelope()
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
        .initial_state(std::collections::BTreeMap::from([
            (SHIP_COUNT.path().to_string(), json!(3)),
            (LAST_SHIP.path().to_string(), json!("nothing shipped yet")),
        ]))
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
                    // One action, two state writes: the ship counter feeds
                    // the route badge (manifest binding) and the last-ship
                    // text below (screen binding).
                    ui::text(LAST_SHIP.bind_text())
                        .role(TextRole::Caption)
                        .tone(Tone::Muted),
                    ui::actions()
                        .id("playground-route-tabs.actions")
                        .children([
                            ui::action("Compose", navigate_action(COMPOSE_PATH))
                                .id("playground-route-tabs.compose")
                                .intent(ActionIntent::Primary),
                            ui::action("Review", navigate_action(REVIEW_PATH))
                                .id("playground-route-tabs.review")
                                .intent(ActionIntent::Secondary),
                            // One button, two effects: the dispatch handler
                            // writes state AND returns a Navigate outcome.
                            ui::action(
                                "Ship",
                                ActionRef {
                                    r#type: ActionType::Custom("ship.open".to_string()),
                                    params: None,
                                    confirm: None,
                                },
                            )
                            .id("playground-route-tabs.ship")
                            .intent(ActionIntent::Secondary),
                            ui::action(
                                "Ship one more",
                                ActionRef {
                                    r#type: ActionType::Custom("ship.increment".to_string()),
                                    params: None,
                                    confirm: None,
                                },
                            )
                            .id("playground-route-tabs.ship-one")
                            .intent(ActionIntent::Secondary),
                        ])
                        .into_node(),
                ])
                .into_node(),
        ])
        .initial_focus("playground-route-tabs.compose")
        .build()
}

/// Increments the ship counter and records the last-ship message — two
/// state writes shared by both ship actions.
fn ship_one(snapshot: &std::collections::BTreeMap<String, JsonValue>) -> u32 {
    let next = SHIP_COUNT.get_or(snapshot, 3) + 1;
    SHIP_COUNT.set(next);
    LAST_SHIP.set(format!("shipped #{next}"));
    next
}

fn dispatch_response(request: &PluginDispatchRequest) -> PluginDispatchResponse {
    match &request.action.r#type {
        // One button press mutating two state paths: the badge binding and
        // the on-screen text binding both pick the writes up.
        ActionType::Custom(action) if action == "ship.increment" => {
            let next = ship_one(&request.state_snapshot);
            PluginDispatchResponse {
                handled: true,
                outcome: PluginDispatchOutcome::RefreshCurrentScreen,
                message: Some(format!("ship count is now {next}")),
                data: None,
            }
        }
        // Same button: mutate state AND navigate, in one dispatch.
        ActionType::Custom(action) if action == "ship.open" => {
            let next = ship_one(&request.state_snapshot);
            PluginDispatchResponse {
                handled: true,
                outcome: PluginDispatchOutcome::Navigate {
                    to: SHIP_PATH.to_string(),
                },
                message: Some(format!("shipped #{next} and opened the ship screen")),
                data: None,
            }
        }
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

unode_plugin_sdk::export_plugin! {
    manifest: manifest_envelope,
    load: load_response,
    render: render_screen,
    dispatch: dispatch_response,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;
    use unode_plugin_sdk::prelude::route_tabs_view;

    #[test]
    fn manifest_groups_three_routes_as_tabs() {
        let manifest = manifest_envelope().manifest;
        assert!(manifest.validate().is_ok());
        assert_eq!(manifest.routes.len(), 3);
        assert_eq!(manifest.route_groups.len(), 1);

        // Host-side derivation: active comes from the matched route and the
        // ship badge resolves against the state snapshot.
        let state = BTreeMap::from([(SHIP_COUNT.path().to_string(), serde_json::json!(3))]);
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
    fn ship_increment_writes_two_states_and_refreshes() {
        ui::host::clear_recorded_host_calls();
        let response = dispatch_response(&PluginDispatchRequest {
            route: unode_plugin_sdk::prelude::ResolvedRoute {
                pattern: SHIP_PATH.to_string(),
                params: Default::default(),
                query: Default::default(),
            },
            action: ActionRef {
                r#type: ActionType::Custom("ship.increment".to_string()),
                params: None,
                confirm: None,
            },
            state_snapshot: BTreeMap::from([(SHIP_COUNT.path().to_string(), serde_json::json!(5))]),
            locale: None,
        });

        assert!(response.handled);
        assert_eq!(response.message.as_deref(), Some("ship count is now 6"));

        // One action, two state writes.
        let calls = ui::host::recorded_host_calls();
        assert_eq!(calls.len(), 2);
        assert!(calls.iter().all(|call| call.operation == "state.set"));
        assert_eq!(calls[0].params["value"], serde_json::json!(6));
        assert_eq!(calls[1].params["value"], serde_json::json!("shipped #6"));
    }

    #[test]
    fn render_branches_on_route_pattern() {
        let screen = render_screen(&PluginRenderRequest {
            route: unode_plugin_sdk::prelude::ResolvedRoute {
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
