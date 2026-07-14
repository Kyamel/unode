pub mod abi;
pub mod i18n;
pub mod manifest;
pub mod permissions;

pub use abi::{
    decode_json_bytes, encode_json_bytes, HostCallEnvelope, PluginDispatchOutcome,
    PluginDispatchRequest, PluginDispatchResponse, PluginLoadRequest, PluginManifestEnvelope,
    PluginRenderRequest, WasmPtrLen, EXPORT_PLUGIN_ABI_VERSION, EXPORT_PLUGIN_DISPATCH,
    EXPORT_PLUGIN_DISPATCH_RESULT_LEN, EXPORT_PLUGIN_LOAD, EXPORT_PLUGIN_LOAD_RESULT_LEN,
    EXPORT_PLUGIN_MANIFEST, EXPORT_PLUGIN_MANIFEST_LEN, EXPORT_PLUGIN_RENDER,
    EXPORT_PLUGIN_RENDER_RESULT_LEN, EXPORT_UNODE_ALLOC, EXPORT_UNODE_DEALLOC,
    IMPORT_HOST_CALL, IMPORT_HOST_CALL_RESULT_LEN, REQUIRED_EXPORTS, UNODE_PLUGIN_ABI_VERSION,
};
pub use i18n::{
    msg, msg_with, I18nCatalogRegistrationEvent, I18nError, I18nInspector, I18nLookupEvent, I18nText,
    LocaleSource, MessageCatalog, MessageCatalogs, MessageEntry, MessageValue, MessageValues,
    PluginI18n, PluginTranslator,
};
pub use manifest::{permission, plugin_manifest, PermissionRequestBuilder, PluginManifestBuilder};
pub use permissions::{core_permission, CoreBuiltinPermission, PermissionGrant, PermissionGuard, PermissionProfile, PermissionRequest};

pub mod prelude {
    pub use crate::abi::{
        decode_json_bytes, encode_json_bytes, HostCallEnvelope, PluginDispatchOutcome,
        PluginDispatchRequest, PluginDispatchResponse, PluginLoadRequest,
        PluginManifestEnvelope, PluginRenderRequest, WasmPtrLen, EXPORT_PLUGIN_ABI_VERSION,
        EXPORT_PLUGIN_DISPATCH, EXPORT_PLUGIN_DISPATCH_RESULT_LEN, EXPORT_PLUGIN_LOAD,
        EXPORT_PLUGIN_LOAD_RESULT_LEN, EXPORT_PLUGIN_MANIFEST, EXPORT_PLUGIN_MANIFEST_LEN,
        EXPORT_PLUGIN_RENDER, EXPORT_PLUGIN_RENDER_RESULT_LEN, EXPORT_UNODE_ALLOC,
        EXPORT_UNODE_DEALLOC, IMPORT_HOST_CALL, IMPORT_HOST_CALL_RESULT_LEN, REQUIRED_EXPORTS,
        UNODE_PLUGIN_ABI_VERSION,
    };
    pub use crate::i18n::{
        msg, msg_with, I18nInspector, I18nText, LocaleSource, MessageCatalog, MessageCatalogs,
        MessageEntry, MessageValue, MessageValues, PluginI18n, PluginTranslator,
    };
    pub use crate::manifest::{permission, plugin_manifest, PermissionRequestBuilder, PluginManifestBuilder};
    pub use crate::permissions::{core, core_permission, CoreBuiltinPermission, PermissionGrant, PermissionGuard, PermissionProfile, PermissionRequest};
    pub use unode::core::ast::*;
    pub use unode::core::chrome::*;
    pub use unode::core::dsl::*;
    pub use unode::core::permissions::*;
    pub use unode::core::runtime::{PluginManifest, ResolvedRoute, UNODE_CORE_API_VERSION};
    pub use unode::core::state::{MemoryStateStore, StateStore};
}
