//! End-to-end proof of the web host pipeline, on the native toolchain.
//!
//! This exercises the exact code path the browser `WebSession` runs — only the
//! JSON (de)serialization shim is skipped. If this passes, the reactive patch
//! loop the React adapter re-applies is correct regardless of the wasm build.

use std::collections::BTreeMap;

use serde_json::json;

use unode::core::dsl::{self as ui, expr, IntoNode};
use unode_web_host::WebSessionCore;

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
    assert_eq!(op.f.as_deref(), Some("ct"), "content field");
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

/// Walk an IR JSON tree collecting every node key (`p._k`).
fn collect_keys(node: &serde_json::Value) -> Vec<String> {
    let mut out = Vec::new();
    if let Some(key) = node.get("p").and_then(|p| p.get("_k")).and_then(|k| k.as_str()) {
        out.push(key.to_string());
    }
    if let Some(children) = node.get("c").and_then(|c| c.as_array()) {
        for child in children {
            out.extend(collect_keys(child));
        }
    }
    out
}
