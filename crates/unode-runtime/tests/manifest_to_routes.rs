//! Integration test: the full native path from plugin authoring to host
//! routing and navigation chrome — manifest builder → envelope → route
//! registration → route resolution → route-tabs derivation.

use std::collections::BTreeMap;

use serde_json::json;
use unode::core::chrome::route_tabs_view;
use unode_plugin_sdk::prelude::{StateKey, perm, plugin_manifest, route, route_group};
use unode_runtime::RouteRegistry;

const UNREAD: StateKey<u32> = StateKey::new("notes.unread");

fn manifest() -> unode_plugin_sdk::PluginManifestEnvelope {
    plugin_manifest("demo.notes", "Notes")
        .version("1.0.0")
        .permission(perm("state.write").required(true).reason("Persist notes"))
        .route_group(route_group("main").tabs())
        .routes([
            route("/notes").group("main").label("Notes"),
            route("/notes/archive")
                .group("main")
                .label("Archive")
                .badge_bind(UNREAD.path()),
            route("/notes/:id").screen_kind("demo.notes.detail"),
        ])
        .envelope()
}

#[test]
fn manifest_flows_from_authoring_to_host_routing() {
    let envelope = manifest();

    // Envelope carries the current ABI version without manual wiring.
    assert_eq!(
        envelope.abi_version,
        unode_plugin_sdk::UNODE_PLUGIN_ABI_VERSION
    );
    let manifest = envelope.manifest;
    assert!(manifest.validate().is_ok());

    // Host registers the declared routes and resolves navigations.
    let mut routes = RouteRegistry::default();
    routes.register_manifest_routes(&manifest, 500);

    let list = routes.resolve("/notes").expect("list route");
    assert_eq!(list.plugin_id, "demo.notes");
    assert_eq!(list.pattern, "/notes");

    let detail = routes.resolve("/notes/42").expect("detail route");
    assert_eq!(detail.screen_kind, "demo.notes.detail");
    assert_eq!(detail.params.get("id").map(String::as_str), Some("42"));

    // Navigation chrome derives from the matched route + state snapshot.
    let state = BTreeMap::from([(UNREAD.path().to_string(), json!(2))]);
    let tabs = route_tabs_view(&manifest, &list.pattern, &state).expect("tabs");
    assert_eq!(tabs.active, "/notes");
    assert_eq!(tabs.tabs.len(), 2); // the `:id` route is not grouped
    assert_eq!(tabs.tabs[1].badge.as_deref(), Some("2"));

    // The detail route is standalone: no tabs.
    assert!(route_tabs_view(&manifest, &detail.pattern, &state).is_none());
}
