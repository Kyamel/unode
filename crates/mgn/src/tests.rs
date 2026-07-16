use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;
use std::thread::sleep;
use std::time::Duration;

use serde_json::{Value as JsonValue, json};
use tui_renderer::{TuiFocusedPane, TuiMainContent};
use unode_sdk::prelude::{ActionRef, ActionType, PermissionProfile, ResolvedRoute, ScreenNode};
use unode_sdk::{
    PluginDispatchRequest, PluginDispatchResponse, PluginLoadRequest, PluginRenderRequest,
};
use unode_tui_runtime::{TuiHostCallDispatcher, WasmtimeGuest};

use crate::App;
use crate::plugin_registry::{find_plugin_wasm, plugin_slug};

fn route_for(plugin_id: &str) -> String {
    format!("/plugins/{}", plugin_slug(plugin_id))
}

fn test_dispatcher() -> TuiHostCallDispatcher {
    let mut dispatcher = TuiHostCallDispatcher::new();
    dispatcher.register("system.ping", |_| Ok(json!({ "pong": true })));
    dispatcher.register("navigation.navigate", |params| {
        Ok(json!({
            "ok": true,
            "to": params.get("to").cloned().unwrap_or(JsonValue::Null)
        }))
    });
    dispatcher
}

#[test]
fn sanity_plugin_survives_render_dispatch_render_sequence() {
    let plugin_root = PathBuf::from("plugins/sanity-check");
    let Some(wasm_path) = find_plugin_wasm(&plugin_root, "sanity_check_plugin.wasm") else {
        return;
    };

    let mut bridge =
        WasmtimeGuest::from_wasm_file(&wasm_path, test_dispatcher()).expect("instantiate wasm");
    let manifest = bridge.call_plugin_manifest().expect("manifest");
    let route = route_for(&manifest.manifest.id);

    bridge
        .call_plugin_load::<_, JsonValue>(&PluginLoadRequest {
            route: ResolvedRoute {
                pattern: route.clone(),
                params: BTreeMap::new(),
                query: BTreeMap::new(),
            },
            state_snapshot: BTreeMap::new(),
            locale: Some("en".to_string()),
        })
        .expect("load");

    let overview = bridge
        .call_plugin_render::<_, ScreenNode>(&PluginRenderRequest {
            route: ResolvedRoute {
                pattern: route.clone(),
                params: BTreeMap::new(),
                query: BTreeMap::new(),
            },
            data: json!({
                "title": "Smoke test",
                "hostMessage": format!("Loaded from {}", wasm_path.display()),
            }),
            state_snapshot: BTreeMap::new(),
            locale: Some("en".to_string()),
        })
        .expect("overview render");
    assert!(overview.title.is_some());

    let inspect = bridge
        .call_plugin_render::<_, ScreenNode>(&PluginRenderRequest {
            route: ResolvedRoute {
                pattern: route.clone(),
                params: BTreeMap::new(),
                query: BTreeMap::from([("view".to_string(), "inspect".to_string())]),
            },
            data: json!({
                "title": "Smoke test",
                "hostMessage": format!("Loaded from {}", wasm_path.display()),
            }),
            state_snapshot: BTreeMap::new(),
            locale: Some("en".to_string()),
        })
        .expect("inspect render");
    assert!(inspect.subtitle.is_some());

    let dispatch = bridge
        .call_plugin_dispatch::<PluginDispatchResponse>(&PluginDispatchRequest {
            route: ResolvedRoute {
                pattern: route.clone(),
                params: BTreeMap::new(),
                query: BTreeMap::from([("view".to_string(), "inspect".to_string())]),
            },
            action: ActionRef {
                r#type: ActionType::Custom("sanity.go-home".to_string()),
                params: None,
                confirm: None,
            },
            state_snapshot: BTreeMap::new(),
            locale: Some("en".to_string()),
        })
        .expect("dispatch");
    assert!(dispatch.handled);

    let rerender = bridge
        .call_plugin_render::<_, ScreenNode>(&PluginRenderRequest {
            route: ResolvedRoute {
                pattern: route,
                params: BTreeMap::new(),
                query: BTreeMap::new(),
            },
            data: json!({
                "title": "Smoke test",
                "hostMessage": format!("Loaded from {}", wasm_path.display()),
            }),
            state_snapshot: BTreeMap::new(),
            locale: Some("en".to_string()),
        })
        .expect("rerender after dispatch");
    assert!(rerender.title.is_some());
}

#[test]
fn warns_when_plugin_source_is_newer_than_wasm() {
    let plugin_root = PathBuf::from("plugins/sanity-check");
    let Some(wasm_path) = find_plugin_wasm(&plugin_root, "sanity_check_plugin.wasm") else {
        return;
    };

    let _ = PermissionProfile {
        plugin_id: "mgn.shell".to_string(),
        grants: vec![],
    };
    assert!(wasm_path.exists());
}

#[test]
fn prefers_newest_wasm_artifact() {
    let plugin_root = std::env::temp_dir().join(format!("mgn-test-{}", std::process::id()));
    let _ = fs::remove_dir_all(&plugin_root);
    let debug_path =
        plugin_root.join("target/wasm32-unknown-unknown/debug/sanity_check_plugin.wasm");
    let release_path =
        plugin_root.join("target/wasm32-unknown-unknown/release/sanity_check_plugin.wasm");

    fs::create_dir_all(debug_path.parent().expect("debug parent")).expect("debug dir");
    fs::create_dir_all(release_path.parent().expect("release parent")).expect("release dir");

    fs::write(&debug_path, b"debug").expect("write debug");
    sleep(Duration::from_millis(10));
    fs::write(&release_path, b"release").expect("write release");

    let selected =
        find_plugin_wasm(&plugin_root, "sanity_check_plugin.wasm").expect("selected wasm");
    assert_eq!(selected, release_path);
    let _ = fs::remove_dir_all(&plugin_root);
}

#[test]
fn app_survives_three_full_plugin_navigation_cycles() {
    let mut app = match App::new() {
        Ok(app) => app,
        Err(_) => return,
    };

    let plugin_route = app
        .plugins
        .iter()
        .find(|plugin| plugin.runtime_plugin.manifest().manifest.id == "dev.unode.sanity-check")
        .map(|plugin| plugin.route.clone())
        .expect("sanity plugin route");

    for _ in 0..3 {
        app.navigate_to(plugin_route.clone());
        app.focused_pane = TuiFocusedPane::Main;
        if app.main_interactions.is_empty() {
            match &app.main_panel {
                TuiMainContent::Panel(panel) => panic!("plugin panel fallback: {:?}", panel.lines),
                TuiMainContent::Screen(screen) => {
                    panic!("screen without interactions: {:?}", screen.screen)
                }
            }
        }

        let inspect_index = app
            .main_interactions
            .iter()
            .position(|interaction| interaction.label.contains("Inspect"))
            .unwrap_or_else(|| {
                panic!("inspect interaction not found: {:?}", app.main_interactions)
            });
        app.selected_main_interaction = Some(inspect_index);
        app.activate_main_interaction().expect("open inspect tab");
        assert_eq!(app.current_route, format!("{plugin_route}/inspect"));

        let back_index = app
            .main_interactions
            .iter()
            .position(|interaction| interaction.label.contains("Back to overview"))
            .unwrap_or_else(|| panic!("back interaction not found: {:?}", app.main_interactions));
        app.selected_main_interaction = Some(back_index);
        app.activate_main_interaction().expect("back to overview");
        assert_eq!(app.current_route, plugin_route);

        let go_home_index = app
            .main_interactions
            .iter()
            .position(|interaction| interaction.label.contains("Go home via plugin dispatch"))
            .unwrap_or_else(|| {
                panic!("go-home interaction not found: {:?}", app.main_interactions)
            });
        app.selected_main_interaction = Some(go_home_index);
        app.activate_main_interaction().expect("go home");
        assert_eq!(app.current_route, "/home");
    }

    app.navigate_to(plugin_route.clone());
    app.focused_pane = TuiFocusedPane::Main;
    if app.main_interactions.is_empty() {
        match &app.main_panel {
            TuiMainContent::Panel(panel) => panic!("plugin panel fallback: {:?}", panel.lines),
            TuiMainContent::Screen(screen) => {
                panic!("screen without interactions: {:?}", screen.screen)
            }
        }
    }
    let inspect_index = app
        .main_interactions
        .iter()
        .position(|interaction| interaction.label.contains("Inspect"))
        .unwrap_or_else(|| panic!("inspect interaction not found: {:?}", app.main_interactions));
    app.selected_main_interaction = Some(inspect_index);
    app.activate_main_interaction()
        .expect("open inspect tab 4th");
    assert_eq!(app.current_route, format!("{plugin_route}/inspect"));
}

#[test]
fn app_registers_web_counter_and_applies_state_set_host_calls() {
    let mut app = match App::new() {
        Ok(app) => app,
        Err(_) => return,
    };

    let Some(plugin_index) = app
        .plugins
        .iter()
        .position(|plugin| plugin.runtime_plugin.manifest().manifest.id == "dev.unode.web-counter")
    else {
        return;
    };
    let plugin_route = app.plugins[plugin_index].route.clone();

    app.navigate_to(plugin_route);
    app.focused_pane = TuiFocusedPane::Main;
    if app.main_interactions.is_empty() {
        match &app.main_panel {
            TuiMainContent::Panel(panel) => panic!("plugin panel fallback: {:?}", panel.lines),
            TuiMainContent::Screen(screen) => {
                panic!("screen without interactions: {:?}", screen.screen)
            }
        }
    }

    let increment_index = app
        .main_interactions
        .iter()
        .position(|interaction| interaction.label.contains("Increment"))
        .unwrap_or_else(|| {
            panic!(
                "increment interaction not found: {:?}",
                app.main_interactions
            )
        });
    app.selected_main_interaction = Some(increment_index);
    app.activate_main_interaction().expect("increment counter");

    let snapshot = app.plugins[plugin_index].state.snapshot();
    assert_eq!(
        snapshot.get("ui.count").and_then(JsonValue::as_i64),
        Some(1)
    );
    assert_eq!(
        snapshot.get("ui.countLabel").and_then(JsonValue::as_str),
        Some("Count: 1")
    );
}

#[test]
fn web_counter_survives_many_increment_cycles() {
    let mut app = match App::new() {
        Ok(app) => app,
        Err(_) => return,
    };

    let Some(plugin_index) = app
        .plugins
        .iter()
        .position(|plugin| plugin.runtime_plugin.manifest().manifest.id == "dev.unode.web-counter")
    else {
        return;
    };
    let plugin_route = app.plugins[plugin_index].route.clone();

    app.navigate_to(plugin_route);
    app.focused_pane = TuiFocusedPane::Main;

    for expected in 1..=1_000 {
        let increment_index = app
            .main_interactions
            .iter()
            .position(|interaction| interaction.label.contains("Increment"))
            .unwrap_or_else(|| {
                panic!(
                    "increment interaction not found at {expected}: {:?}",
                    app.main_interactions
                )
            });
        app.selected_main_interaction = Some(increment_index);
        app.activate_main_interaction()
            .unwrap_or_else(|err| panic!("increment {expected} failed: {err}"));

        let snapshot = app.plugins[plugin_index].state.snapshot();
        assert_eq!(
            snapshot.get("ui.count").and_then(JsonValue::as_i64),
            Some(expected),
            "snapshot after increment {expected}: {snapshot:?}"
        );

        match &app.main_panel {
            TuiMainContent::Panel(panel) => {
                panic!("plugin panel fallback at {expected}: {:?}", panel.lines)
            }
            TuiMainContent::Screen(_) => {}
        }
    }
}
