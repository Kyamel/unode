use unode::core::permissions::PermissionRequest;
use unode::core::runtime::{PluginManifest, UNODE_CORE_API_VERSION};

/// Starts a plugin permission request builder.
///
/// Permission strings are host-defined capabilities such as `http.fetch` or a
/// domain-specific method group. Mark permissions as required when the plugin
/// cannot function without them; optional permissions can be granted later by
/// host policy.
pub fn permission(permission: impl Into<String>) -> PermissionRequestBuilder {
    PermissionRequestBuilder {
        request: PermissionRequest {
            permission: permission.into(),
            required: false,
            reason: None,
            allowed_origins: vec![],
        },
    }
}

/// Starts a plugin manifest builder with the current core API version.
///
/// Plugin authors normally expose the built manifest through the WASM ABI
/// `plugin_manifest` export. Hosts read it before instantiation to validate API
/// compatibility and requested permissions.
pub fn plugin_manifest(id: impl Into<String>, name: impl Into<String>) -> PluginManifestBuilder {
    let mut manifest = PluginManifest::default();
    manifest.id = id.into();
    manifest.name = name.into();
    PluginManifestBuilder { manifest }
}

#[derive(Debug, Clone)]
pub struct PermissionRequestBuilder {
    request: PermissionRequest,
}

impl PermissionRequestBuilder {
    pub fn required(mut self, required: bool) -> Self {
        self.request.required = required;
        self
    }

    pub fn reason(mut self, reason: impl Into<String>) -> Self {
        self.request.reason = Some(reason.into());
        self
    }

    pub fn allow_origin(mut self, origin: impl Into<String>) -> Self {
        self.request.allowed_origins.push(origin.into());
        self
    }

    pub fn allow_origins<I, S>(mut self, origins: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.request
            .allowed_origins
            .extend(origins.into_iter().map(Into::into));
        self
    }

    pub fn build(self) -> PermissionRequest {
        self.request
    }
}

impl From<PermissionRequestBuilder> for PermissionRequest {
    fn from(value: PermissionRequestBuilder) -> Self {
        value.build()
    }
}

#[derive(Debug, Clone)]
pub struct PluginManifestBuilder {
    manifest: PluginManifest,
}

impl PluginManifestBuilder {
    pub fn version(mut self, version: impl Into<String>) -> Self {
        self.manifest.version = version.into();
        self
    }

    pub fn api_version(mut self, api_version: impl Into<String>) -> Self {
        self.manifest.api_version = api_version.into();
        self
    }

    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.manifest.description = Some(description.into());
        self
    }

    pub fn author(mut self, author: impl Into<String>) -> Self {
        self.manifest.author = Some(author.into());
        self
    }

    pub fn require(mut self, dependency: impl Into<String>) -> Self {
        self.manifest.requires.push(dependency.into());
        self
    }

    pub fn permission(mut self, permission: impl Into<PermissionRequest>) -> Self {
        self.manifest.permissions.push(permission.into());
        self
    }

    pub fn permissions<I, P>(mut self, permissions: I) -> Self
    where
        I: IntoIterator<Item = P>,
        P: Into<PermissionRequest>,
    {
        self.manifest
            .permissions
            .extend(permissions.into_iter().map(Into::into));
        self
    }

    pub fn host_id(mut self, host_id: impl Into<String>) -> Self {
        self.manifest.host_id = Some(host_id.into());
        self
    }

    pub fn build(self) -> PluginManifest {
        self.manifest
    }
}

impl Default for PluginManifestBuilder {
    fn default() -> Self {
        Self {
            manifest: PluginManifest {
                api_version: UNODE_CORE_API_VERSION.to_string(),
                ..PluginManifest::default()
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{permission, plugin_manifest};

    #[test]
    fn builds_manifest_with_permissions() {
        let manifest = plugin_manifest("demo.plugin", "Demo")
            .version("1.2.3")
            .description("demo plugin")
            .author("Lucas")
            .require("catalog.read")
            .permission(
                permission("http.fetch")
                    .required(true)
                    .reason("load remote data")
                    .allow_origin("https://api.example.com"),
            )
            .build();

        assert_eq!(manifest.id, "demo.plugin");
        assert_eq!(manifest.name, "Demo");
        assert_eq!(manifest.version, "1.2.3");
        assert_eq!(manifest.permissions.len(), 1);
        assert!(manifest.permissions[0].required);
        assert_eq!(
            manifest.permissions[0].allowed_origins,
            vec!["https://api.example.com".to_string()]
        );
        assert_eq!(manifest.requires, vec!["catalog.read".to_string()]);
    }
}
