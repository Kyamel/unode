use std::path::Path;

use serde::de::DeserializeOwned;
use serde_json::Value as JsonValue;
use thiserror::Error;
use unode_sdk::{
    PluginDispatchRequest, PluginLoadRequest, PluginManifestEnvelope, PluginRenderRequest,
    PluginRenderSlotRequest,
};

use crate::bridge::{TuiAbiBridgeError, TuiPluginBridge};
use crate::host_call::TuiHostCallDispatcher;
use crate::wasmtime_guest::{CompiledWasmtimePlugin, WasmtimeGuest, WasmtimeGuestError};

#[derive(Debug, Error)]
pub enum TuiPluginRuntimeError {
    #[error(transparent)]
    Guest(#[from] WasmtimeGuestError),
    #[error(transparent)]
    Bridge(#[from] TuiAbiBridgeError),
}

#[derive(Debug)]
pub struct CachedTuiPlugin {
    compiled: CompiledWasmtimePlugin,
    dispatcher: TuiHostCallDispatcher,
    manifest: PluginManifestEnvelope,
}

impl CachedTuiPlugin {
    pub fn from_wasm_file(
        path: impl AsRef<Path>,
        dispatcher: TuiHostCallDispatcher,
    ) -> Result<Self, TuiPluginRuntimeError> {
        let compiled = WasmtimeGuest::compile_wasm_file(path)?;
        let manifest = {
            let mut bridge = compiled.instantiate(dispatcher.clone())?;
            bridge.call_plugin_manifest()?
        };

        Ok(Self {
            compiled,
            dispatcher,
            manifest,
        })
    }

    pub fn manifest(&self) -> &PluginManifestEnvelope {
        &self.manifest
    }

    pub fn start_session(
        &self,
        request: &PluginLoadRequest,
    ) -> Result<PluginSession, TuiPluginRuntimeError> {
        let mut bridge = self.compiled.instantiate(self.dispatcher.clone())?;
        bridge.call_plugin_load::<_, JsonValue>(request)?;
        Ok(PluginSession { bridge })
    }
}

pub struct PluginSession {
    bridge: TuiPluginBridge<WasmtimeGuest>,
}

impl PluginSession {
    pub fn render<Resp>(
        &mut self,
        request: &PluginRenderRequest,
    ) -> Result<Resp, TuiPluginRuntimeError>
    where
        Resp: DeserializeOwned,
    {
        self.bridge.call_plugin_render(request).map_err(Into::into)
    }

    pub fn dispatch<Resp>(
        &mut self,
        request: &PluginDispatchRequest,
    ) -> Result<Resp, TuiPluginRuntimeError>
    where
        Resp: DeserializeOwned,
    {
        self.bridge
            .call_plugin_dispatch(request)
            .map_err(Into::into)
    }

    pub fn render_slot<Resp>(
        &mut self,
        request: &PluginRenderSlotRequest,
    ) -> Result<Resp, TuiPluginRuntimeError>
    where
        Resp: DeserializeOwned,
    {
        self.bridge
            .call_plugin_render_slot(request)
            .map_err(Into::into)
    }

    pub fn bridge(&self) -> &TuiPluginBridge<WasmtimeGuest> {
        &self.bridge
    }

    pub fn bridge_mut(&mut self) -> &mut TuiPluginBridge<WasmtimeGuest> {
        &mut self.bridge
    }
}
