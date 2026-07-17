//! Component Model host path — the typed twin of [`crate::wasmtime_guest`].
//!
//! Loads a plugin compiled against `wit/unode-plugin.wit`, wires the typed
//! capability imports (`state`, `navigation`, plus the generic `host.call`
//! escape hatch) into the same [`TuiHostCallDispatcher`] the raw ABI uses,
//! and converts the typed WIT records back into the canonical serde types.
//! The golden rule: for the same plugin source, this path and the raw
//! pointer/length ABI must produce identical JSON.

use std::collections::BTreeMap;
use std::path::Path;

use serde_json::{Value as JsonValue, json};
use unode_plugin_sdk::prelude::{
    NavIntent, PluginManifest, ResolvedRoute, RouteDecl, RouteGroupDecl, SlotContributionDecl,
};
use unode_plugin_sdk::{
    HostCallEnvelope, PluginDispatchOutcome, PluginDispatchRequest, PluginDispatchResponse,
    PluginLoadRequest, PluginManifestEnvelope, PluginRenderRequest,
};
use wasmtime::component::{Component, Linker};
use wasmtime::{Engine, Store};

use crate::host_call::TuiHostCallDispatcher;

mod bindings {
    use wasmtime::component::bindgen;

    bindgen!({
        path: "../../wit",
        world: "unode-plugin",
    });
}

use bindings::UnodePlugin;
use bindings::unode::plugin::types as wit;
use unode_plugin_sdk::prelude::{ActionType, BoolOrExpr, OneOrExpr, StringOrExpr, UiExpr};

/// Store data backing the typed capability imports.
struct HostState {
    dispatcher: TuiHostCallDispatcher,
}

impl HostState {
    fn dispatch(&mut self, operation: &str, params: BTreeMap<String, JsonValue>) {
        let envelope = HostCallEnvelope {
            operation: operation.to_string(),
            params,
        };
        // Capability checks and side effects live in the host dispatcher —
        // identical behavior to the raw ABI's `unode.host_call` import.
        let _ = self.dispatcher.dispatch_envelope(&envelope);
    }
}

impl bindings::unode::plugin::state::Host for HostState {
    fn set(&mut self, path: String, value_json: String) {
        let value = serde_json::from_str(&value_json).unwrap_or(JsonValue::Null);
        self.dispatch(
            "state.set",
            BTreeMap::from([
                ("path".to_string(), json!(path)),
                ("value".to_string(), value),
            ]),
        );
    }
}

impl bindings::unode::plugin::navigation::Host for HostState {
    fn navigate(&mut self, to: String) {
        self.dispatch(
            "navigation.navigate",
            BTreeMap::from([("to".to_string(), json!(to))]),
        );
    }
}

impl bindings::unode::plugin::host::Host for HostState {
    fn call(&mut self, operation: String, params_json: String) -> Result<String, wit::PluginError> {
        let params: BTreeMap<String, JsonValue> =
            serde_json::from_str(&params_json).unwrap_or_default();
        let envelope = HostCallEnvelope { operation, params };
        self.dispatcher
            .dispatch_envelope(&envelope)
            .map(|bytes| String::from_utf8_lossy(bytes).into_owned())
            .map_err(|err| wit::PluginError {
                message: err.to_string(),
                details_json: None,
            })
    }
}

impl bindings::unode::plugin::types::Host for HostState {}

/// A plugin loaded through the Component Model boundary.
pub struct ComponentTuiPlugin {
    store: Store<HostState>,
    plugin: UnodePlugin,
    manifest: PluginManifestEnvelope,
}

impl ComponentTuiPlugin {
    pub fn from_wasm_file(
        path: impl AsRef<Path>,
        dispatcher: TuiHostCallDispatcher,
    ) -> wasmtime::Result<Self> {
        let engine = Engine::default();
        let component = Component::from_file(&engine, path)?;

        let mut linker = Linker::new(&engine);
        UnodePlugin::add_to_linker::<_, wasmtime::component::HasSelf<_>>(
            &mut linker,
            |state: &mut HostState| state,
        )?;

        let mut store = Store::new(&engine, HostState { dispatcher });
        let plugin = UnodePlugin::instantiate(&mut store, &component, &linker)?;

        let manifest =
            from_wit_envelope(plugin.unode_plugin_lifecycle().call_manifest(&mut store)?);

        Ok(Self {
            store,
            plugin,
            manifest,
        })
    }

    pub fn manifest(&self) -> &PluginManifestEnvelope {
        &self.manifest
    }

    /// Returns the plugin's load data JSON.
    pub fn load(&mut self, request: &PluginLoadRequest) -> wasmtime::Result<JsonValue> {
        let response = self
            .plugin
            .unode_plugin_lifecycle()
            .call_load(&mut self.store, &to_wit_load(request))?
            .map_err(plugin_error)?;
        Ok(serde_json::from_str(&response)?)
    }

    /// Returns the rendered `ScreenNode` as JSON.
    pub fn render(&mut self, request: &PluginRenderRequest) -> wasmtime::Result<JsonValue> {
        let response = self
            .plugin
            .unode_plugin_lifecycle()
            .call_render(&mut self.store, &to_wit_render(request))?
            .map_err(plugin_error)?;
        Ok(serde_json::from_str(&response)?)
    }

    pub fn dispatch(
        &mut self,
        request: &PluginDispatchRequest,
    ) -> wasmtime::Result<PluginDispatchResponse> {
        let response = self
            .plugin
            .unode_plugin_lifecycle()
            .call_dispatch(&mut self.store, &to_wit_dispatch(request))?
            .map_err(plugin_error)?;
        Ok(from_wit_dispatch_response(response))
    }
}

fn plugin_error(err: wit::PluginError) -> wasmtime::Error {
    wasmtime::Error::msg(format!("plugin error: {}", err.message))
}

// ---------------------------------------------------------------------------
// serde -> WIT (requests sent to the plugin)
// ---------------------------------------------------------------------------

fn to_wit_route(route: &ResolvedRoute) -> wit::ResolvedRoute {
    wit::ResolvedRoute {
        pattern: route.pattern.clone(),
        params: route
            .params
            .iter()
            .map(|(key, value)| wit::Kv {
                key: key.clone(),
                value: value.clone(),
            })
            .collect(),
        query: route
            .query
            .iter()
            .map(|(key, value)| wit::Kv {
                key: key.clone(),
                value: value.clone(),
            })
            .collect(),
    }
}

fn to_wit_state(state: &BTreeMap<String, JsonValue>) -> Vec<wit::StateEntry> {
    state
        .iter()
        .map(|(path, value)| wit::StateEntry {
            path: path.clone(),
            value_json: value.to_string(),
        })
        .collect()
}

fn to_wit_load(request: &PluginLoadRequest) -> wit::LoadRequest {
    wit::LoadRequest {
        route: to_wit_route(&request.route),
        state: to_wit_state(&request.state_snapshot),
        locale: request.locale.clone(),
    }
}

fn to_wit_render(request: &PluginRenderRequest) -> wit::RenderRequest {
    wit::RenderRequest {
        route: to_wit_route(&request.route),
        data_json: request.data.to_string(),
        state: to_wit_state(&request.state_snapshot),
        locale: request.locale.clone(),
    }
}

fn to_wit_dispatch(request: &PluginDispatchRequest) -> wit::DispatchRequest {
    let kind = match &request.action.r#type {
        ActionType::Core(core) => wit::ActionKind::Core(
            serde_json::to_value(core)
                .ok()
                .and_then(|value| value.as_str().map(ToString::to_string))
                .unwrap_or_default(),
        ),
        ActionType::Custom(name) => wit::ActionKind::Custom(name.clone()),
    };
    wit::DispatchRequest {
        route: to_wit_route(&request.route),
        action: wit::ActionRef {
            kind,
            params_json: request
                .action
                .params
                .as_ref()
                .and_then(|params| serde_json::to_string(params).ok()),
            confirm_json: request
                .action
                .confirm
                .as_ref()
                .and_then(|confirm| serde_json::to_string(confirm).ok()),
        },
        state: to_wit_state(&request.state_snapshot),
        locale: request.locale.clone(),
    }
}

// ---------------------------------------------------------------------------
// WIT -> serde (responses read from the plugin)
// ---------------------------------------------------------------------------

fn from_wit_text(value: wit::TextValue) -> StringOrExpr {
    match value {
        wit::TextValue::Literal(text) => OneOrExpr::Value(text),
        wit::TextValue::Binding(path) => OneOrExpr::Expr(UiExpr::Binding { path }),
        wit::TextValue::Param(name) => OneOrExpr::Expr(UiExpr::Param { name }),
    }
}

fn from_wit_bool(value: wit::BoolValue) -> BoolOrExpr {
    match value {
        wit::BoolValue::Literal(flag) => OneOrExpr::Value(flag),
        wit::BoolValue::Binding(path) => OneOrExpr::Expr(UiExpr::Binding { path }),
        wit::BoolValue::Param(name) => OneOrExpr::Expr(UiExpr::Param { name }),
    }
}

fn from_wit_envelope(envelope: wit::ManifestEnvelope) -> PluginManifestEnvelope {
    let manifest = envelope.manifest;
    PluginManifestEnvelope {
        abi_version: envelope.abi_version,
        manifest: PluginManifest {
            id: manifest.id,
            name: manifest.name,
            version: manifest.version,
            api_version: manifest.api_version,
            description: manifest.description,
            author: manifest.author,
            permissions: manifest
                .permissions
                .into_iter()
                .map(|request| unode_plugin_sdk::prelude::PermissionRequest {
                    permission: request.permission,
                    required: request.required,
                    reason: request.reason,
                    allowed_origins: request.allowed_origins,
                })
                .collect(),
            requires: manifest.requires,
            host_id: manifest.host_id,
            slot_contributions: manifest
                .slot_contributions
                .into_iter()
                .map(|decl| SlotContributionDecl {
                    id: decl.id,
                    target: decl.target,
                    priority: decl.priority,
                    when: decl.when.map(from_wit_bool),
                })
                .collect(),
            routes: manifest
                .routes
                .into_iter()
                .map(|route| RouteDecl {
                    pattern: route.pattern,
                    screen_kind: route.screen_kind,
                    priority: route.priority,
                    label: route.label.map(from_wit_text),
                    badge: route.badge.map(from_wit_text),
                    group: route.group,
                })
                .collect(),
            route_groups: manifest
                .route_groups
                .into_iter()
                .map(|group| RouteGroupDecl {
                    id: group.id,
                    intent: match group.intent {
                        wit::NavIntent::Tabs => NavIntent::Tabs,
                        wit::NavIntent::Pages => NavIntent::Pages,
                    },
                })
                .collect(),
        },
    }
}

fn from_wit_dispatch_response(response: wit::DispatchResponse) -> PluginDispatchResponse {
    PluginDispatchResponse {
        handled: response.handled,
        outcome: match response.outcome {
            wit::DispatchOutcome::None => PluginDispatchOutcome::None,
            wit::DispatchOutcome::RefreshCurrentScreen => {
                PluginDispatchOutcome::RefreshCurrentScreen
            }
            wit::DispatchOutcome::Navigate(to) => PluginDispatchOutcome::Navigate { to },
        },
        message: response.message,
        data: response
            .data_json
            .as_deref()
            .and_then(|data| serde_json::from_str(data).ok()),
    }
}
