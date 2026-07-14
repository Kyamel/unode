//! `web-counter` — the reactive plugin behind the web vertical slice.
//!
//! It renders one reactive line bound to `ui.countLabel` plus three actions.
//! On dispatch it reads the current count from the `state_snapshot` the host
//! passed in, computes the next value, and returns the state writes to apply in
//! `PluginDispatchResponse.data.stateWrites`. The host applies them to its
//! store, which produces a single patch op re-rendering only the bound line.
//!
//! State never lives inside the plugin's linear memory — it is owned by the
//! host store and handed back each dispatch. That is the sandbox boundary: the
//! plugin only declares intent and returns data.

use std::cell::Cell;
use std::collections::BTreeMap;

use serde::de::DeserializeOwned;
use serde_json::{json, Value as JsonValue};

use unode_sdk::export_allocators;
use unode_sdk::prelude::{
    self as ui, decode_json_bytes, encode_json_bytes, expr, ActionIntent, ActionRef, ActionType,
    HostCallEnvelope, IntoNode, PluginDispatchOutcome, PluginDispatchRequest,
    PluginDispatchResponse, PluginLoadRequest, PluginManifestEnvelope, PluginRenderRequest,
    ScreenNode, TextRole, Tone, UNODE_PLUGIN_ABI_VERSION,
};

/// The `host_call` boundary — the only way state leaves the plugin sandbox.
///
/// On wasm these are real imports the host provides (module `unode`, matching
/// `crates/unode-tui-runtime` and `pluginHost.ts`). On the host toolchain they
/// are stubs that record the calls so the dispatch logic stays unit-testable.
#[cfg(target_arch = "wasm32")]
mod host_import {
    #[link(wasm_import_module = "unode")]
    unsafe extern "C" {
        pub fn host_call(ptr: u32, len: u32) -> u32;
        pub fn host_call_result_len() -> u32;
    }
}

#[cfg(not(target_arch = "wasm32"))]
mod host_import {
    use std::cell::RefCell;

    use unode_sdk::prelude::{decode_json_bytes, HostCallEnvelope};

    thread_local! {
        /// Host calls captured during a native test run.
        pub static RECORDED: RefCell<Vec<HostCallEnvelope>> = const { RefCell::new(Vec::new()) };
    }

    /// Native stand-in for the `host_call` boundary: decode and record the
    /// envelope. No pointer round-trip (native pointers are 64-bit).
    pub fn record(bytes: &[u8]) {
        if let Ok(env) = decode_json_bytes::<HostCallEnvelope>(bytes) {
            RECORDED.with(|recorded| recorded.borrow_mut().push(env));
        }
    }
}

/// Request a single state write through the sandbox boundary. The host owns the
/// store; the plugin only expresses intent.
fn host_state_set(path: &str, value: JsonValue) {
    let envelope = HostCallEnvelope {
        operation: "state.set".to_string(),
        params: [("path".to_string(), json!(path)), ("value".to_string(), value)]
            .into_iter()
            .collect(),
    };
    let bytes = encode_json_bytes(&envelope).expect("encode host call");

    #[cfg(target_arch = "wasm32")]
    unsafe {
        let ptr = unode_alloc(bytes.len()) as u32;
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), ptr as *mut u8, bytes.len());
        // The host writes a response we don't need for `state.set`; draining the
        // length keeps the ABI contract symmetric with other host calls.
        let _response_ptr = host_import::host_call(ptr, bytes.len() as u32);
        let _response_len = host_import::host_call_result_len();
    }

    #[cfg(not(target_arch = "wasm32"))]
    host_import::record(&bytes);
}

const PLUGIN_ID: &str = "dev.unode.web-counter";
const PLUGIN_NAME: &str = "Web Counter";
#[cfg_attr(not(test), allow(dead_code))]
const ROUTE_PATH: &str = "/plugins/web-counter";
const COUNT_PATH: &str = "ui.count";
const LABEL_PATH: &str = "ui.countLabel";

static ABI_VERSION_BYTES: &[u8] = b"0.1.0\0";

thread_local! {
    static MANIFEST_BUFFER_LEN: Cell<u32> = const { Cell::new(0) };
    static LOAD_BUFFER_LEN: Cell<u32> = const { Cell::new(0) };
    static RENDER_BUFFER_LEN: Cell<u32> = const { Cell::new(0) };
    static DISPATCH_BUFFER_LEN: Cell<u32> = const { Cell::new(0) };
}

export_allocators!();

fn with_output_buffer<F>(len_cell: &'static std::thread::LocalKey<Cell<u32>>, build: F) -> u32
where
    F: FnOnce() -> Vec<u8>,
{
    len_cell.with(|slot| {
        let bytes = build();
        let len = bytes.len() as u32;
        let ptr = unode_alloc(bytes.len()) as u32;
        unsafe {
            std::ptr::copy_nonoverlapping(bytes.as_ptr(), ptr as *mut u8, bytes.len());
        }
        slot.set(len);
        ptr
    })
}

fn output_len(len_cell: &'static std::thread::LocalKey<Cell<u32>>) -> u32 {
    len_cell.with(Cell::get)
}

fn decode_guest_json<T: DeserializeOwned>(ptr: u32, len: u32) -> T {
    let bytes = unsafe { std::slice::from_raw_parts(ptr as *const u8, len as usize) };
    decode_json_bytes(bytes).expect("guest request must be valid ABI JSON")
}

fn json_or_error<T: serde::Serialize>(value: &T) -> Vec<u8> {
    encode_json_bytes(value).unwrap_or_else(|err| {
        serde_json::to_vec(&json!({ "error": err.to_string() })).expect("fallback json")
    })
}

fn manifest_envelope() -> PluginManifestEnvelope {
    PluginManifestEnvelope {
        abi_version: UNODE_PLUGIN_ABI_VERSION.to_string(),
        manifest: unode_sdk::plugin_manifest(PLUGIN_ID, PLUGIN_NAME)
            .version("0.1.0")
            .description("Reactive counter proving the unode web runtime slice.")
            .author("unode")
            .build(),
    }
}

fn label_for(count: i64) -> String {
    format!("Count: {count}")
}

/// A plain Custom action with no params.
fn custom(action: &str) -> ActionRef {
    ActionRef {
        r#type: ActionType::Custom(action.to_string()),
        params: None,
        confirm: None,
    }
}

fn render_screen(_request: &PluginRenderRequest) -> ScreenNode {
    ui::screen()
        .id("web-counter.screen")
        .title(format!("{PLUGIN_NAME}"))
        .subtitle("Rendered from a Rust plugin compiled to WebAssembly.")
        .initial_state(BTreeMap::from([
            (COUNT_PATH.to_string(), json!(0)),
            (LABEL_PATH.to_string(), json!(label_for(0))),
        ]))
        .children([
            // The one reactive node: its content is a binding, so the host
            // tracks it and patches only this line when `ui.countLabel` changes.
            ui::text(expr::binding::<String>(LABEL_PATH))
                .id("web-counter.value")
                .role(TextRole::Title)
                .tone(Tone::Info)
                .into_node(),
            ui::text("The number above is host state; the buttons dispatch intents.")
                .id("web-counter.hint")
                .role(TextRole::Caption)
                .tone(Tone::Muted)
                .into_node(),
            ui::actions()
                .id("web-counter.actions")
                .children([
                    ui::action("Increment", custom("counter.inc"))
                        .id("web-counter.inc")
                        .intent(ActionIntent::Primary),
                    ui::action("Decrement", custom("counter.dec"))
                        .id("web-counter.dec")
                        .intent(ActionIntent::Secondary),
                    ui::action("Reset", custom("counter.reset"))
                        .id("web-counter.reset")
                        .intent(ActionIntent::Ghost),
                ])
                .into_node(),
        ])
        .initial_focus("web-counter.inc")
        .build()
}

fn current_count(request: &PluginDispatchRequest) -> i64 {
    request
        .state_snapshot
        .get(COUNT_PATH)
        .and_then(JsonValue::as_i64)
        .unwrap_or(0)
}

/// Compute the next count for a known action, or `None` if unhandled.
fn next_count(request: &PluginDispatchRequest) -> Option<i64> {
    match &request.action.r#type {
        ActionType::Custom(a) if a == "counter.inc" => Some(current_count(request) + 1),
        ActionType::Custom(a) if a == "counter.dec" => Some(current_count(request) - 1),
        ActionType::Custom(a) if a == "counter.reset" => Some(0),
        _ => None,
    }
}

fn dispatch_response(request: &PluginDispatchRequest) -> PluginDispatchResponse {
    match next_count(request) {
        Some(count) => {
            // Writes cross the sandbox boundary as capability calls — the plugin
            // never returns UI state in its response.
            host_state_set(COUNT_PATH, json!(count));
            host_state_set(LABEL_PATH, json!(label_for(count)));
            PluginDispatchResponse {
                handled: true,
                outcome: PluginDispatchOutcome::None,
                message: Some(format!("count -> {count}")),
                data: None,
            }
        }
        None => PluginDispatchResponse {
            handled: false,
            outcome: PluginDispatchOutcome::None,
            message: Some("web-counter ignored action".to_string()),
            data: None,
        },
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn plugin_abi_version() -> *const u8 {
    ABI_VERSION_BYTES.as_ptr()
}

#[unsafe(no_mangle)]
pub extern "C" fn plugin_manifest() -> u32 {
    with_output_buffer(&MANIFEST_BUFFER_LEN, || json_or_error(&manifest_envelope()))
}

#[unsafe(no_mangle)]
pub extern "C" fn plugin_manifest_len() -> u32 {
    output_len(&MANIFEST_BUFFER_LEN)
}

#[unsafe(no_mangle)]
pub extern "C" fn plugin_load(request_ptr: u32, request_len: u32) -> u32 {
    let request = decode_guest_json::<PluginLoadRequest>(request_ptr, request_len);
    with_output_buffer(&LOAD_BUFFER_LEN, || {
        json_or_error(&json!({
            "loaded": true,
            "pluginId": PLUGIN_ID,
            "route": request.route.pattern,
        }))
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn plugin_load_result_len() -> u32 {
    output_len(&LOAD_BUFFER_LEN)
}

#[unsafe(no_mangle)]
pub extern "C" fn plugin_render(request_ptr: u32, request_len: u32) -> u32 {
    let request = decode_guest_json::<PluginRenderRequest>(request_ptr, request_len);
    with_output_buffer(&RENDER_BUFFER_LEN, || json_or_error(&render_screen(&request)))
}

#[unsafe(no_mangle)]
pub extern "C" fn plugin_render_result_len() -> u32 {
    output_len(&RENDER_BUFFER_LEN)
}

#[unsafe(no_mangle)]
pub extern "C" fn plugin_dispatch(request_ptr: u32, request_len: u32) -> u32 {
    let request = decode_guest_json::<PluginDispatchRequest>(request_ptr, request_len);
    with_output_buffer(&DISPATCH_BUFFER_LEN, || json_or_error(&dispatch_response(&request)))
}

#[unsafe(no_mangle)]
pub extern "C" fn plugin_dispatch_result_len() -> u32 {
    output_len(&DISPATCH_BUFFER_LEN)
}

#[cfg(test)]
mod tests {
    use super::*;
    use unode_sdk::prelude::ResolvedRoute;

    fn dispatch_req(action: &str, count: i64) -> PluginDispatchRequest {
        PluginDispatchRequest {
            route: ResolvedRoute {
                pattern: ROUTE_PATH.to_string(),
                params: BTreeMap::new(),
                query: BTreeMap::new(),
            },
            action: custom(action),
            state_snapshot: BTreeMap::from([(COUNT_PATH.to_string(), json!(count))]),
            locale: Some("en".to_string()),
        }
    }

    #[test]
    fn manifest_identity() {
        assert_eq!(manifest_envelope().manifest.id, PLUGIN_ID);
    }

    fn recorded_state_sets() -> Vec<(String, JsonValue)> {
        host_import::RECORDED.with(|recorded| {
            recorded
                .borrow()
                .iter()
                .filter(|env| env.operation == "state.set")
                .map(|env| {
                    (
                        env.params["path"].as_str().unwrap_or_default().to_string(),
                        env.params["value"].clone(),
                    )
                })
                .collect()
        })
    }

    fn clear_recorded() {
        host_import::RECORDED.with(|recorded| recorded.borrow_mut().clear());
    }

    #[test]
    fn increment_issues_state_set_host_calls() {
        clear_recorded();
        let resp = dispatch_response(&dispatch_req("counter.inc", 4));
        assert!(resp.handled);
        assert!(resp.data.is_none(), "no UI state returned in the response");

        let sets = recorded_state_sets();
        assert!(sets.contains(&(COUNT_PATH.to_string(), json!(5))), "sets: {sets:?}");
        assert!(sets.contains(&(LABEL_PATH.to_string(), json!("Count: 5"))), "sets: {sets:?}");
    }

    #[test]
    fn reset_sets_zero() {
        clear_recorded();
        dispatch_response(&dispatch_req("counter.reset", 99));
        let sets = recorded_state_sets();
        assert!(sets.contains(&(COUNT_PATH.to_string(), json!(0))), "sets: {sets:?}");
    }

    #[test]
    fn unknown_action_is_unhandled_and_writes_nothing() {
        clear_recorded();
        let resp = dispatch_response(&dispatch_req("counter.spin", 1));
        assert!(!resp.handled);
        assert!(recorded_state_sets().is_empty());
    }
}
