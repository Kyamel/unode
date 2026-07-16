use serde::{Deserialize, Serialize};
use thiserror::Error;

/// A permission identifier: typed at the authoring edges, a plain string on
/// the wire. The set is open by construction — the core ships its builtins in
/// [`builtin`], and hosts/apps declare their own domain permissions with
/// [`Permission::new`]; the unode core never enumerates or limits them.
///
/// ```
/// use unode::core::permissions::{Permission, builtin};
///
/// // App-defined domain permission (lives in the app's SDK crate):
/// pub const CATALOG_READ: Permission = Permission::new("catalog.read");
///
/// assert_eq!(builtin::HTTP_FETCH.as_str(), "http.fetch");
/// assert_eq!(builtin::STORAGE_SESSION_WRITE.scoped("drafts"), "storage.session.write:drafts");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Permission(&'static str);

impl Permission {
    pub const fn new(name: &'static str) -> Self {
        Self(name)
    }

    pub const fn as_str(&self) -> &'static str {
        self.0
    }

    /// Applies the `resource:scope` convention, e.g.
    /// `STATE_WRITE.scoped("tasks")` → `"state.write:tasks"`.
    pub fn scoped(&self, scope: impl AsRef<str>) -> String {
        format!("{}:{}", self.0, scope.as_ref())
    }
}

impl From<Permission> for String {
    fn from(value: Permission) -> Self {
        value.0.to_string()
    }
}

impl From<CoreBuiltinPermission> for Permission {
    fn from(value: CoreBuiltinPermission) -> Self {
        Permission::new(value.as_str())
    }
}

impl std::fmt::Display for Permission {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.0)
    }
}

/// The core's built-in permissions as typed constants.
pub mod builtin {
    use super::Permission;

    pub const HTTP_FETCH: Permission = Permission::new("http.fetch");
    pub const HTTP_FETCH_ANY: Permission = Permission::new("http.fetch.any");
    pub const STORAGE_SESSION_READ: Permission = Permission::new("storage.session.read");
    pub const STORAGE_SESSION_WRITE: Permission = Permission::new("storage.session.write");
    pub const STORAGE_PERSISTENT_READ: Permission = Permission::new("storage.persistent.read");
    pub const STORAGE_PERSISTENT_WRITE: Permission = Permission::new("storage.persistent.write");
    pub const EVENTS_READ: Permission = Permission::new("events.read");
    pub const EVENTS_WRITE: Permission = Permission::new("events.write");
    pub const NAVIGATION_READ: Permission = Permission::new("navigation.read");
    pub const NAVIGATION_WRITE: Permission = Permission::new("navigation.write");
    pub const COMMANDS_WRITE: Permission = Permission::new("commands.write");
    pub const FEEDBACK_WRITE: Permission = Permission::new("feedback.write");
    pub const SYSTEM_READ: Permission = Permission::new("system.read");
}

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
