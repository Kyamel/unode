use std::collections::BTreeSet;
use std::sync::Arc;

use thiserror::Error;
use unode::core::permissions::PermissionProfile;
use unode_plugin_sdk::{PluginManifestEnvelope, REQUIRED_EXPORTS, UNODE_PLUGIN_ABI_VERSION};

#[derive(Debug, Clone)]
pub enum WebPluginSource {
    Url(String),
    Bytes(Arc<[u8]>),
}

#[derive(Debug, Clone)]
pub struct WebLoaderConfig {
    pub expected_abi_version: String,
    pub cache_modules: bool,
}

impl Default for WebLoaderConfig {
    fn default() -> Self {
        Self {
            expected_abi_version: UNODE_PLUGIN_ABI_VERSION.to_string(),
            cache_modules: true,
        }
    }
}

#[derive(Debug, Clone)]
pub struct WebPluginDescriptor {
    pub source: WebPluginSource,
    pub permission_profile: PermissionProfile,
    pub manifest: PluginManifestEnvelope,
    pub exports: BTreeSet<String>,
}

#[derive(Debug, Clone)]
pub struct PreparedWebPlugin {
    pub descriptor: WebPluginDescriptor,
    pub config: WebLoaderConfig,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum WebLoaderError {
    #[error("plugin ABI version mismatch: expected `{expected}`, found `{found}`")]
    AbiVersionMismatch { expected: String, found: String },
    #[error("plugin is missing required export `{0}`")]
    MissingRequiredExport(String),
    #[error("plugin manifest is invalid: {0}")]
    InvalidManifest(String),
}

impl WebPluginDescriptor {
    pub fn validate(&self, expected_abi_version: &str) -> Result<(), WebLoaderError> {
        if self.manifest.abi_version != expected_abi_version {
            return Err(WebLoaderError::AbiVersionMismatch {
                expected: expected_abi_version.to_string(),
                found: self.manifest.abi_version.clone(),
            });
        }

        for export in REQUIRED_EXPORTS {
            if !self.exports.contains(export) {
                return Err(WebLoaderError::MissingRequiredExport(export.to_string()));
            }
        }

        self.manifest
            .manifest
            .validate()
            .map_err(|err| WebLoaderError::InvalidManifest(err.to_string()))?;

        Ok(())
    }
}

pub struct WebPluginLoader {
    config: WebLoaderConfig,
}

impl WebPluginLoader {
    pub fn new(config: WebLoaderConfig) -> Self {
        Self { config }
    }

    pub fn prepare(
        &self,
        descriptor: WebPluginDescriptor,
    ) -> Result<PreparedWebPlugin, WebLoaderError> {
        descriptor.validate(&self.config.expected_abi_version)?;
        Ok(PreparedWebPlugin {
            descriptor,
            config: self.config.clone(),
        })
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use std::sync::Arc;

    use unode::core::permissions::PermissionProfile;
    use unode_plugin_sdk::{
        PluginManifestEnvelope, REQUIRED_EXPORTS, UNODE_PLUGIN_ABI_VERSION, plugin_manifest,
    };

    use super::{
        WebLoaderConfig, WebLoaderError, WebPluginDescriptor, WebPluginLoader, WebPluginSource,
    };

    fn descriptor() -> WebPluginDescriptor {
        WebPluginDescriptor {
            source: WebPluginSource::Bytes(Arc::from([0u8, 97u8].as_slice())),
            permission_profile: PermissionProfile {
                plugin_id: "demo.plugin".to_string(),
                grants: vec![],
            },
            manifest: PluginManifestEnvelope {
                abi_version: UNODE_PLUGIN_ABI_VERSION.to_string(),
                manifest: plugin_manifest("demo.plugin", "Demo").build(),
            },
            exports: REQUIRED_EXPORTS
                .iter()
                .map(|value| value.to_string())
                .collect::<BTreeSet<_>>(),
        }
    }

    #[test]
    fn prepares_plugin_when_required_exports_are_present() {
        let loader = WebPluginLoader::new(WebLoaderConfig::default());
        let prepared = loader.prepare(descriptor()).expect("prepared");
        assert_eq!(prepared.descriptor.manifest.manifest.id, "demo.plugin");
    }

    #[test]
    fn rejects_abi_version_mismatch() {
        let loader = WebPluginLoader::new(WebLoaderConfig::default());
        let mut descriptor = descriptor();
        descriptor.manifest.abi_version = "999.0.0".to_string();

        assert!(matches!(
            loader.prepare(descriptor),
            Err(WebLoaderError::AbiVersionMismatch { .. })
        ));
    }

    #[test]
    fn rejects_missing_render_slot_export() {
        let loader = WebPluginLoader::new(WebLoaderConfig::default());
        let mut descriptor = descriptor();
        descriptor.exports.remove("plugin_render_slot");

        assert!(matches!(
            loader.prepare(descriptor),
            Err(WebLoaderError::MissingRequiredExport(export)) if export == "plugin_render_slot"
        ));
    }
}
