use serde::{Serialize, de::DeserializeOwned};
use thiserror::Error;
use unode_sdk::abi::AbiError;
use unode_sdk::{PluginManifestEnvelope, WasmPtrLen, decode_json_bytes, encode_json_bytes};

use crate::host_call::{TuiHostCallDispatcher, TuiHostCallError};
use crate::memory::TuiMemoryError;

pub trait TuiGuestInstance {
    fn read_memory(&self, ptr: u32, len: u32) -> Result<Vec<u8>, TuiAbiBridgeError>;
    fn write_memory(&mut self, ptr: u32, bytes: &[u8]) -> Result<(), TuiAbiBridgeError>;
    fn alloc(&mut self, len: u32) -> Result<u32, TuiAbiBridgeError>;
    fn dealloc(&mut self, ptr: u32, len: u32) -> Result<(), TuiAbiBridgeError>;
    fn plugin_manifest(&mut self) -> Result<u32, TuiAbiBridgeError>;
    fn plugin_manifest_len(&mut self) -> Result<u32, TuiAbiBridgeError>;
    fn plugin_load(&mut self, request_ptr: u32, request_len: u32)
    -> Result<u32, TuiAbiBridgeError>;
    fn plugin_load_result_len(&mut self) -> Result<u32, TuiAbiBridgeError>;
    fn plugin_render(
        &mut self,
        request_ptr: u32,
        request_len: u32,
    ) -> Result<u32, TuiAbiBridgeError>;
    fn plugin_render_result_len(&mut self) -> Result<u32, TuiAbiBridgeError>;
    fn plugin_render_slot(
        &mut self,
        request_ptr: u32,
        request_len: u32,
    ) -> Result<u32, TuiAbiBridgeError>;
    fn plugin_render_slot_result_len(&mut self) -> Result<u32, TuiAbiBridgeError>;
    fn plugin_dispatch(
        &mut self,
        request_ptr: u32,
        request_len: u32,
    ) -> Result<u32, TuiAbiBridgeError>;
    fn plugin_dispatch_result_len(&mut self) -> Result<u32, TuiAbiBridgeError>;
}

#[derive(Debug, Error)]
pub enum TuiAbiBridgeError {
    #[error(transparent)]
    Abi(#[from] AbiError),
    #[error(transparent)]
    Memory(#[from] TuiMemoryError),
    #[error(transparent)]
    HostCall(#[from] TuiHostCallError),
    #[error("guest export error: {0}")]
    Guest(String),
}

#[derive(Debug, Default)]
pub struct TuiHostImportAdapter {
    dispatcher: TuiHostCallDispatcher,
    last_result_len: u32,
}

impl TuiHostImportAdapter {
    pub fn new(dispatcher: TuiHostCallDispatcher) -> Self {
        Self {
            dispatcher,
            last_result_len: 0,
        }
    }

    pub fn host_call<G: TuiGuestInstance>(
        &mut self,
        guest: &mut G,
        request_ptr: u32,
        request_len: u32,
    ) -> Result<u32, TuiAbiBridgeError> {
        let request_bytes = guest.read_memory(request_ptr, request_len)?;
        let response_bytes = self.dispatcher.dispatch_bytes(&request_bytes)?.to_vec();
        let response_ptr = guest.alloc(response_bytes.len() as u32)?;
        guest.write_memory(response_ptr, &response_bytes)?;
        self.last_result_len = response_bytes.len() as u32;
        Ok(response_ptr)
    }

    pub fn host_call_result_len(&self) -> u32 {
        self.last_result_len
    }
}

#[derive(Debug)]
pub struct TuiPluginBridge<G> {
    guest: G,
    imports: TuiHostImportAdapter,
}

impl<G> TuiPluginBridge<G> {
    pub fn new(guest: G, imports: TuiHostImportAdapter) -> Self {
        Self { guest, imports }
    }

    pub fn guest(&self) -> &G {
        &self.guest
    }

    pub fn guest_mut(&mut self) -> &mut G {
        &mut self.guest
    }

    pub fn imports(&self) -> &TuiHostImportAdapter {
        &self.imports
    }
}

impl<G: TuiGuestInstance> TuiPluginBridge<G> {
    pub fn call_plugin_manifest(&mut self) -> Result<PluginManifestEnvelope, TuiAbiBridgeError> {
        let ptr = self.guest.plugin_manifest()?;
        let len = self.guest.plugin_manifest_len()?;
        let bytes = self.guest.read_memory(ptr, len)?;
        self.guest.dealloc(ptr, len)?;
        decode_json_bytes(&bytes).map_err(TuiAbiBridgeError::from)
    }

    pub fn call_plugin_render<Req, Resp>(
        &mut self,
        request: &Req,
    ) -> Result<Resp, TuiAbiBridgeError>
    where
        Req: Serialize,
        Resp: DeserializeOwned,
    {
        self.call_json_export(
            request,
            |guest, ptr, len| guest.plugin_render(ptr, len),
            |guest| guest.plugin_render_result_len(),
        )
    }

    pub fn call_plugin_render_slot<Resp>(
        &mut self,
        request: &unode_sdk::PluginRenderSlotRequest,
    ) -> Result<Resp, TuiAbiBridgeError>
    where
        Resp: DeserializeOwned,
    {
        self.call_json_export(
            request,
            |guest, ptr, len| guest.plugin_render_slot(ptr, len),
            |guest| guest.plugin_render_slot_result_len(),
        )
    }

    pub fn call_plugin_load<Req, Resp>(&mut self, request: &Req) -> Result<Resp, TuiAbiBridgeError>
    where
        Req: Serialize,
        Resp: DeserializeOwned,
    {
        self.call_json_export(
            request,
            |guest, ptr, len| guest.plugin_load(ptr, len),
            |guest| guest.plugin_load_result_len(),
        )
    }

    pub fn call_plugin_dispatch<Resp>(
        &mut self,
        request: &unode_sdk::PluginDispatchRequest,
    ) -> Result<Resp, TuiAbiBridgeError>
    where
        Resp: DeserializeOwned,
    {
        self.call_json_export(
            request,
            |guest, ptr, len| guest.plugin_dispatch(ptr, len),
            |guest| guest.plugin_dispatch_result_len(),
        )
    }

    fn call_json_export<Req, Resp>(
        &mut self,
        request: &Req,
        call: impl FnOnce(&mut G, u32, u32) -> Result<u32, TuiAbiBridgeError>,
        result_len: impl FnOnce(&mut G) -> Result<u32, TuiAbiBridgeError>,
    ) -> Result<Resp, TuiAbiBridgeError>
    where
        Req: Serialize,
        Resp: DeserializeOwned,
    {
        let request_bytes = encode_json_bytes(request)?;
        let request_ptr = self.guest.alloc(request_bytes.len() as u32)?;
        self.guest.write_memory(request_ptr, &request_bytes)?;

        let result_ptr = call(&mut self.guest, request_ptr, request_bytes.len() as u32)?;
        self.guest
            .dealloc(request_ptr, request_bytes.len() as u32)?;
        let result_len = result_len(&mut self.guest)?;
        let result_bytes = self.guest.read_memory(result_ptr, result_len)?;
        self.guest.dealloc(result_ptr, result_len)?;
        decode_json_bytes(&result_bytes).map_err(TuiAbiBridgeError::from)
    }

    pub fn invoke_host_call_import(
        &mut self,
        request_ptr: u32,
        request_len: u32,
    ) -> Result<WasmPtrLen, TuiAbiBridgeError> {
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
    use unode::core::dsl::IntoNode;
    use unode::core::runtime::ResolvedRoute;
    use unode_sdk::{
        HostCallEnvelope, PluginDispatchOutcome, PluginDispatchResponse, PluginManifestEnvelope,
        PluginRenderRequest, PluginRenderSlotResponse, UNODE_PLUGIN_ABI_VERSION, plugin_manifest,
    };

    use super::{TuiAbiBridgeError, TuiGuestInstance, TuiHostImportAdapter, TuiPluginBridge};
    use crate::host_call::TuiHostCallDispatcher;
    use crate::memory::{read_bytes, read_json, write_bytes};

    #[derive(Debug)]
    struct FakeGuest {
        memory: Vec<u8>,
        manifest_len: u32,
        load_len: u32,
        render_len: u32,
        render_slot_len: u32,
        dispatch_len: u32,
    }

    impl FakeGuest {
        fn new() -> Self {
            Self {
                memory: vec![],
                manifest_len: 0,
                load_len: 0,
                render_len: 0,
                render_slot_len: 0,
                dispatch_len: 0,
            }
        }

        fn write_guest_bytes(&mut self, bytes: &[u8]) -> u32 {
            let ptr = self.memory.len() as u32;
            self.memory.extend_from_slice(bytes);
            ptr
        }
    }

    impl TuiGuestInstance for FakeGuest {
        fn read_memory(&self, ptr: u32, len: u32) -> Result<Vec<u8>, TuiAbiBridgeError> {
            read_bytes(&self.memory, ptr, len).map_err(TuiAbiBridgeError::from)
        }

        fn write_memory(&mut self, ptr: u32, bytes: &[u8]) -> Result<(), TuiAbiBridgeError> {
            write_bytes(&mut self.memory, ptr, bytes).map_err(TuiAbiBridgeError::from)
        }

        fn alloc(&mut self, len: u32) -> Result<u32, TuiAbiBridgeError> {
            let ptr = self.memory.len() as u32;
            self.memory.resize(self.memory.len() + len as usize, 0);
            Ok(ptr)
        }

        fn dealloc(&mut self, _ptr: u32, _len: u32) -> Result<(), TuiAbiBridgeError> {
            Ok(())
        }

        fn plugin_manifest(&mut self) -> Result<u32, TuiAbiBridgeError> {
            let envelope = PluginManifestEnvelope {
                abi_version: UNODE_PLUGIN_ABI_VERSION.to_string(),
                manifest: plugin_manifest("demo.plugin", "Demo").build(),
            };
            let bytes = serde_json::to_vec(&envelope).expect("manifest bytes");
            self.manifest_len = bytes.len() as u32;
            Ok(self.write_guest_bytes(&bytes))
        }

        fn plugin_manifest_len(&mut self) -> Result<u32, TuiAbiBridgeError> {
            Ok(self.manifest_len)
        }

        fn plugin_load(
            &mut self,
            request_ptr: u32,
            request_len: u32,
        ) -> Result<u32, TuiAbiBridgeError> {
            let request =
                read_json::<unode_sdk::PluginLoadRequest>(&self.memory, request_ptr, request_len)
                    .expect("load request");
            let response = json!({
                "route": request.route.pattern,
                "locale": request.locale
            });
            let bytes = serde_json::to_vec(&response).expect("load response");
            self.load_len = bytes.len() as u32;
            Ok(self.write_guest_bytes(&bytes))
        }

        fn plugin_load_result_len(&mut self) -> Result<u32, TuiAbiBridgeError> {
            Ok(self.load_len)
        }

        fn plugin_render(
            &mut self,
            request_ptr: u32,
            request_len: u32,
        ) -> Result<u32, TuiAbiBridgeError> {
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

        fn plugin_render_result_len(&mut self) -> Result<u32, TuiAbiBridgeError> {
            Ok(self.render_len)
        }

        fn plugin_render_slot(
            &mut self,
            request_ptr: u32,
            request_len: u32,
        ) -> Result<u32, TuiAbiBridgeError> {
            let request = read_json::<unode_sdk::PluginRenderSlotRequest>(
                &self.memory,
                request_ptr,
                request_len,
            )
            .expect("render slot request");
            let response = PluginRenderSlotResponse {
                nodes: vec![
                    unode::core::dsl::text(format!(
                        "{}:{}",
                        request.slot_name, request.contribution_id
                    ))
                    .into_node(),
                ],
            };
            let bytes = serde_json::to_vec(&response).expect("render slot response");
            self.render_slot_len = bytes.len() as u32;
            Ok(self.write_guest_bytes(&bytes))
        }

        fn plugin_render_slot_result_len(&mut self) -> Result<u32, TuiAbiBridgeError> {
            Ok(self.render_slot_len)
        }

        fn plugin_dispatch(
            &mut self,
            request_ptr: u32,
            request_len: u32,
        ) -> Result<u32, TuiAbiBridgeError> {
            let request = read_json::<unode_sdk::PluginDispatchRequest>(
                &self.memory,
                request_ptr,
                request_len,
            )
            .expect("dispatch request");
            let response = PluginDispatchResponse {
                handled: true,
                outcome: PluginDispatchOutcome::Navigate {
                    to: request.route.pattern,
                },
                message: Some("handled".to_string()),
                data: None,
            };
            let bytes = serde_json::to_vec(&response).expect("dispatch response");
            self.dispatch_len = bytes.len() as u32;
            Ok(self.write_guest_bytes(&bytes))
        }

        fn plugin_dispatch_result_len(&mut self) -> Result<u32, TuiAbiBridgeError> {
            Ok(self.dispatch_len)
        }
    }

    fn bridge() -> TuiPluginBridge<FakeGuest> {
        let mut dispatcher = TuiHostCallDispatcher::new();
        dispatcher.register("navigation.navigate", |params| {
            Ok(json!({
                "ok": true,
                "to": params.get("to").cloned().unwrap_or(JsonValue::Null)
            }))
        });

        TuiPluginBridge::new(FakeGuest::new(), TuiHostImportAdapter::new(dispatcher))
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
    fn calls_plugin_load_through_guest_exports() {
        let mut bridge = bridge();
        let request = unode_sdk::PluginLoadRequest {
            route: ResolvedRoute {
                pattern: "/plugins/sanity-check".to_string(),
                params: BTreeMap::new(),
                query: BTreeMap::new(),
            },
            state_snapshot: BTreeMap::new(),
            locale: Some("en".to_string()),
        };

        let response = bridge
            .call_plugin_load::<_, JsonValue>(&request)
            .expect("load response");

        assert_eq!(response["route"], "/plugins/sanity-check");
        assert_eq!(response["locale"], "en");
    }

    #[test]
    fn calls_plugin_dispatch_through_guest_exports() {
        let mut bridge = bridge();
        let response = bridge
            .call_plugin_dispatch::<PluginDispatchResponse>(&unode_sdk::PluginDispatchRequest {
                route: ResolvedRoute {
                    pattern: "/plugins/demo".to_string(),
                    params: BTreeMap::new(),
                    query: BTreeMap::new(),
                },
                action: unode::core::ast::ActionRef {
                    r#type: unode::core::ast::ActionType::Custom("demo.refresh".to_string()),
                    params: None,
                    confirm: None,
                },
                state_snapshot: BTreeMap::new(),
                locale: Some("en".to_string()),
            })
            .expect("dispatch");

        assert!(response.handled);
        assert_eq!(
            response,
            PluginDispatchResponse {
                handled: true,
                outcome: PluginDispatchOutcome::Navigate {
                    to: "/plugins/demo".to_string(),
                },
                message: Some("handled".to_string()),
                data: None,
            }
        );
    }

    #[test]
    fn calls_plugin_render_slot_through_guest_exports() {
        let mut bridge = bridge();
        let response = bridge
            .call_plugin_render_slot::<PluginRenderSlotResponse>(
                &unode_sdk::PluginRenderSlotRequest {
                    contribution_id: "reviews-summary".to_string(),
                    slot_name: "catalog.work-detail:footer".to_string(),
                    route: ResolvedRoute::default(),
                    state_snapshot: BTreeMap::new(),
                    locale: Some("en".to_string()),
                },
            )
            .expect("render slot");

        assert_eq!(response.nodes.len(), 1);
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
        bridge
            .guest_mut()
            .write_memory(request_ptr, &request)
            .expect("write request");

        let response = bridge
            .invoke_host_call_import(request_ptr, request.len() as u32)
            .expect("host call");

        let response_bytes = bridge
            .guest()
            .read_memory(response.ptr, response.len)
            .expect("response bytes");
        let response_json =
            serde_json::from_slice::<JsonValue>(&response_bytes).expect("response json");
        assert_eq!(response_json["ok"], true);
        assert_eq!(response_json["to"], "/app/mangas/hot");
        assert_eq!(bridge.imports().host_call_result_len(), response.len);
    }
}
