pub mod bridge;
pub mod host_call;
pub mod loader;
pub mod memory;
pub mod session;
pub mod wasmtime_guest;

use unode::core::permissions::PermissionProfile;
use unode_runtime::{HostedRuntime, RuntimeTarget};

pub struct TuiRuntime<Ctx> {
    pub inner: HostedRuntime<Ctx>,
}

impl<Ctx> TuiRuntime<Ctx> {
    pub fn new(profile: PermissionProfile) -> Self {
        Self {
            inner: HostedRuntime::new(
                RuntimeTarget::Tui,
                unode::core::permissions::PermissionGuard::new(profile),
            ),
        }
    }
}

pub use bridge::{TuiAbiBridgeError, TuiGuestInstance, TuiHostImportAdapter, TuiPluginBridge};
pub use host_call::{TuiHostCallDispatcher, TuiHostCallError};
pub use loader::{PreparedTuiPlugin, TuiLoaderConfig, TuiLoaderError, TuiPluginDescriptor, TuiPluginLoader, TuiPluginSource};
pub use memory::{read_bytes, read_json, write_bytes, write_json, TuiMemoryError};
pub use session::{CachedTuiPlugin, PluginSession, TuiPluginRuntimeError};
pub use wasmtime_guest::{CompiledWasmtimePlugin, WasmtimeGuest, WasmtimeGuestError};
