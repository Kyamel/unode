use std::collections::BTreeMap;

use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json::Value as JsonValue;
use thiserror::Error;
use unode::core::ast::ActionRef;
use unode::core::runtime::{PluginManifest, ResolvedRoute};
pub use unode::core::slot::{PluginRenderSlotRequest, PluginRenderSlotResponse};

pub const UNODE_PLUGIN_ABI_VERSION: &str = "0.2.0";
pub const UNODE_PLUGIN_ABI_VERSION_BYTES: &[u8] = b"0.2.0\0";

pub const EXPORT_UNODE_ALLOC: &str = "unode_alloc";
pub const EXPORT_UNODE_DEALLOC: &str = "unode_dealloc";
pub const EXPORT_PLUGIN_ABI_VERSION: &str = "plugin_abi_version";
pub const EXPORT_PLUGIN_MANIFEST: &str = "plugin_manifest";
pub const EXPORT_PLUGIN_MANIFEST_LEN: &str = "plugin_manifest_len";
pub const EXPORT_PLUGIN_LOAD: &str = "plugin_load";
pub const EXPORT_PLUGIN_LOAD_RESULT_LEN: &str = "plugin_load_result_len";
pub const EXPORT_PLUGIN_RENDER: &str = "plugin_render";
pub const EXPORT_PLUGIN_RENDER_RESULT_LEN: &str = "plugin_render_result_len";
pub const EXPORT_PLUGIN_RENDER_SLOT: &str = "plugin_render_slot";
pub const EXPORT_PLUGIN_RENDER_SLOT_RESULT_LEN: &str = "plugin_render_slot_result_len";
pub const EXPORT_PLUGIN_DISPATCH: &str = "plugin_dispatch";
pub const EXPORT_PLUGIN_DISPATCH_RESULT_LEN: &str = "plugin_dispatch_result_len";
pub const IMPORT_HOST_CALL: &str = "host_call";
pub const IMPORT_HOST_CALL_RESULT_LEN: &str = "host_call_result_len";

pub const REQUIRED_EXPORTS: [&str; 13] = [
    EXPORT_UNODE_ALLOC,
    EXPORT_UNODE_DEALLOC,
    EXPORT_PLUGIN_ABI_VERSION,
    EXPORT_PLUGIN_MANIFEST,
    EXPORT_PLUGIN_MANIFEST_LEN,
    EXPORT_PLUGIN_LOAD,
    EXPORT_PLUGIN_LOAD_RESULT_LEN,
    EXPORT_PLUGIN_RENDER,
    EXPORT_PLUGIN_RENDER_RESULT_LEN,
    EXPORT_PLUGIN_RENDER_SLOT,
    EXPORT_PLUGIN_RENDER_SLOT_RESULT_LEN,
    EXPORT_PLUGIN_DISPATCH,
    EXPORT_PLUGIN_DISPATCH_RESULT_LEN,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct WasmPtrLen {
    pub ptr: u32,
    pub len: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PluginManifestEnvelope {
    pub abi_version: String,
    pub manifest: PluginManifest,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PluginLoadRequest {
    pub route: ResolvedRoute,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub state_snapshot: BTreeMap<String, JsonValue>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locale: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PluginRenderRequest {
    pub route: ResolvedRoute,
    pub data: JsonValue,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub state_snapshot: BTreeMap<String, JsonValue>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locale: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginDispatchRequest {
    pub route: ResolvedRoute,
    pub action: ActionRef,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub state_snapshot: BTreeMap<String, JsonValue>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locale: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum PluginDispatchOutcome {
    None,
    RefreshCurrentScreen,
    Navigate { to: String },
}

impl Default for PluginDispatchOutcome {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub struct PluginDispatchResponse {
    #[serde(default)]
    pub handled: bool,
    #[serde(default)]
    pub outcome: PluginDispatchOutcome,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<JsonValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct HostCallEnvelope {
    pub operation: String,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub params: BTreeMap<String, JsonValue>,
}

#[derive(Debug, Error)]
pub enum AbiError {
    #[error("failed to encode ABI json")]
    Encode(#[from] serde_json::Error),
}

pub fn encode_json_bytes<T: Serialize>(value: &T) -> Result<Vec<u8>, AbiError> {
    serde_json::to_vec(value).map_err(AbiError::from)
}

pub fn decode_json_bytes<T: DeserializeOwned>(bytes: &[u8]) -> Result<T, AbiError> {
    serde_json::from_slice(bytes).map_err(AbiError::from)
}

/// Exports the allocator functions required by the raw plugin ABI.
///
/// Most plugins should use [`export_plugin!`] instead, which includes these
/// allocators and the standard manifest/load/render/dispatch exports.
#[macro_export]
macro_rules! export_allocators {
    () => {
        #[unsafe(no_mangle)]
        pub extern "C" fn unode_alloc(len: usize) -> *mut u8 {
            let size = len.max(1);
            let layout = std::alloc::Layout::from_size_align(size, std::mem::align_of::<u8>())
                .expect("valid allocation layout");
            unsafe { std::alloc::alloc(layout) }
        }

        #[unsafe(no_mangle)]
        pub extern "C" fn unode_dealloc(ptr: *mut u8, len: usize) {
            if ptr.is_null() {
                return;
            }

            let size = len.max(1);
            let layout = std::alloc::Layout::from_size_align(size, std::mem::align_of::<u8>())
                .expect("valid allocation layout");
            unsafe { std::alloc::dealloc(ptr, layout) };
        }
    };
}

/// Exports the complete raw Unode plugin ABI for a Rust plugin.
///
/// The plugin author provides ordinary Rust functions for each lifecycle hook:
///
/// ```ignore
/// fn manifest() -> PluginManifestEnvelope { ... }
/// fn load(request: &PluginLoadRequest) -> serde_json::Value { ... }
/// fn render(request: &PluginRenderRequest) -> ScreenNode { ... }
/// fn dispatch(request: &PluginDispatchRequest) -> PluginDispatchResponse { ... }
///
/// unode_sdk::export_plugin! {
///     manifest: manifest,
///     load: load,
///     render: render,
///     dispatch: dispatch,
/// }
/// ```
///
/// The macro generates the `extern "C"` exports the host runtimes look up by
/// name, plus `unode_alloc` and `unode_dealloc`. This keeps the ABI explicit for
/// hosts while making plugins feel like normal Rust code.
#[macro_export]
macro_rules! export_plugin {
    (
        manifest: $manifest:expr,
        load: $load:expr,
        render: $render:expr,
        render_slot: $render_slot:expr,
        dispatch: $dispatch:expr $(,)?
    ) => {
        $crate::export_allocators!();

        thread_local! {
            static __UNODE_MANIFEST_BUFFER_LEN: ::std::cell::Cell<u32> = const { ::std::cell::Cell::new(0) };
            static __UNODE_LOAD_BUFFER_LEN: ::std::cell::Cell<u32> = const { ::std::cell::Cell::new(0) };
            static __UNODE_RENDER_BUFFER_LEN: ::std::cell::Cell<u32> = const { ::std::cell::Cell::new(0) };
            static __UNODE_RENDER_SLOT_BUFFER_LEN: ::std::cell::Cell<u32> = const { ::std::cell::Cell::new(0) };
            static __UNODE_DISPATCH_BUFFER_LEN: ::std::cell::Cell<u32> = const { ::std::cell::Cell::new(0) };
        }

        fn __unode_with_output_buffer<F>(
            len_cell: &'static ::std::thread::LocalKey<::std::cell::Cell<u32>>,
            build: F,
        ) -> u32
        where
            F: FnOnce() -> ::std::vec::Vec<u8>,
        {
            len_cell.with(|slot| {
                let bytes = build();
                let len = bytes.len() as u32;
                let ptr = unode_alloc(bytes.len()) as u32;
                unsafe {
                    ::std::ptr::copy_nonoverlapping(bytes.as_ptr(), ptr as *mut u8, bytes.len());
                }
                slot.set(len);
                ptr
            })
        }

        fn __unode_output_len(
            len_cell: &'static ::std::thread::LocalKey<::std::cell::Cell<u32>>,
        ) -> u32 {
            len_cell.with(::std::cell::Cell::get)
        }

        fn __unode_decode_guest_json<T: ::serde::de::DeserializeOwned>(
            ptr: u32,
            len: u32,
        ) -> T {
            let bytes = unsafe { ::std::slice::from_raw_parts(ptr as *const u8, len as usize) };
            $crate::decode_json_bytes(bytes).expect("guest request must be valid ABI JSON")
        }

        fn __unode_json_or_error<T: ::serde::Serialize>(value: &T) -> ::std::vec::Vec<u8> {
            $crate::encode_json_bytes(value).unwrap_or_else(|err| {
                ::serde_json::to_vec(&::serde_json::json!({ "error": err.to_string() }))
                    .expect("fallback json")
            })
        }

        #[unsafe(no_mangle)]
        pub extern "C" fn plugin_abi_version() -> *const u8 {
            $crate::UNODE_PLUGIN_ABI_VERSION_BYTES.as_ptr()
        }

        #[unsafe(no_mangle)]
        pub extern "C" fn plugin_manifest() -> u32 {
            __unode_with_output_buffer(&__UNODE_MANIFEST_BUFFER_LEN, || {
                __unode_json_or_error(&($manifest)())
            })
        }

        #[unsafe(no_mangle)]
        pub extern "C" fn plugin_manifest_len() -> u32 {
            __unode_output_len(&__UNODE_MANIFEST_BUFFER_LEN)
        }

        #[unsafe(no_mangle)]
        pub extern "C" fn plugin_load(request_ptr: u32, request_len: u32) -> u32 {
            let request = __unode_decode_guest_json::<$crate::PluginLoadRequest>(
                request_ptr,
                request_len,
            );
            __unode_with_output_buffer(&__UNODE_LOAD_BUFFER_LEN, || {
                __unode_json_or_error(&($load)(&request))
            })
        }

        #[unsafe(no_mangle)]
        pub extern "C" fn plugin_load_result_len() -> u32 {
            __unode_output_len(&__UNODE_LOAD_BUFFER_LEN)
        }

        #[unsafe(no_mangle)]
        pub extern "C" fn plugin_render(request_ptr: u32, request_len: u32) -> u32 {
            let request = __unode_decode_guest_json::<$crate::PluginRenderRequest>(
                request_ptr,
                request_len,
            );
            __unode_with_output_buffer(&__UNODE_RENDER_BUFFER_LEN, || {
                __unode_json_or_error(&($render)(&request))
            })
        }

        #[unsafe(no_mangle)]
        pub extern "C" fn plugin_render_result_len() -> u32 {
            __unode_output_len(&__UNODE_RENDER_BUFFER_LEN)
        }

        #[unsafe(no_mangle)]
        pub extern "C" fn plugin_render_slot(request_ptr: u32, request_len: u32) -> u32 {
            let request = __unode_decode_guest_json::<$crate::PluginRenderSlotRequest>(
                request_ptr,
                request_len,
            );
            __unode_with_output_buffer(&__UNODE_RENDER_SLOT_BUFFER_LEN, || {
                __unode_json_or_error(&($render_slot)(&request))
            })
        }

        #[unsafe(no_mangle)]
        pub extern "C" fn plugin_render_slot_result_len() -> u32 {
            __unode_output_len(&__UNODE_RENDER_SLOT_BUFFER_LEN)
        }

        #[unsafe(no_mangle)]
        pub extern "C" fn plugin_dispatch(request_ptr: u32, request_len: u32) -> u32 {
            let request = __unode_decode_guest_json::<$crate::PluginDispatchRequest>(
                request_ptr,
                request_len,
            );
            __unode_with_output_buffer(&__UNODE_DISPATCH_BUFFER_LEN, || {
                __unode_json_or_error(&($dispatch)(&request))
            })
        }

        #[unsafe(no_mangle)]
        pub extern "C" fn plugin_dispatch_result_len() -> u32 {
            __unode_output_len(&__UNODE_DISPATCH_BUFFER_LEN)
        }
    };
    (
        manifest: $manifest:expr,
        load: $load:expr,
        render: $render:expr,
        dispatch: $dispatch:expr,
        render_slot: $render_slot:expr $(,)?
    ) => {
        $crate::export_plugin! {
            manifest: $manifest,
            load: $load,
            render: $render,
            render_slot: $render_slot,
            dispatch: $dispatch,
        }
    };
    (
        manifest: $manifest:expr,
        load: $load:expr,
        render: $render:expr,
        dispatch: $dispatch:expr $(,)?
    ) => {
        $crate::export_plugin! {
            manifest: $manifest,
            load: $load,
            render: $render,
            render_slot: |_request: &$crate::PluginRenderSlotRequest| {
                $crate::PluginRenderSlotResponse { nodes: ::std::vec::Vec::new() }
            },
            dispatch: $dispatch,
        }
    };
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use serde_json::json;

    use super::{
        EXPORT_PLUGIN_DISPATCH, EXPORT_PLUGIN_LOAD, EXPORT_PLUGIN_MANIFEST, EXPORT_PLUGIN_RENDER,
        EXPORT_PLUGIN_RENDER_SLOT, HostCallEnvelope, PluginDispatchOutcome, PluginDispatchRequest,
        PluginDispatchResponse, PluginManifestEnvelope, PluginRenderRequest,
        PluginRenderSlotRequest, PluginRenderSlotResponse, REQUIRED_EXPORTS,
        UNODE_PLUGIN_ABI_VERSION, decode_json_bytes, encode_json_bytes,
    };
    use crate::plugin_manifest;
    use unode::core::ast::{ActionRef, ActionType};
    use unode::core::runtime::ResolvedRoute;

    #[test]
    fn roundtrips_manifest_envelope() {
        let envelope = PluginManifestEnvelope {
            abi_version: UNODE_PLUGIN_ABI_VERSION.to_string(),
            manifest: plugin_manifest("demo.plugin", "Demo").build(),
        };

        let bytes = encode_json_bytes(&envelope).expect("encode");
        let decoded = decode_json_bytes::<PluginManifestEnvelope>(&bytes).expect("decode");
        assert_eq!(decoded, envelope);
    }

    #[test]
    fn abi_version_and_required_exports_include_slots() {
        assert_eq!(UNODE_PLUGIN_ABI_VERSION, "0.2.0");
        assert!(REQUIRED_EXPORTS.contains(&EXPORT_PLUGIN_RENDER_SLOT));
        assert!(REQUIRED_EXPORTS.contains(&"plugin_render_slot_result_len"));
    }

    #[test]
    fn serializes_render_and_dispatch_requests() {
        let route = ResolvedRoute {
            pattern: "/works/:id".to_string(),
            params: BTreeMap::from([(String::from("id"), String::from("42"))]),
            query: BTreeMap::new(),
        };

        let render = PluginRenderRequest {
            route: route.clone(),
            data: json!({ "title": "Hot" }),
            state_snapshot: BTreeMap::from([(String::from("ui.page"), json!(2))]),
            locale: Some("pt-BR".to_string()),
        };
        let dispatch = PluginDispatchRequest {
            route,
            action: ActionRef {
                r#type: ActionType::Custom("favorite.toggle".to_string()),
                params: None,
                confirm: None,
            },
            state_snapshot: BTreeMap::new(),
            locale: Some("pt-BR".to_string()),
        };

        let render_json = serde_json::to_value(render).expect("render json");
        let dispatch_json = serde_json::to_value(dispatch).expect("dispatch json");

        assert_eq!(render_json["locale"], "pt-BR");
        assert_eq!(dispatch_json["action"]["type"], "favorite.toggle");
    }

    #[test]
    fn serializes_dispatch_response() {
        let response = PluginDispatchResponse {
            handled: true,
            outcome: PluginDispatchOutcome::Navigate {
                to: "/plugins/demo".to_string(),
            },
            message: Some("navigating".to_string()),
            data: Some(json!({ "source": "plugin" })),
        };

        let bytes = encode_json_bytes(&response).expect("encode");
        let decoded = decode_json_bytes::<PluginDispatchResponse>(&bytes).expect("decode");
        assert_eq!(decoded, response);
    }

    #[test]
    fn serializes_default_render_slot_response() {
        let request = PluginRenderSlotRequest {
            contribution_id: "reviews-summary".to_string(),
            slot_name: "catalog.work-detail:footer".to_string(),
            route: ResolvedRoute::default(),
            state_snapshot: BTreeMap::new(),
            locale: None,
        };
        let response = PluginRenderSlotResponse::default();

        let request_json = serde_json::to_value(&request).expect("request");
        let response_json = serde_json::to_value(&response).expect("response");

        assert_eq!(request_json["contributionId"], "reviews-summary");
        assert_eq!(response_json, json!({ "nodes": [] }));
    }

    #[test]
    fn serializes_host_call_envelope() {
        let call = HostCallEnvelope {
            operation: "navigation.navigate".to_string(),
            params: BTreeMap::from([
                (String::from("to"), json!("/app/mangas/hot")),
                (String::from("replace"), json!(false)),
            ]),
        };

        let bytes = encode_json_bytes(&call).expect("encode");
        let decoded = decode_json_bytes::<HostCallEnvelope>(&bytes).expect("decode");
        assert_eq!(decoded.operation, "navigation.navigate");
        assert_eq!(decoded.params.get("to"), Some(&json!("/app/mangas/hot")));
    }

    #[test]
    fn raw_abi_and_wit_expose_same_lifecycle_functions() {
        let wit = include_str!("../../../wit/unode-plugin.wit");
        for export in [
            EXPORT_PLUGIN_MANIFEST,
            EXPORT_PLUGIN_LOAD,
            EXPORT_PLUGIN_RENDER,
            EXPORT_PLUGIN_RENDER_SLOT,
            EXPORT_PLUGIN_DISPATCH,
        ] {
            let wit_name = export.strip_prefix("plugin_").unwrap().replace('_', "-");
            assert!(
                wit.contains(&format!("{wit_name}: func")),
                "missing {wit_name} in WIT"
            );
        }
    }
}
