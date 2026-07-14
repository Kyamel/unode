use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoreBuiltinPermission {
    HttpFetch,
    HttpFetchAny,
    StorageSessionRead,
    StorageSessionWrite,
    StoragePersistentRead,
    StoragePersistentWrite,
    EventsRead,
    EventsWrite,
    NavigationRead,
    NavigationWrite,
    CommandsWrite,
    FeedbackWrite,
    SystemRead,
}

impl CoreBuiltinPermission {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::HttpFetch => "http.fetch",
            Self::HttpFetchAny => "http.fetch.any",
            Self::StorageSessionRead => "storage.session.read",
            Self::StorageSessionWrite => "storage.session.write",
            Self::StoragePersistentRead => "storage.persistent.read",
            Self::StoragePersistentWrite => "storage.persistent.write",
            Self::EventsRead => "events.read",
            Self::EventsWrite => "events.write",
            Self::NavigationRead => "navigation.read",
            Self::NavigationWrite => "navigation.write",
            Self::CommandsWrite => "commands.write",
            Self::FeedbackWrite => "feedback.write",
            Self::SystemRead => "system.read",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PermissionRequest {
    pub permission: String,
    #[serde(default)]
    pub required: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub allowed_origins: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PermissionGrant {
    pub permission: String,
    pub granted: bool,
    pub granted_at: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub allowed_origins: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PermissionProfile {
    pub plugin_id: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub grants: Vec<PermissionGrant>,
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum PermissionError {
    #[error("permission denied for plugin `{plugin_id}`: `{permission}`")]
    PermissionDenied {
        plugin_id: String,
        permission: String,
    },
    #[error("origin not allowed for plugin `{plugin_id}`: `{url}`")]
    OriginNotAllowed { plugin_id: String, url: String },
}

#[derive(Debug, Clone)]
pub struct PermissionGuard {
    profile: PermissionProfile,
}

impl PermissionGuard {
    pub fn new(profile: PermissionProfile) -> Self {
        Self { profile }
    }

    pub fn profile(&self) -> &PermissionProfile {
        &self.profile
    }

    pub fn has(&self, permission: &str) -> bool {
        self.profile
            .grants
            .iter()
            .find(|grant| grant.permission == permission)
            .map(|grant| grant.granted)
            .unwrap_or(false)
    }

    pub fn assert(&self, permission: &str) -> Result<(), PermissionError> {
        if self.has(permission) {
            Ok(())
        } else {
            Err(PermissionError::PermissionDenied {
                plugin_id: self.profile.plugin_id.clone(),
                permission: permission.to_string(),
            })
        }
    }

    pub fn approved_origins(&self, permission: &str) -> &[String] {
        self.profile
            .grants
            .iter()
            .find(|grant| grant.permission == permission)
            .map(|grant| grant.allowed_origins.as_slice())
            .unwrap_or(&[])
    }

    pub fn assert_origin(&self, url: &str) -> Result<(), PermissionError> {
        if self.has(CoreBuiltinPermission::HttpFetchAny.as_str()) {
            return Ok(());
        }

        self.assert(CoreBuiltinPermission::HttpFetch.as_str())?;

        let origin = extract_origin(url).unwrap_or(url);
        let allowed = self.approved_origins(CoreBuiltinPermission::HttpFetch.as_str());

        if allowed
            .iter()
            .any(|candidate| candidate == "*" || candidate == origin)
        {
            Ok(())
        } else {
            Err(PermissionError::OriginNotAllowed {
                plugin_id: self.profile.plugin_id.clone(),
                url: url.to_string(),
            })
        }
    }
}

fn extract_origin(url: &str) -> Option<&str> {
    let scheme_end = url.find("://")?;
    let after_scheme = &url[(scheme_end + 3)..];
    let path_start = after_scheme
        .find(&['/', '?', '#'][..])
        .unwrap_or(after_scheme.len());
    Some(&url[..scheme_end + 3 + path_start])
}

#[cfg(test)]
mod tests {
    use super::{
        CoreBuiltinPermission, PermissionError, PermissionGrant, PermissionGuard, PermissionProfile,
    };

    fn guard(grants: Vec<PermissionGrant>) -> PermissionGuard {
        PermissionGuard::new(PermissionProfile {
            plugin_id: "plugin.test".to_string(),
            grants,
        })
    }

    #[test]
    fn denies_missing_permissions_by_default() {
        let guard = guard(vec![]);
        assert!(matches!(
            guard.assert("catalog.read"),
            Err(PermissionError::PermissionDenied { .. })
        ));
    }

    #[test]
    fn accepts_allowed_http_origin() {
        let guard = guard(vec![PermissionGrant {
            permission: CoreBuiltinPermission::HttpFetch.as_str().to_string(),
            granted: true,
            granted_at: "2026-04-03T00:00:00Z".to_string(),
            allowed_origins: vec!["https://api.example.com".to_string()],
        }]);

        assert!(
            guard
                .assert_origin("https://api.example.com/works/1")
                .is_ok()
        );
        assert!(
            guard
                .assert_origin("https://evil.example.com/works/1")
                .is_err()
        );
    }

    #[test]
    fn http_fetch_any_bypasses_origin_checks() {
        let guard = guard(vec![PermissionGrant {
            permission: CoreBuiltinPermission::HttpFetchAny.as_str().to_string(),
            granted: true,
            granted_at: "2026-04-03T00:00:00Z".to_string(),
            allowed_origins: vec![],
        }]);

        assert!(
            guard
                .assert_origin("https://wherever.example.com/works/1")
                .is_ok()
        );
    }
}
