use std::fs;
use std::path::Path;
use std::sync::{Arc, Mutex};

use thiserror::Error;
use wasmtime::{Caller, Config, Engine, Extern, Instance, Linker, Memory, Module, Store, TypedFunc};

use unode_sdk::{
    EXPORT_PLUGIN_DISPATCH, EXPORT_PLUGIN_DISPATCH_RESULT_LEN,
    EXPORT_PLUGIN_LOAD, EXPORT_PLUGIN_LOAD_RESULT_LEN, EXPORT_PLUGIN_MANIFEST,
    EXPORT_PLUGIN_MANIFEST_LEN, EXPORT_PLUGIN_RENDER, EXPORT_PLUGIN_RENDER_RESULT_LEN,
    EXPORT_UNODE_ALLOC, EXPORT_UNODE_DEALLOC, IMPORT_HOST_CALL, IMPORT_HOST_CALL_RESULT_LEN,
};

use crate::bridge::{TuiAbiBridgeError, TuiGuestInstance, TuiHostImportAdapter, TuiPluginBridge};
use crate::host_call::{TuiHostCallDispatcher, TuiHostCallError};
use crate::loader::{PreparedTuiPlugin, TuiPluginSource};
use crate::memory::{read_bytes, TuiMemoryError};

#[derive(Debug)]
struct WasmtimeStoreState {
    last_host_result_len: u32,
}

pub struct WasmtimeGuest {
    _engine: Engine,
    store: Store<WasmtimeStoreState>,
    _module: Module,
    _instance: Instance,
    memory: Memory,
    alloc: TypedFunc<u32, u32>,
    dealloc: TypedFunc<(u32, u32), ()>,
    plugin_manifest: TypedFunc<(), u32>,
    plugin_manifest_len: TypedFunc<(), u32>,
    plugin_load: TypedFunc<(u32, u32), u32>,
    plugin_load_result_len: TypedFunc<(), u32>,
    plugin_render: TypedFunc<(u32, u32), u32>,
    plugin_render_result_len: TypedFunc<(), u32>,
    plugin_dispatch: TypedFunc<(u32, u32), u32>,
    plugin_dispatch_result_len: TypedFunc<(), u32>,
}

#[derive(Debug, Error)]
pub enum WasmtimeGuestError {
    #[error(transparent)]
    Wasmtime(#[from] wasmtime::Error),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("module does not export memory")]
    MissingMemory,
    #[error("host dispatcher mutex poisoned")]
    DispatcherPoisoned,
    #[error("missing export `{0}`")]
    MissingExport(String),
    #[error(transparent)]
    HostCall(#[from] TuiHostCallError),
    #[error(transparent)]
    Memory(#[from] TuiMemoryError),
}

impl From<WasmtimeGuestError> for TuiAbiBridgeError {
    fn from(value: WasmtimeGuestError) -> Self {
        TuiAbiBridgeError::Guest(value.to_string())
    }
}

impl WasmtimeGuest {
    pub fn compile_prepared_plugin(
        prepared: &PreparedTuiPlugin,
    ) -> Result<CompiledWasmtimePlugin, WasmtimeGuestError> {
        let wasm_bytes = match &prepared.descriptor.source {
            TuiPluginSource::File(path) => fs::read(path)?,
            TuiPluginSource::Bytes(bytes) => bytes.as_ref().to_vec(),
        };

        let mut config = Config::new();
        config.consume_fuel(prepared.config.enable_fuel_metering);

        let engine = Engine::new(&config)?;
        let module = Module::from_binary(&engine, &wasm_bytes)?;

        Ok(CompiledWasmtimePlugin {
            engine,
            module,
            enable_fuel_metering: prepared.config.enable_fuel_metering,
        })
    }

    pub fn compile_wasm_file(path: impl AsRef<Path>) -> Result<CompiledWasmtimePlugin, WasmtimeGuestError> {
        let prepared = PreparedTuiPlugin {
            descriptor: crate::loader::TuiPluginDescriptor {
                source: TuiPluginSource::File(path.as_ref().to_path_buf()),
                permission_profile: unode::core::permissions::PermissionProfile {
                    plugin_id: "external.plugin".to_string(),
                    grants: vec![],
                },
                manifest: unode_sdk::PluginManifestEnvelope {
                    abi_version: unode_sdk::UNODE_PLUGIN_ABI_VERSION.to_string(),
                    manifest: unode::core::runtime::PluginManifest::default(),
                },
                exports: Default::default(),
            },
            config: crate::loader::TuiLoaderConfig::default(),
        };
        Self::compile_prepared_plugin(&prepared)
    }

    pub fn from_prepared_plugin(
        prepared: &PreparedTuiPlugin,
        dispatcher: TuiHostCallDispatcher,
    ) -> Result<TuiPluginBridge<Self>, WasmtimeGuestError> {
        let compiled = Self::compile_prepared_plugin(prepared)?;
        compiled.instantiate(dispatcher)
    }

    pub fn from_wasm_file(
        path: impl AsRef<Path>,
        dispatcher: TuiHostCallDispatcher,
    ) -> Result<TuiPluginBridge<Self>, WasmtimeGuestError> {
        let compiled = Self::compile_wasm_file(path)?;
        compiled.instantiate(dispatcher)
    }
}

pub struct CompiledWasmtimePlugin {
    engine: Engine,
    module: Module,
    enable_fuel_metering: bool,
}

impl std::fmt::Debug for CompiledWasmtimePlugin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CompiledWasmtimePlugin")
            .field("enable_fuel_metering", &self.enable_fuel_metering)
            .finish_non_exhaustive()
    }
}

impl CompiledWasmtimePlugin {
    pub fn instantiate(
        &self,
        dispatcher: TuiHostCallDispatcher,
    ) -> Result<TuiPluginBridge<WasmtimeGuest>, WasmtimeGuestError> {
        let dispatcher = Arc::new(Mutex::new(dispatcher));

        let mut linker = Linker::new(&self.engine);
        {
            let dispatcher = dispatcher.clone();
            linker.func_wrap(
                "unode",
                IMPORT_HOST_CALL,
                move |mut caller: Caller<'_, WasmtimeStoreState>, request_ptr: u32, request_len: u32| -> Result<u32, wasmtime::Error> {
                    let memory = get_memory(&mut caller)?;
                    let request_bytes = read_memory_from_caller(&caller, &memory, request_ptr, request_len)?;

                    let response_bytes = {
                        let mut locked = dispatcher.lock().map_err(|_| wasmtime::Error::msg("dispatcher mutex poisoned"))?;
                        locked
                            .dispatch_bytes(&request_bytes)
                            .map(|bytes| bytes.to_vec())
                            .map_err(|err| wasmtime::Error::msg(err.to_string()))?
                    };

                    let alloc = get_typed_export::<u32, u32>(&mut caller, EXPORT_UNODE_ALLOC)?;
                    let response_ptr = alloc.call(&mut caller, response_bytes.len() as u32)?;
                    memory
                        .write(&mut caller, response_ptr as usize, &response_bytes)
                        .map_err(|err| wasmtime::Error::msg(err.to_string()))?;
                    caller.data_mut().last_host_result_len = response_bytes.len() as u32;
                    Ok(response_ptr)
                },
            )?;
        }
        linker.func_wrap(
            "unode",
            IMPORT_HOST_CALL_RESULT_LEN,
            |caller: Caller<'_, WasmtimeStoreState>| -> u32 { caller.data().last_host_result_len },
        )?;

        let mut store = Store::new(
            &self.engine,
            WasmtimeStoreState {
                last_host_result_len: 0,
            },
        );
        if self.enable_fuel_metering {
            store.set_fuel(10_000_000)?;
        }

        let instance = linker.instantiate(&mut store, &self.module)?;
        let memory = instance
            .get_memory(&mut store, "memory")
            .ok_or(WasmtimeGuestError::MissingMemory)?;

        let alloc = instance.get_typed_func::<u32, u32>(&mut store, EXPORT_UNODE_ALLOC)?;
        let dealloc =
            instance.get_typed_func::<(u32, u32), ()>(&mut store, EXPORT_UNODE_DEALLOC)?;
        let plugin_manifest =
            instance.get_typed_func::<(), u32>(&mut store, EXPORT_PLUGIN_MANIFEST)?;
        let plugin_manifest_len =
            instance.get_typed_func::<(), u32>(&mut store, EXPORT_PLUGIN_MANIFEST_LEN)?;
        let plugin_load =
            instance.get_typed_func::<(u32, u32), u32>(&mut store, EXPORT_PLUGIN_LOAD)?;
        let plugin_load_result_len =
            instance.get_typed_func::<(), u32>(&mut store, EXPORT_PLUGIN_LOAD_RESULT_LEN)?;
        let plugin_render =
            instance.get_typed_func::<(u32, u32), u32>(&mut store, EXPORT_PLUGIN_RENDER)?;
        let plugin_render_result_len =
            instance.get_typed_func::<(), u32>(&mut store, EXPORT_PLUGIN_RENDER_RESULT_LEN)?;
        let plugin_dispatch =
            instance.get_typed_func::<(u32, u32), u32>(&mut store, EXPORT_PLUGIN_DISPATCH)?;
        let plugin_dispatch_result_len =
            instance.get_typed_func::<(), u32>(&mut store, EXPORT_PLUGIN_DISPATCH_RESULT_LEN)?;

        let guest = WasmtimeGuest {
            _engine: self.engine.clone(),
            store,
            _module: self.module.clone(),
            _instance: instance,
            memory,
            alloc,
            dealloc,
            plugin_manifest,
            plugin_manifest_len,
            plugin_load,
            plugin_load_result_len,
            plugin_render,
            plugin_render_result_len,
            plugin_dispatch,
            plugin_dispatch_result_len,
        };

        Ok(TuiPluginBridge::new(
            guest,
            TuiHostImportAdapter::new(
                dispatcher
                    .lock()
                    .map_err(|_| WasmtimeGuestError::DispatcherPoisoned)?
                    .clone(),
            ),
        ))
    }
}

impl TuiGuestInstance for WasmtimeGuest {
    fn read_memory(&self, ptr: u32, len: u32) -> Result<Vec<u8>, TuiAbiBridgeError> {
        read_bytes(self.memory.data(&self.store), ptr, len).map_err(TuiAbiBridgeError::from)
    }

    fn write_memory(&mut self, ptr: u32, bytes: &[u8]) -> Result<(), TuiAbiBridgeError> {
        self.memory
            .write(&mut self.store, ptr as usize, bytes)
            .map_err(|err| TuiAbiBridgeError::Guest(err.to_string()))
    }

    fn alloc(&mut self, len: u32) -> Result<u32, TuiAbiBridgeError> {
        self.alloc.call(&mut self.store, len).map_err(|err| TuiAbiBridgeError::Guest(err.to_string()))
    }

    fn dealloc(&mut self, ptr: u32, len: u32) -> Result<(), TuiAbiBridgeError> {
        self.dealloc
            .call(&mut self.store, (ptr, len))
            .map_err(|err| TuiAbiBridgeError::Guest(err.to_string()))
    }

    fn plugin_manifest(&mut self) -> Result<u32, TuiAbiBridgeError> {
        self.plugin_manifest
            .call(&mut self.store, ())
            .map_err(|err| TuiAbiBridgeError::Guest(err.to_string()))
    }

    fn plugin_manifest_len(&mut self) -> Result<u32, TuiAbiBridgeError> {
        self.plugin_manifest_len
            .call(&mut self.store, ())
            .map_err(|err| TuiAbiBridgeError::Guest(err.to_string()))
    }

    fn plugin_render(&mut self, request_ptr: u32, request_len: u32) -> Result<u32, TuiAbiBridgeError> {
        self.plugin_render
            .call(&mut self.store, (request_ptr, request_len))
            .map_err(|err| TuiAbiBridgeError::Guest(err.to_string()))
    }

    fn plugin_load(&mut self, request_ptr: u32, request_len: u32) -> Result<u32, TuiAbiBridgeError> {
        self.plugin_load
            .call(&mut self.store, (request_ptr, request_len))
            .map_err(|err| TuiAbiBridgeError::Guest(err.to_string()))
    }

    fn plugin_load_result_len(&mut self) -> Result<u32, TuiAbiBridgeError> {
        self.plugin_load_result_len
            .call(&mut self.store, ())
            .map_err(|err| TuiAbiBridgeError::Guest(err.to_string()))
    }

    fn plugin_render_result_len(&mut self) -> Result<u32, TuiAbiBridgeError> {
        self.plugin_render_result_len
            .call(&mut self.store, ())
            .map_err(|err| TuiAbiBridgeError::Guest(err.to_string()))
    }

    fn plugin_dispatch(&mut self, request_ptr: u32, request_len: u32) -> Result<u32, TuiAbiBridgeError> {
        self.plugin_dispatch
            .call(&mut self.store, (request_ptr, request_len))
            .map_err(|err| TuiAbiBridgeError::Guest(err.to_string()))
    }

    fn plugin_dispatch_result_len(&mut self) -> Result<u32, TuiAbiBridgeError> {
        self.plugin_dispatch_result_len
            .call(&mut self.store, ())
            .map_err(|err| TuiAbiBridgeError::Guest(err.to_string()))
    }
}

fn get_memory(caller: &mut Caller<'_, WasmtimeStoreState>) -> Result<Memory, wasmtime::Error> {
    caller
        .get_export("memory")
        .and_then(Extern::into_memory)
        .ok_or_else(|| wasmtime::Error::msg("missing memory export"))
}

fn get_typed_export<Params, Results>(
    caller: &mut Caller<'_, WasmtimeStoreState>,
    name: &str,
) -> Result<TypedFunc<Params, Results>, wasmtime::Error>
where
    Params: wasmtime::WasmParams,
    Results: wasmtime::WasmResults,
{
    caller
        .get_export(name)
        .and_then(Extern::into_func)
        .ok_or_else(|| wasmtime::Error::msg(format!("missing export `{name}`")))?
        .typed::<Params, Results>(&mut *caller)
}

fn read_memory_from_caller(
    caller: &Caller<'_, WasmtimeStoreState>,
    memory: &Memory,
    ptr: u32,
    len: u32,
) -> Result<Vec<u8>, wasmtime::Error> {
    let mut buf = vec![0; len as usize];
    memory
        .read(caller, ptr as usize, &mut buf)
        .map_err(|err| wasmtime::Error::msg(err.to_string()))?;
    Ok(buf)
}
