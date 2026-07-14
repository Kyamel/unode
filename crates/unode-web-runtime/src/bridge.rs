use serde::{Serialize, de::DeserializeOwned};
use thiserror::Error;
use unode_sdk::abi::AbiError;
use unode_sdk::{PluginManifestEnvelope, WasmPtrLen, decode_json_bytes, encode_json_bytes};

use crate::host_call::{WebHostCallDispatcher, WebHostCallError};
use crate::memory::{WebMemoryError, read_bytes, write_bytes};

pub trait WebGuestInstance {
    fn memory(&self) -> &[u8];
    fn memory_mut(&mut self) -> &mut Vec<u8>;
    fn alloc(&mut self, len: u32) -> Result<u32, WebAbiBridgeError>;
    fn plugin_manifest(&mut self) -> Result<u32, WebAbiBridgeError>;
    fn plugin_manifest_len(&mut self) -> Result<u32, WebAbiBridgeError>;
    fn plugin_render(
        &mut self,
        request_ptr: u32,
        request_len: u32,
    ) -> Result<u32, WebAbiBridgeError>;
    fn plugin_render_result_len(&mut self) -> Result<u32, WebAbiBridgeError>;
}

#[derive(Debug, Error)]
pub enum WebAbiBridgeError {
    #[error(transparent)]
    Abi(#[from] AbiError),
    #[error(transparent)]
    Memory(#[from] WebMemoryError),
    #[error(transparent)]
    HostCall(#[from] WebHostCallError),
    #[error("guest export error: {0}")]
    Guest(String),
}

#[derive(Debug, Default)]
pub struct WebHostImportAdapter {
    dispatcher: WebHostCallDispatcher,
    last_result_len: u32,
}

impl WebHostImportAdapter {
    pub fn new(dispatcher: WebHostCallDispatcher) -> Self {
        Self {
            dispatcher,
            last_result_len: 0,
        }
    }

    pub fn dispatcher(&self) -> &WebHostCallDispatcher {
        &self.dispatcher
    }

    pub fn dispatcher_mut(&mut self) -> &mut WebHostCallDispatcher {
        &mut self.dispatcher
    }

    pub fn host_call<G: WebGuestInstance>(
        &mut self,
        guest: &mut G,
        request_ptr: u32,
        request_len: u32,
    ) -> Result<u32, WebAbiBridgeError> {
        let request_bytes = read_bytes(guest.memory(), request_ptr, request_len)?;
        let response_bytes = self.dispatcher.dispatch_bytes(&request_bytes)?.to_vec();
        let response_ptr = guest.alloc(response_bytes.len() as u32)?;
        write_bytes(guest.memory_mut(), response_ptr, &response_bytes)?;
        self.last_result_len = response_bytes.len() as u32;
        Ok(response_ptr)
    }

    pub fn host_call_result_len(&self) -> u32 {
        self.last_result_len
    }
}

#[derive(Debug)]
pub struct WebPluginBridge<G> {
    guest: G,
    imports: WebHostImportAdapter,
}

impl<G> WebPluginBridge<G> {
    pub fn new(guest: G, imports: WebHostImportAdapter) -> Self {
        Self { guest, imports }
    }

    pub fn guest(&self) -> &G {
        &self.guest
    }

    pub fn guest_mut(&mut self) -> &mut G {
        &mut self.guest
    }

    pub fn imports(&self) -> &WebHostImportAdapter {
        &self.imports
    }

    pub fn imports_mut(&mut self) -> &mut WebHostImportAdapter {
        &mut self.imports
    }
}

impl<G: WebGuestInstance> WebPluginBridge<G> {
    pub fn call_plugin_manifest(&mut self) -> Result<PluginManifestEnvelope, WebAbiBridgeError> {
        let ptr = self.guest.plugin_manifest()?;
        let len = self.guest.plugin_manifest_len()?;
        let bytes = read_bytes(self.guest.memory(), ptr, len)?;
        decode_json_bytes(&bytes).map_err(WebAbiBridgeError::from)
    }

    pub fn call_plugin_render<Req, Resp>(
        &mut self,
        request: &Req,
    ) -> Result<Resp, WebAbiBridgeError>
    where
        Req: Serialize,
        Resp: DeserializeOwned,
    {
        let request_bytes = encode_json_bytes(request)?;
        let request_ptr = self.guest.alloc(request_bytes.len() as u32)?;
        write_bytes(self.guest.memory_mut(), request_ptr, &request_bytes)?;

        let result_ptr = self
            .guest
            .plugin_render(request_ptr, request_bytes.len() as u32)?;
        let result_len = self.guest.plugin_render_result_len()?;
        let result_bytes = read_bytes(self.guest.memory(), result_ptr, result_len)?;

        decode_json_bytes(&result_bytes).map_err(WebAbiBridgeError::from)
    }

    pub fn invoke_host_call_import(
        &mut self,
        request_ptr: u32,
        request_len: u32,
    ) -> Result<WasmPtrLen, WebAbiBridgeError> {
        let ptr = self
            .imports
            .host_call(&mut self.guest, request_ptr, request_len)?;
        Ok(WasmPtrLen {
            ptr,
            len: self.imports.host_call_result_len(),
        })
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use serde_json::{Value as JsonValue, json};
    use unode::core::runtime::ResolvedRoute;
    use unode_sdk::{
        HostCallEnvelope, PluginManifestEnvelope, PluginRenderRequest, UNODE_PLUGIN_ABI_VERSION,
        plugin_manifest,
    };

    use super::{WebAbiBridgeError, WebGuestInstance, WebHostImportAdapter, WebPluginBridge};
    use crate::host_call::WebHostCallDispatcher;
    use crate::memory::{read_json, write_bytes};

    #[derive(Debug)]
    struct FakeGuest {
        memory: Vec<u8>,
        manifest_len: u32,
        render_len: u32,
    }

    impl FakeGuest {
        fn new() -> Self {
            Self {
                memory: vec![],
                manifest_len: 0,
                render_len: 0,
            }
        }

        fn write_guest_bytes(&mut self, bytes: &[u8]) -> u32 {
            let ptr = self.memory.len() as u32;
            self.memory.extend_from_slice(bytes);
            ptr
        }
    }

    impl WebGuestInstance for FakeGuest {
        fn memory(&self) -> &[u8] {
            &self.memory
        }

        fn memory_mut(&mut self) -> &mut Vec<u8> {
            &mut self.memory
        }

        fn alloc(&mut self, len: u32) -> Result<u32, WebAbiBridgeError> {
            let ptr = self.memory.len() as u32;
            self.memory.resize(self.memory.len() + len as usize, 0);
            Ok(ptr)
        }

        fn plugin_manifest(&mut self) -> Result<u32, WebAbiBridgeError> {
            let envelope = PluginManifestEnvelope {
                abi_version: UNODE_PLUGIN_ABI_VERSION.to_string(),
                manifest: plugin_manifest("demo.plugin", "Demo").build(),
            };
            let bytes = serde_json::to_vec(&envelope).expect("manifest bytes");
            self.manifest_len = bytes.len() as u32;
            Ok(self.write_guest_bytes(&bytes))
        }

        fn plugin_manifest_len(&mut self) -> Result<u32, WebAbiBridgeError> {
            Ok(self.manifest_len)
        }

        fn plugin_render(
            &mut self,
            request_ptr: u32,
            request_len: u32,
        ) -> Result<u32, WebAbiBridgeError> {
            let request = read_json::<PluginRenderRequest>(&self.memory, request_ptr, request_len)
                .expect("render request");
            let response = json!({
                "screenKind": request.route.pattern,
                "title": request.data["title"].clone(),
                "locale": request.locale
            });
            let bytes = serde_json::to_vec(&response).expect("render response");
            self.render_len = bytes.len() as u32;
            Ok(self.write_guest_bytes(&bytes))
        }

        fn plugin_render_result_len(&mut self) -> Result<u32, WebAbiBridgeError> {
            Ok(self.render_len)
        }
    }

    fn bridge() -> WebPluginBridge<FakeGuest> {
        let mut dispatcher = WebHostCallDispatcher::new();
        dispatcher.register("navigation.navigate", |params| {
            Ok(json!({
                "ok": true,
                "to": params.get("to").cloned().unwrap_or(JsonValue::Null)
            }))
        });

        WebPluginBridge::new(FakeGuest::new(), WebHostImportAdapter::new(dispatcher))
    }

    #[test]
    fn calls_plugin_manifest_through_guest_exports() {
        let mut bridge = bridge();
        let manifest = bridge.call_plugin_manifest().expect("manifest");
        assert_eq!(manifest.manifest.id, "demo.plugin");
    }

    #[test]
    fn calls_plugin_render_through_guest_exports() {
        let mut bridge = bridge();
        let request = PluginRenderRequest {
            route: ResolvedRoute {
                pattern: "/app/mangas/hot".to_string(),
                params: BTreeMap::new(),
                query: BTreeMap::new(),
            },
            data: json!({ "title": "Hot" }),
            state_snapshot: BTreeMap::new(),
            locale: Some("pt-BR".to_string()),
        };

        let response = bridge
            .call_plugin_render::<_, JsonValue>(&request)
            .expect("render response");

        assert_eq!(response["screenKind"], "/app/mangas/hot");
        assert_eq!(response["title"], "Hot");
        assert_eq!(response["locale"], "pt-BR");
    }

    #[test]
    fn implements_host_call_import_by_reading_guest_memory_and_writing_response_back() {
        let mut bridge = bridge();
        let request = serde_json::to_vec(&HostCallEnvelope {
            operation: "navigation.navigate".to_string(),
            params: BTreeMap::from([(String::from("to"), json!("/app/mangas/hot"))]),
        })
        .expect("host call request");

        let request_ptr = bridge
            .guest_mut()
            .alloc(request.len() as u32)
            .expect("alloc");
        write_bytes(bridge.guest_mut().memory_mut(), request_ptr, &request).expect("write request");

        let response = bridge
            .invoke_host_call_import(request_ptr, request.len() as u32)
            .expect("host call");

        let response_json =
            read_json::<JsonValue>(bridge.guest().memory(), response.ptr, response.len)
                .expect("response json");
        assert_eq!(response_json["ok"], true);
        assert_eq!(response_json["to"], "/app/mangas/hot");
        assert_eq!(bridge.imports().host_call_result_len(), response.len);
    }
}
