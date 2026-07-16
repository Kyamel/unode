use crate::manifest::PermissionRequestBuilder;

pub use unode::core::permissions::{
    CoreBuiltinPermission, PermissionGrant, PermissionGuard, PermissionProfile, PermissionRequest,
};

pub fn core_permission(permission: CoreBuiltinPermission) -> PermissionRequestBuilder {
    crate::manifest::perm(permission.as_str())
}

pub mod core {
    use super::{CoreBuiltinPermission, PermissionRequestBuilder, core_permission};

    pub fn http_fetch() -> PermissionRequestBuilder {
        core_permission(CoreBuiltinPermission::HttpFetch)
    }

    pub fn http_fetch_any() -> PermissionRequestBuilder {
        core_permission(CoreBuiltinPermission::HttpFetchAny)
    }

    pub fn storage_session_read() -> PermissionRequestBuilder {
        core_permission(CoreBuiltinPermission::StorageSessionRead)
    }

    pub fn storage_session_write() -> PermissionRequestBuilder {
        core_permission(CoreBuiltinPermission::StorageSessionWrite)
    }

    pub fn storage_persistent_read() -> PermissionRequestBuilder {
        core_permission(CoreBuiltinPermission::StoragePersistentRead)
    }

    pub fn storage_persistent_write() -> PermissionRequestBuilder {
        core_permission(CoreBuiltinPermission::StoragePersistentWrite)
    }

    pub fn events_read() -> PermissionRequestBuilder {
        core_permission(CoreBuiltinPermission::EventsRead)
    }

    pub fn events_write() -> PermissionRequestBuilder {
        core_permission(CoreBuiltinPermission::EventsWrite)
    }

    pub fn navigation_read() -> PermissionRequestBuilder {
        core_permission(CoreBuiltinPermission::NavigationRead)
    }

    pub fn navigation_write() -> PermissionRequestBuilder {
        core_permission(CoreBuiltinPermission::NavigationWrite)
    }

    pub fn commands_write() -> PermissionRequestBuilder {
        core_permission(CoreBuiltinPermission::CommandsWrite)
    }

    pub fn feedback_write() -> PermissionRequestBuilder {
        core_permission(CoreBuiltinPermission::FeedbackWrite)
    }

    pub fn system_read() -> PermissionRequestBuilder {
        core_permission(CoreBuiltinPermission::SystemRead)
    }
}
