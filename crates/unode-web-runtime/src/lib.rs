pub mod bridge;
pub mod host_call;
pub mod loader;
pub mod memory;

use unode::core::permissions::PermissionProfile;
use unode_runtime::{HostedRuntime, RuntimeTarget};

pub struct WebRuntime<Ctx> {
    pub inner: HostedRuntime<Ctx>,
}

impl<Ctx> WebRuntime<Ctx> {
    pub fn new(profile: PermissionProfile) -> Self {
        Self {
            inner: HostedRuntime::new(
                RuntimeTarget::Web,
                unode::core::permissions::PermissionGuard::new(profile),
            ),
        }
    }
}

pub use bridge::{WebAbiBridgeError, WebGuestInstance, WebHostImportAdapter, WebPluginBridge};
pub use host_call::{WebHostCallDispatcher, WebHostCallError};
pub use loader::{PreparedWebPlugin, WebLoaderConfig, WebLoaderError, WebPluginDescriptor, WebPluginLoader, WebPluginSource};
pub use memory::{read_bytes, read_json, write_bytes, write_json, WebMemoryError};
