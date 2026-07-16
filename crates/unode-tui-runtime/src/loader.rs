use std::collections::BTreeSet;
use std::path::PathBuf;
use std::sync::Arc;

use thiserror::Error;
use unode::core::permissions::PermissionProfile;
use unode_sdk::{PluginManifestEnvelope, REQUIRED_EXPORTS, UNODE_PLUGIN_ABI_VERSION};

#[derive(Debug, Clone)]
pub enum TuiPluginSource {
    File(PathBuf),
    Bytes(Arc<[u8]>),
}

#[derive(Debug, Clone)]
pub struct TuiLoaderConfig {
    pub expected_abi_version: String,
    pub cache_modules: bool,
    pub enable_fuel_metering: bool,
}

impl Default for TuiLoaderConfig {
    fn default() -> Self {
        Self {
            expected_abi_version: UNODE_PLUGIN_ABI_VERSION.to_string(),
            cache_modules: true,
            enable_fuel_metering: true,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TuiPluginDescriptor {
    pub source: TuiPluginSource,
    pub permission_profile: PermissionProfile,
    pub manifest: PluginManifestEnvelope,
    pub exports: BTreeSet<String>,
}

#[derive(Debug, Clone)]
pub struct PreparedTuiPlugin {
    pub descriptor: TuiPluginDescriptor,
    pub config: TuiLoaderConfig,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum TuiLoaderError {
    #[error("plugin ABI version mismatch: expected `{expected}`, found `{found}`")]
    AbiVersionMismatch { expected: String, found: String },
    #[error("plugin is missing required export `{0}`")]
    MissingRequiredExport(String),
    #[error("plugin manifest is invalid: {0}")]
    InvalidManifest(String),
}

impl TuiPluginDescriptor {
    pub fn validate(&self, expected_abi_version: &str) -> Result<(), TuiLoaderError> {
        if self.manifest.abi_version != expected_abi_version {
            return Err(TuiLoaderError::AbiVersionMismatch {
                expected: expected_abi_version.to_string(),
                found: self.manifest.abi_version.clone(),
            });
        }

        for export in REQUIRED_EXPORTS {
            if !self.exports.contains(export) {
                return Err(TuiLoaderError::MissingRequiredExport(export.to_string()));
            }
        }

        self.manifest
            .manifest
            .validate()
            .map_err(|err| TuiLoaderError::InvalidManifest(err.to_string()))?;

        Ok(())
    }
}

pub struct TuiPluginLoader {
    config: TuiLoaderConfig,
}

impl TuiPluginLoader {
    pub fn new(config: TuiLoaderConfig) -> Self {
        Self { config }
    }

    pub fn prepare(
        &self,
        descriptor: TuiPluginDescriptor,
    ) -> Result<PreparedTuiPlugin, TuiLoaderError> {
        descriptor.validate(&self.config.expected_abi_version)?;
        Ok(PreparedTuiPlugin {
            descriptor,
            config: self.config.clone(),
        })
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use std::path::PathBuf;

    use unode::core::permissions::PermissionProfile;
    use unode_sdk::{
        PluginManifestEnvelope, REQUIRED_EXPORTS, UNODE_PLUGIN_ABI_VERSION, plugin_manifest,
    };

    use super::{
        TuiLoaderConfig, TuiLoaderError, TuiPluginDescriptor, TuiPluginLoader, TuiPluginSource,
    };

    fn descriptor() -> TuiPluginDescriptor {
        TuiPluginDescriptor {
            source: TuiPluginSource::File(PathBuf::from("/plugins/demo.plugin.wasm")),
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
    fn prepares_tui_plugin_when_exports_are_present() {
        let loader = TuiPluginLoader::new(TuiLoaderConfig::default());
        let prepared = loader.prepare(descriptor()).expect("prepared");
        assert_eq!(prepared.descriptor.manifest.manifest.id, "demo.plugin");
    }

    #[test]
    fn rejects_missing_required_export() {
        let loader = TuiPluginLoader::new(TuiLoaderConfig::default());
        let mut descriptor = descriptor();
        descriptor.exports.remove("plugin_render");

        assert!(matches!(
            loader.prepare(descriptor),
            Err(TuiLoaderError::MissingRequiredExport(export)) if export == "plugin_render"
        ));
    }

    #[test]
    fn rejects_missing_render_slot_export() {
        let loader = TuiPluginLoader::new(TuiLoaderConfig::default());
        let mut descriptor = descriptor();
        descriptor.exports.remove("plugin_render_slot");

        assert!(matches!(
            loader.prepare(descriptor),
            Err(TuiLoaderError::MissingRequiredExport(export)) if export == "plugin_render_slot"
        ));
    }

    #[test]
    fn rejects_old_abi_version() {
        let loader = TuiPluginLoader::new(TuiLoaderConfig::default());
        let mut descriptor = descriptor();
        descriptor.manifest.abi_version = "0.1.0".to_string();

        assert!(matches!(
            loader.prepare(descriptor),
            Err(TuiLoaderError::AbiVersionMismatch { found, .. }) if found == "0.1.0"
        ));
    }
}
