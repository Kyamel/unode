use unode::core::permissions::{PermissionError, PermissionGuard};

use crate::runtime::RuntimeTarget;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HostCapability {
    pub key: String,
    pub required_permissions: Vec<String>,
}

impl HostCapability {
    pub fn new(key: impl Into<String>, required_permissions: Vec<String>) -> Self {
        Self {
            key: key.into(),
            required_permissions,
        }
    }
}

pub trait RuntimeHost {
    fn target(&self) -> RuntimeTarget;
    fn host_id(&self) -> Option<&str>;
    fn permission_guard(&self) -> &PermissionGuard;
}

#[derive(Debug, Clone)]
pub struct RuntimeSandbox {
    target: RuntimeTarget,
    guard: PermissionGuard,
}

impl RuntimeSandbox {
    pub fn new(target: RuntimeTarget, guard: PermissionGuard) -> Self {
        Self { target, guard }
    }

    pub fn target(&self) -> RuntimeTarget {
        self.target
    }

    pub fn guard(&self) -> &PermissionGuard {
        &self.guard
    }

    pub fn can_expose(&self, capability: &HostCapability) -> bool {
        capability
            .required_permissions
            .iter()
            .all(|permission| self.guard.has(permission))
    }

    pub fn assert_capability(&self, capability: &HostCapability) -> Result<(), PermissionError> {
        for permission in &capability.required_permissions {
            self.guard.assert(permission)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use unode::core::permissions::{PermissionGrant, PermissionGuard, PermissionProfile};

    use super::{HostCapability, RuntimeSandbox};
    use crate::runtime::RuntimeTarget;

    fn sandbox() -> RuntimeSandbox {
        RuntimeSandbox::new(
            RuntimeTarget::Web,
            PermissionGuard::new(PermissionProfile {
                plugin_id: "demo.plugin".to_string(),
                grants: vec![PermissionGrant {
                    permission: "navigation.write".to_string(),
                    granted: true,
                    granted_at: "2026-04-03T00:00:00Z".to_string(),
                    allowed_origins: vec![],
                }],
            }),
        )
    }

    #[test]
    fn checks_capabilities_through_permission_guard() {
        let capability =
            HostCapability::new("host.navigation", vec!["navigation.write".to_string()]);
        assert!(sandbox().can_expose(&capability));
    }
}
