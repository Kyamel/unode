pub mod abi;
pub mod i18n;
pub mod manifest;
pub mod permissions;

pub use abi::{
    EXPORT_PLUGIN_ABI_VERSION, EXPORT_PLUGIN_DISPATCH, EXPORT_PLUGIN_DISPATCH_RESULT_LEN,
    EXPORT_PLUGIN_LOAD, EXPORT_PLUGIN_LOAD_RESULT_LEN, EXPORT_PLUGIN_MANIFEST,
    EXPORT_PLUGIN_MANIFEST_LEN, EXPORT_PLUGIN_RENDER, EXPORT_PLUGIN_RENDER_RESULT_LEN,
    EXPORT_UNODE_ALLOC, EXPORT_UNODE_DEALLOC, HostCallEnvelope, IMPORT_HOST_CALL,
    IMPORT_HOST_CALL_RESULT_LEN, PluginDispatchOutcome, PluginDispatchRequest,
    PluginDispatchResponse, PluginLoadRequest, PluginManifestEnvelope, PluginRenderRequest,
    REQUIRED_EXPORTS, UNODE_PLUGIN_ABI_VERSION, UNODE_PLUGIN_ABI_VERSION_BYTES, WasmPtrLen,
    decode_json_bytes, encode_json_bytes,
};
pub use i18n::{
    I18nCatalogRegistrationEvent, I18nError, I18nInspector, I18nLookupEvent, I18nText,
    LocaleSource, MessageCatalog, MessageCatalogs, MessageEntry, MessageValue, MessageValues,
    PluginI18n, PluginTranslator, msg, msg_with,
};
pub use manifest::{PermissionRequestBuilder, PluginManifestBuilder, permission, plugin_manifest};
pub use permissions::{
    CoreBuiltinPermission, PermissionGrant, PermissionGuard, PermissionProfile, PermissionRequest,
    core_permission,
};

pub mod prelude {
    pub use crate::abi::{
        EXPORT_PLUGIN_ABI_VERSION, EXPORT_PLUGIN_DISPATCH, EXPORT_PLUGIN_DISPATCH_RESULT_LEN,
        EXPORT_PLUGIN_LOAD, EXPORT_PLUGIN_LOAD_RESULT_LEN, EXPORT_PLUGIN_MANIFEST,
        EXPORT_PLUGIN_MANIFEST_LEN, EXPORT_PLUGIN_RENDER, EXPORT_PLUGIN_RENDER_RESULT_LEN,
        EXPORT_UNODE_ALLOC, EXPORT_UNODE_DEALLOC, HostCallEnvelope, IMPORT_HOST_CALL,
        IMPORT_HOST_CALL_RESULT_LEN, PluginDispatchOutcome, PluginDispatchRequest,
        PluginDispatchResponse, PluginLoadRequest, PluginManifestEnvelope, PluginRenderRequest,
        REQUIRED_EXPORTS, UNODE_PLUGIN_ABI_VERSION, UNODE_PLUGIN_ABI_VERSION_BYTES, WasmPtrLen,
        decode_json_bytes, encode_json_bytes,
    };
    pub use crate::i18n::{
        I18nInspector, I18nText, LocaleSource, MessageCatalog, MessageCatalogs, MessageEntry,
        MessageValue, MessageValues, PluginI18n, PluginTranslator, msg, msg_with,
    };
    pub use crate::manifest::{
        PermissionRequestBuilder, PluginManifestBuilder, permission, plugin_manifest,
    };
    pub use crate::permissions::{
        CoreBuiltinPermission, PermissionGrant, PermissionGuard, PermissionProfile,
        PermissionRequest, core, core_permission,
    };
    pub use crate::{export_allocators, export_plugin};
    pub use unode::core::ast::*;
    pub use unode::core::chrome::*;
    pub use unode::core::dsl::*;
    pub use unode::core::permissions::*;
    pub use unode::core::runtime::{PluginManifest, ResolvedRoute, UNODE_CORE_API_VERSION};
    pub use unode::core::state::{MemoryStateStore, StateStore};
}
