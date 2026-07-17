pub mod bridge;
pub mod component_guest;
pub mod host_call;
pub mod loader;
pub mod memory;
pub mod session;
pub mod state;
pub mod wasmtime_guest;

pub use component_guest::ComponentTuiPlugin;
pub use state::{PluginState, resolve_screen_state};
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
pub use loader::{
    PreparedTuiPlugin, TuiLoaderConfig, TuiLoaderError, TuiPluginDescriptor, TuiPluginLoader,
    TuiPluginSource,
};
pub use memory::{TuiMemoryError, read_bytes, read_json, write_bytes, write_json};
pub use session::{CachedTuiPlugin, PluginSession, TuiPluginRuntimeError};
pub use wasmtime_guest::{CompiledWasmtimePlugin, WasmtimeGuest, WasmtimeGuestError};
