use std::collections::BTreeMap;
use std::sync::Arc;

use serde_json::Value as JsonValue;
use thiserror::Error;
use unode_sdk::abi::AbiError;
use unode_sdk::{
    HostCallEnvelope, IMPORT_HOST_CALL, IMPORT_HOST_CALL_RESULT_LEN, decode_json_bytes,
    encode_json_bytes,
};

use crate::memory::{WebMemoryError, read_bytes};

type HostCallHandler =
    Arc<dyn Fn(&BTreeMap<String, JsonValue>) -> Result<JsonValue, WebHostCallError> + Send + Sync>;

#[derive(Debug, Error)]
pub enum WebHostCallError {
    #[error("unknown host operation `{0}`")]
    UnknownOperation(String),
    #[error("invalid host call payload")]
    InvalidPayload(#[from] AbiError),
    #[error(transparent)]
    Memory(#[from] WebMemoryError),
    #[error("handler error for `{operation}`: {message}")]
    Handler { operation: String, message: String },
}

#[derive(Default)]
pub struct WebHostCallDispatcher {
    handlers: BTreeMap<String, HostCallHandler>,
    last_response: Vec<u8>,
}

impl std::fmt::Debug for WebHostCallDispatcher {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WebHostCallDispatcher")
            .field("handler_count", &self.handlers.len())
            .field("last_response_len", &self.last_response.len())
            .finish()
    }
}

impl WebHostCallDispatcher {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn import_name() -> &'static str {
        IMPORT_HOST_CALL
    }

    pub fn result_len_import_name() -> &'static str {
        IMPORT_HOST_CALL_RESULT_LEN
    }

    pub fn register<F>(&mut self, operation: impl Into<String>, handler: F)
    where
        F: Fn(&BTreeMap<String, JsonValue>) -> Result<JsonValue, WebHostCallError>
            + Send
            + Sync
            + 'static,
    {
        self.handlers.insert(operation.into(), Arc::new(handler));
    }

    pub fn dispatch_envelope(
        &mut self,
        envelope: &HostCallEnvelope,
    ) -> Result<&[u8], WebHostCallError> {
        let handler = self
            .handlers
            .get(&envelope.operation)
            .ok_or_else(|| WebHostCallError::UnknownOperation(envelope.operation.clone()))?;

        let result = handler(&envelope.params).map_err(|err| match err {
            WebHostCallError::Handler { .. } => err,
            other => WebHostCallError::Handler {
                operation: envelope.operation.clone(),
                message: other.to_string(),
            },
        })?;

        self.last_response = encode_json_bytes(&result)?;
        Ok(self.last_response.as_slice())
    }

    pub fn dispatch_bytes(&mut self, request_bytes: &[u8]) -> Result<&[u8], WebHostCallError> {
        let envelope = decode_json_bytes::<HostCallEnvelope>(request_bytes)?;
        self.dispatch_envelope(&envelope)
    }

    pub fn dispatch_from_memory(
        &mut self,
        memory: &[u8],
        ptr: u32,
        len: u32,
    ) -> Result<&[u8], WebHostCallError> {
        let request_bytes = read_bytes(memory, ptr, len)?;
        self.dispatch_bytes(&request_bytes)
    }

    pub fn last_response(&self) -> &[u8] {
        self.last_response.as_slice()
    }

    pub fn last_response_len(&self) -> usize {
        self.last_response.len()
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use serde_json::json;
    use unode_sdk::{HostCallEnvelope, decode_json_bytes};

    use super::{WebHostCallDispatcher, WebHostCallError};
    use crate::memory::write_bytes;

    #[test]
    fn dispatches_registered_operation_and_stores_response() {
        let mut dispatcher = WebHostCallDispatcher::new();
        dispatcher.register("system.ping", |_| Ok(json!({ "pong": true })));

        let response = dispatcher
            .dispatch_envelope(&HostCallEnvelope {
                operation: "system.ping".to_string(),
                params: BTreeMap::new(),
            })
            .expect("dispatch");
        let response_len = response.len();

        let value = decode_json_bytes::<serde_json::Value>(response).expect("decode");
        assert_eq!(value["pong"], true);
        assert_eq!(dispatcher.last_response_len(), response_len);
    }

    #[test]
    fn dispatches_requests_read_from_linear_memory() {
        let mut dispatcher = WebHostCallDispatcher::new();
        dispatcher.register("navigation.navigate", |params| {
            Ok(json!({
                "ok": params.get("to").cloned().unwrap_or(json!(null))
            }))
        });

        let request = serde_json::to_vec(&HostCallEnvelope {
            operation: "navigation.navigate".to_string(),
            params: BTreeMap::from([(String::from("to"), json!("/app/mangas/hot"))]),
        })
        .expect("request");

        let mut memory = vec![];
        write_bytes(&mut memory, 0, &request).expect("write request");
        let response = dispatcher
            .dispatch_from_memory(&memory, 0, request.len() as u32)
            .expect("dispatch from memory");

        let value = decode_json_bytes::<serde_json::Value>(response).expect("decode");
        assert_eq!(value["ok"], "/app/mangas/hot");
    }

    #[test]
    fn errors_on_unknown_operation() {
        let mut dispatcher = WebHostCallDispatcher::new();
        assert!(matches!(
            dispatcher.dispatch_envelope(&HostCallEnvelope {
                operation: "missing.op".to_string(),
                params: BTreeMap::new(),
            }),
            Err(WebHostCallError::UnknownOperation(op)) if op == "missing.op"
        ));
    }
}
