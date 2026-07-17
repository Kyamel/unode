//! Golden test for the two plugin boundaries: the same plugin source
//! (`plugins/counter`) compiled through the raw ptr/len ABI and through the
//! Component Model (WIT) must produce identical JSON and identical host-call
//! side effects.
//!
//! Artifacts (skipped when missing):
//! - raw:       `cargo build -p web-counter-plugin --target wasm32-unknown-unknown --release`
//! - component: same build with `--features component`, then
//!              `wasm-tools component new <core.wasm> -o <...>.component.wasm`

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use serde_json::{Value as JsonValue, json};
use unode_plugin_sdk::prelude::{ActionRef, ActionType, ResolvedRoute};
use unode_plugin_sdk::{PluginDispatchRequest, PluginDispatchResponse, PluginRenderRequest};
use unode_tui_runtime::{ComponentTuiPlugin, TuiHostCallDispatcher, WasmtimeGuest};

type Writes = Arc<Mutex<Vec<(String, JsonValue)>>>;

fn recording_dispatcher() -> (TuiHostCallDispatcher, Writes) {
    let writes: Writes = Arc::new(Mutex::new(Vec::new()));
    let sink = writes.clone();
    let mut dispatcher = TuiHostCallDispatcher::new();
    dispatcher.register("state.set", move |params| {
        let path = params
            .get("path")
            .and_then(JsonValue::as_str)
            .unwrap_or_default()
            .to_string();
        let value = params.get("value").cloned().unwrap_or(JsonValue::Null);
        sink.lock().expect("writes lock").push((path, value));
        Ok(json!({ "ok": true }))
    });
    (dispatcher, writes)
}

fn artifact(name: &str) -> Option<PathBuf> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../plugins/counter/target/wasm32-unknown-unknown/release")
        .join(name);
    path.exists().then_some(path)
}

fn route() -> ResolvedRoute {
    ResolvedRoute {
        pattern: "/plugins/counter".to_string(),
        params: BTreeMap::new(),
        query: BTreeMap::new(),
    }
}

#[test]
fn raw_abi_and_component_produce_identical_json() {
    let (Some(raw_path), Some(component_path)) = (
        artifact("web_counter_plugin.wasm"),
        artifact("web_counter_plugin.component.wasm"),
    ) else {
        return;
    };

    // --- load both boundaries ---
    let (raw_dispatcher, raw_writes) = recording_dispatcher();
    let mut raw = WasmtimeGuest::from_wasm_file(&raw_path, raw_dispatcher).expect("raw plugin");
    let raw_manifest = raw.call_plugin_manifest().expect("raw manifest");

    let (component_dispatcher, component_writes) = recording_dispatcher();
    let mut component = ComponentTuiPlugin::from_wasm_file(&component_path, component_dispatcher)
        .expect("component plugin");

    // --- manifest: identical JSON ---
    assert_eq!(
        serde_json::to_value(&raw_manifest).expect("raw manifest json"),
        serde_json::to_value(component.manifest()).expect("component manifest json"),
        "manifest must be identical across boundaries"
    );

    // --- render: identical ScreenNode JSON ---
    let render_request = PluginRenderRequest {
        route: route(),
        data: json!({}),
        state_snapshot: BTreeMap::from([("ui.count".to_string(), json!(4))]),
        locale: Some("en".to_string()),
    };
    let raw_screen: JsonValue = raw.call_plugin_render(&render_request).expect("raw render");
    let component_screen = component.render(&render_request).expect("component render");
    assert_eq!(
        raw_screen, component_screen,
        "render must be identical across boundaries"
    );

    // --- dispatch: identical response and identical state writes ---
    let dispatch_request = PluginDispatchRequest {
        route: route(),
        action: ActionRef {
            r#type: ActionType::Custom("counter.inc".to_string()),
            params: None,
            confirm: None,
        },
        state_snapshot: BTreeMap::from([("ui.count".to_string(), json!(4))]),
        locale: Some("en".to_string()),
    };
    let raw_response: PluginDispatchResponse = raw
        .call_plugin_dispatch(&dispatch_request)
        .expect("raw dispatch");
    let component_response = component
        .dispatch(&dispatch_request)
        .expect("component dispatch");
    assert_eq!(
        serde_json::to_value(&raw_response).expect("raw response json"),
        serde_json::to_value(&component_response).expect("component response json"),
        "dispatch response must be identical across boundaries"
    );

    let raw_writes = raw_writes.lock().expect("raw writes").clone();
    let component_writes = component_writes.lock().expect("component writes").clone();
    assert_eq!(
        raw_writes, component_writes,
        "host-call side effects must be identical across boundaries"
    );
    assert!(
        raw_writes.contains(&("ui.count".to_string(), json!(5))),
        "increment reached the host: {raw_writes:?}"
    );
}
