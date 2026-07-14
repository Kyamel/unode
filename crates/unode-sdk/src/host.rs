use std::collections::BTreeMap;

use serde_json::{Value as JsonValue, json};

use crate::abi::{AbiError, HostCallEnvelope, encode_json_bytes};

/// Sends a host-call envelope through the sandbox boundary.
///
/// On `wasm32` this calls the raw `unode.host_call` import expected by the host
/// runtimes. On native targets it records the envelope so plugin dispatch logic
/// can be unit-tested without a WASM runtime.
pub fn call_host(envelope: HostCallEnvelope) {
    try_call_host(envelope).expect("encode host call");
}

pub fn try_call_host(envelope: HostCallEnvelope) -> Result<(), AbiError> {
    let bytes = encode_json_bytes(&envelope)?;

    #[cfg(target_arch = "wasm32")]
    wasm::send(&bytes);

    #[cfg(not(target_arch = "wasm32"))]
    drop(bytes);

    #[cfg(not(target_arch = "wasm32"))]
    native::record(envelope);

    Ok(())
}

/// Requests a single state write from the host-owned state store.
///
/// Plugins do not mutate host state directly. They declare the write intent and
/// the host applies it after capability and runtime checks.
pub fn state_set(path: &str, value: JsonValue) {
    try_state_set(path, value).expect("encode host state.set call");
}

pub fn try_state_set(path: &str, value: JsonValue) -> Result<(), AbiError> {
    try_call_host(state_set_envelope(path, value))
}

fn state_set_envelope(path: &str, value: JsonValue) -> HostCallEnvelope {
    HostCallEnvelope {
        operation: "state.set".to_string(),
        params: BTreeMap::from([
            ("path".to_string(), json!(path)),
            ("value".to_string(), value),
        ]),
    }
}

#[cfg(target_arch = "wasm32")]
mod wasm {
    #[link(wasm_import_module = "unode")]
    unsafe extern "C" {
        fn host_call(ptr: u32, len: u32) -> u32;
        fn host_call_result_len() -> u32;
    }

    pub(super) fn send(bytes: &[u8]) {
        let len = u32::try_from(bytes.len()).expect("host call payload exceeds u32 length");
        unsafe {
            let _response_ptr = host_call(bytes.as_ptr() as u32, len);
            let _response_len = host_call_result_len();
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
mod native {
    use std::cell::RefCell;

    use crate::HostCallEnvelope;

    thread_local! {
        static RECORDED: RefCell<Vec<HostCallEnvelope>> = const { RefCell::new(Vec::new()) };
    }

    pub(super) fn record(envelope: HostCallEnvelope) {
        RECORDED.with(|recorded| recorded.borrow_mut().push(envelope));
    }

    pub fn recorded_host_calls() -> Vec<HostCallEnvelope> {
        RECORDED.with(|recorded| recorded.borrow().clone())
    }

    pub fn clear_recorded_host_calls() {
        RECORDED.with(|recorded| recorded.borrow_mut().clear());
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub use native::{clear_recorded_host_calls, recorded_host_calls};

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{clear_recorded_host_calls, recorded_host_calls, state_set};

    #[test]
    fn native_state_set_records_host_call() {
        clear_recorded_host_calls();

        state_set("ui.count", json!(5));

        let calls = recorded_host_calls();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].operation, "state.set");
        assert_eq!(calls[0].params.get("path"), Some(&json!("ui.count")));
        assert_eq!(calls[0].params.get("value"), Some(&json!(5)));
    }
}
