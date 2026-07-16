//! End-to-end proof of the web host pipeline, on the native toolchain.
//!
//! This exercises the exact code path the browser `WebSession` runs — only the
//! JSON (de)serialization shim is skipped. If this passes, the reactive patch
//! loop the React adapter re-applies is correct regardless of the wasm build.

use std::collections::BTreeMap;

use serde_json::json;

use unode::core::ast::{ActionRef, ActionType};
use unode::core::dsl::{self as ui, IntoNode, expr};
use unode::core::runtime::{PluginManifest, SlotContributionDecl};
use unode::core::slot::PluginRenderSlotResponse;
use unode_web_host::{WebSessionCore, session::WebSlotResponseEnvelope};

/// A screen with one reactive line (bound to `ui.count`) and one static line.
fn counter_screen() -> unode::core::ast::ScreenNode {
    ui::screen()
        .id("counter.screen")
        .title("Counter".to_string())
        .children([
            ui::text(expr::binding::<String>("ui.count"))
                .id("count-label")
                .into_node(),
            ui::text("this line never changes".to_string())
                .id("static-line")
                .into_node(),
        ])
        .build()
}

#[test]
fn mounts_screen_and_lowers_to_ir() {
    let mut session = WebSessionCore::new("en");
    let ir = session
        .mount(
            counter_screen(),
            BTreeMap::from([("ui.count".to_string(), json!(0))]),
        )
        .expect("mount");

    assert_eq!(ir.t, "screen");
    // The bound node is present and keyed for patch addressing.
    let json = serde_json::to_value(&ir).expect("ir json");
    let keys = collect_keys(&json);
    assert!(keys.contains(&"count-label".to_string()));
    assert!(keys.contains(&"static-line".to_string()));
}

#[test]
fn state_write_patches_only_the_bound_node() {
    let mut session = WebSessionCore::new("en");
    session
        .mount(
            counter_screen(),
            BTreeMap::from([("ui.count".to_string(), json!(0))]),
        )
        .expect("mount");

    let ops = session
        .apply_writes(BTreeMap::from([("ui.count".to_string(), json!(5))]))
        .expect("apply_writes");

    // Granularity: exactly one node is affected, not the whole screen.
    assert_eq!(ops.len(), 1, "expected a single patch op, got {ops:?}");
    let op = &ops[0];
    assert_eq!(op.o, "sp", "set-prop patch");
    assert_eq!(op.k, "count-label", "targets only the bound node");
    assert_eq!(op.f.as_deref(), Some("content"), "content field");
    // The patched value carries the resolved literal, not the binding.
    assert_eq!(op.v, Some(json!({ "v": "5" })));
}

#[test]
fn state_snapshot_is_flat_after_writes() {
    let mut session = WebSessionCore::new("en");
    session
        .mount(
            counter_screen(),
            BTreeMap::from([("ui.count".to_string(), json!(0))]),
        )
        .expect("mount");

    session
        .apply_writes(BTreeMap::from([("ui.count".to_string(), json!(1))]))
        .expect("first write");
    assert_eq!(session.state_snapshot().get("ui.count"), Some(&json!(1)));

    session
        .apply_writes(BTreeMap::from([("ui.count".to_string(), json!(2))]))
        .expect("second write");
    assert_eq!(session.state_snapshot().get("ui.count"), Some(&json!(2)));

    session
        .apply_writes(BTreeMap::from([("ui.count".to_string(), json!(1))]))
        .expect("third write");
    assert_eq!(session.state_snapshot().get("ui.count"), Some(&json!(1)));
}

#[test]
fn initial_patches_resolve_bindings_against_seed() {
    let mut session = WebSessionCore::new("en");
    session
        .mount(
            counter_screen(),
            BTreeMap::from([("ui.count".to_string(), json!(7))]),
        )
        .expect("mount");

    // The mounted IR keeps `ui.count` symbolic; the initial pass resolves it.
    let ops = session.initial_patches().expect("initial_patches");
    assert_eq!(ops.len(), 1, "one reactive node to resolve, got {ops:?}");
    assert_eq!(ops[0].k, "count-label");
    assert_eq!(ops[0].v, Some(json!({ "v": "7" })));
}

#[test]
fn unrelated_write_produces_no_patches() {
    let mut session = WebSessionCore::new("en");
    session
        .mount(
            counter_screen(),
            BTreeMap::from([("ui.count".to_string(), json!(0))]),
        )
        .expect("mount");

    let ops = session
        .apply_writes(BTreeMap::from([("ui.unrelated".to_string(), json!(true))]))
        .expect("apply_writes");

    assert!(ops.is_empty(), "no node depends on ui.unrelated: {ops:?}");
}

#[test]
fn mount_with_slots_preserves_contributor_action_origin() {
    let mut session = WebSessionCore::new("en");
    let host_screen = ui::screen()
        .id("slot-host.screen")
        .title("Slot host")
        .child(ui::slot("demo.slot").id("slot-host.anchor"))
        .build();
    let contributor_manifest = PluginManifest {
        id: "plugin.b".to_string(),
        name: "Plugin B".to_string(),
        slot_contributions: vec![SlotContributionDecl {
            id: "inject-button".to_string(),
            target: "demo.slot".to_string(),
            priority: 10,
            when: None,
        }],
        ..PluginManifest::default()
    };
    let response = WebSlotResponseEnvelope {
        plugin_id: "plugin.b".to_string(),
        contribution_id: "inject-button".to_string(),
        response: PluginRenderSlotResponse {
            nodes: vec![
                ui::action(
                    "Injected",
                    ActionRef {
                        r#type: ActionType::Custom("plugin-b.approve".to_string()),
                        params: None,
                        confirm: None,
                    },
                )
                .id("approve")
                .into_node(),
            ],
        },
    };

    let ir = session
        .mount_with_slots(
            host_screen,
            BTreeMap::new(),
            vec![contributor_manifest],
            vec![response],
        )
        .expect("mount with slots");
    let json = serde_json::to_value(&ir).expect("ir json");
    let action = find_node_by_type(&json, "action").expect("injected action");

    assert_eq!(
        action.pointer("/p/_originPluginId"),
        Some(&json!("plugin.b")),
        "contributed actions must keep contributor origin"
    );
    assert_eq!(
        action.pointer("/p/_originContributionId"),
        Some(&json!("inject-button"))
    );
}

/// Walk an IR JSON tree collecting every node key (`p._k`).
fn collect_keys(node: &serde_json::Value) -> Vec<String> {
    let mut out = Vec::new();
    if let Some(key) = node
        .get("p")
        .and_then(|p| p.get("_k"))
        .and_then(|k| k.as_str())
    {
        out.push(key.to_string());
    }
    if let Some(children) = node.get("c").and_then(|c| c.as_array()) {
        for child in children {
            out.extend(collect_keys(child));
        }
    }
    out
}

fn find_node_by_type<'a>(
    node: &'a serde_json::Value,
    node_type: &str,
) -> Option<&'a serde_json::Value> {
    if node.get("t").and_then(|t| t.as_str()) == Some(node_type) {
        return Some(node);
    }
    for child in node
        .get("c")
        .and_then(|c| c.as_array())
        .into_iter()
        .flatten()
    {
        if let Some(found) = find_node_by_type(child, node_type) {
            return Some(found);
        }
    }
    None
}
