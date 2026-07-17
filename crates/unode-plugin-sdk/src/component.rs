//! Component Model (WIT) boundary — the typed twin of the raw ptr/len ABI.
//!
//! `wit/unode-plugin.wit` is the contract; this module holds the generated
//! bindings, the conversions between WIT records and the canonical serde
//! types, and [`export_plugin_component!`], which accepts the exact same
//! plugin functions as `export_plugin!`. One plugin source, two boundaries,
//! identical JSON — that is the golden rule hosts test against.

use serde_json::Value as JsonValue;
use unode::core::ast::{ActionRef, ActionType, BoolOrExpr, OneOrExpr, StringOrExpr, UiExpr};
use unode::core::runtime::{NavIntent, PluginManifest, ResolvedRoute};

use crate::abi::{PluginManifestEnvelope, PluginRenderSlotRequest};
use crate::{PluginDispatchOutcome, PluginDispatchRequest, PluginDispatchResponse};
use crate::{PluginLoadRequest, PluginRenderRequest};

pub mod bindings {
    wit_bindgen::generate!({
        path: "../../wit",
        world: "unode-plugin",
        pub_export_macro: true,
        export_macro_name: "export_unode_plugin",
        default_bindings_module: "unode_plugin_sdk::component::bindings",
    });
}

use bindings::unode::plugin::types as wit;

// ---------------------------------------------------------------------------
// serde -> WIT (exports produced by the plugin)
// ---------------------------------------------------------------------------

fn to_wit_text(value: &StringOrExpr) -> wit::TextValue {
    match value {
        OneOrExpr::Value(text) => wit::TextValue::Literal(text.clone()),
        OneOrExpr::Expr(UiExpr::Literal { value }) => wit::TextValue::Literal(value.clone()),
        OneOrExpr::Expr(UiExpr::Binding { path }) => wit::TextValue::Binding(path.clone()),
        OneOrExpr::Expr(UiExpr::Param { name }) => wit::TextValue::Param(name.clone()),
    }
}

fn to_wit_bool(value: &BoolOrExpr) -> wit::BoolValue {
    match value {
        OneOrExpr::Value(flag) => wit::BoolValue::Literal(*flag),
        OneOrExpr::Expr(UiExpr::Literal { value }) => wit::BoolValue::Literal(*value),
        OneOrExpr::Expr(UiExpr::Binding { path }) => wit::BoolValue::Binding(path.clone()),
        OneOrExpr::Expr(UiExpr::Param { name }) => wit::BoolValue::Param(name.clone()),
    }
}

fn to_wit_manifest(manifest: &PluginManifest) -> wit::Manifest {
    wit::Manifest {
        id: manifest.id.clone(),
        name: manifest.name.clone(),
        version: manifest.version.clone(),
        api_version: manifest.api_version.clone(),
        description: manifest.description.clone(),
        author: manifest.author.clone(),
        permissions: manifest
            .permissions
            .iter()
            .map(|request| wit::PermissionRequest {
                permission: request.permission.clone(),
                required: request.required,
                reason: request.reason.clone(),
                allowed_origins: request.allowed_origins.clone(),
            })
            .collect(),
        requires: manifest.requires.clone(),
        host_id: manifest.host_id.clone(),
        slot_contributions: manifest
            .slot_contributions
            .iter()
            .map(|decl| wit::SlotContribution {
                id: decl.id.clone(),
                target: decl.target.clone(),
                priority: decl.priority,
                when: decl.when.as_ref().map(to_wit_bool),
            })
            .collect(),
        routes: manifest
            .routes
            .iter()
            .map(|route| wit::Route {
                pattern: route.pattern.clone(),
                screen_kind: route.screen_kind.clone(),
                priority: route.priority,
                label: route.label.as_ref().map(to_wit_text),
                badge: route.badge.as_ref().map(to_wit_text),
                group: route.group.clone(),
            })
            .collect(),
        route_groups: manifest
            .route_groups
            .iter()
            .map(|group| wit::RouteGroup {
                id: group.id.clone(),
                intent: match group.intent {
                    NavIntent::Tabs => wit::NavIntent::Tabs,
                    NavIntent::Pages => wit::NavIntent::Pages,
                },
            })
            .collect(),
    }
}

pub fn to_wit_envelope(envelope: &PluginManifestEnvelope) -> wit::ManifestEnvelope {
    wit::ManifestEnvelope {
        abi_version: envelope.abi_version.clone(),
        manifest: to_wit_manifest(&envelope.manifest),
    }
}

pub fn to_wit_dispatch_response(response: &PluginDispatchResponse) -> wit::DispatchResponse {
    wit::DispatchResponse {
        handled: response.handled,
        outcome: match &response.outcome {
            PluginDispatchOutcome::None => wit::DispatchOutcome::None,
            PluginDispatchOutcome::RefreshCurrentScreen => {
                wit::DispatchOutcome::RefreshCurrentScreen
            }
            PluginDispatchOutcome::Navigate { to } => wit::DispatchOutcome::Navigate(to.clone()),
        },
        message: response.message.clone(),
        data_json: response.data.as_ref().map(|data| data.to_string()),
    }
}

// ---------------------------------------------------------------------------
// WIT -> serde (requests consumed by the plugin)
// ---------------------------------------------------------------------------

fn from_wit_route(route: wit::ResolvedRoute) -> ResolvedRoute {
    ResolvedRoute {
        pattern: route.pattern,
        params: route
            .params
            .into_iter()
            .map(|entry| (entry.key, entry.value))
            .collect(),
        query: route
            .query
            .into_iter()
            .map(|entry| (entry.key, entry.value))
            .collect(),
    }
}

fn from_wit_state(entries: Vec<wit::StateEntry>) -> std::collections::BTreeMap<String, JsonValue> {
    entries
        .into_iter()
        .map(|entry| {
            let value = serde_json::from_str(&entry.value_json).unwrap_or(JsonValue::Null);
            (entry.path, value)
        })
        .collect()
}

pub fn from_wit_load(request: wit::LoadRequest) -> PluginLoadRequest {
    PluginLoadRequest {
        route: from_wit_route(request.route),
        state_snapshot: from_wit_state(request.state),
        locale: request.locale,
    }
}

pub fn from_wit_render(request: wit::RenderRequest) -> PluginRenderRequest {
    PluginRenderRequest {
        route: from_wit_route(request.route),
        data: serde_json::from_str(&request.data_json).unwrap_or(JsonValue::Null),
        state_snapshot: from_wit_state(request.state),
        locale: request.locale,
    }
}

pub fn from_wit_render_slot(request: wit::RenderSlotRequest) -> PluginRenderSlotRequest {
    PluginRenderSlotRequest {
        contribution_id: request.contribution_id,
        slot_name: request.slot_name,
        route: from_wit_route(request.route),
        state_snapshot: from_wit_state(request.state),
        locale: request.locale,
    }
}

pub fn from_wit_dispatch(request: wit::DispatchRequest) -> PluginDispatchRequest {
    let kind = match request.action.kind {
        wit::ActionKind::Core(name) => serde_json::from_value(JsonValue::String(name.clone()))
            .map(ActionType::Core)
            .unwrap_or(ActionType::Custom(name)),
        wit::ActionKind::Custom(name) => ActionType::Custom(name),
    };
    PluginDispatchRequest {
        route: from_wit_route(request.route),
        action: ActionRef {
            r#type: kind,
            params: request
                .action
                .params_json
                .as_deref()
                .and_then(|params| serde_json::from_str(params).ok()),
            confirm: request
                .action
                .confirm_json
                .as_deref()
                .and_then(|confirm| serde_json::from_str(confirm).ok()),
        },
        state_snapshot: from_wit_state(request.state),
        locale: request.locale,
    }
}

/// Serializes a lifecycle payload, mapping serde failures into the typed
/// `plugin-error` result instead of trapping.
pub fn json_result<T: serde::Serialize>(value: &T) -> Result<String, wit::PluginError> {
    serde_json::to_string(value).map_err(|err| wit::PluginError {
        message: format!("failed to serialize plugin payload: {err}"),
        details_json: None,
    })
}

// ---------------------------------------------------------------------------
// Host-call routing: the same `HostCallEnvelope` the raw ABI sends, delivered
// through the typed capability imports.
// ---------------------------------------------------------------------------

#[cfg(target_arch = "wasm32")]
pub(crate) fn send_host_call(envelope: &crate::abi::HostCallEnvelope) {
    match envelope.operation.as_str() {
        "state.set" => {
            let path = envelope
                .params
                .get("path")
                .and_then(JsonValue::as_str)
                .unwrap_or_default();
            let value = envelope
                .params
                .get("value")
                .cloned()
                .unwrap_or(JsonValue::Null);
            bindings::unode::plugin::state::set(path, &value.to_string());
        }
        "navigation.navigate" => {
            let to = envelope
                .params
                .get("to")
                .and_then(JsonValue::as_str)
                .unwrap_or_default();
            bindings::unode::plugin::navigation::navigate(to);
        }
        operation => {
            let params = serde_json::to_string(&envelope.params).unwrap_or_else(|_| "{}".into());
            let _ = bindings::unode::plugin::host::call(operation, &params);
        }
    }
}

/// Exports the plugin through the Component Model boundary. Takes the exact
/// same functions as `export_plugin!`:
///
/// ```ignore
/// unode_plugin_sdk::export_plugin_component! {
///     manifest: manifest_envelope,
///     load: load_response,
///     render: render_screen,
///     dispatch: dispatch_response,
/// }
/// ```
#[macro_export]
macro_rules! export_plugin_component {
    (
        manifest: $manifest:expr,
        load: $load:expr,
        render: $render:expr,
        dispatch: $dispatch:expr,
        render_slot: $render_slot:expr $(,)?
    ) => {
        struct __UnodePluginComponent;

        impl $crate::component::bindings::exports::unode::plugin::lifecycle::Guest
            for __UnodePluginComponent
        {
            fn manifest() -> $crate::component::bindings::unode::plugin::types::ManifestEnvelope {
                $crate::component::to_wit_envelope(&($manifest)())
            }

            fn load(
                request: $crate::component::bindings::unode::plugin::types::LoadRequest,
            ) -> Result<String, $crate::component::bindings::unode::plugin::types::PluginError>
            {
                let request = $crate::component::from_wit_load(request);
                $crate::component::json_result(&($load)(&request))
            }

            fn render(
                request: $crate::component::bindings::unode::plugin::types::RenderRequest,
            ) -> Result<String, $crate::component::bindings::unode::plugin::types::PluginError>
            {
                let request = $crate::component::from_wit_render(request);
                $crate::component::json_result(&($render)(&request))
            }

            fn render_slot(
                request: $crate::component::bindings::unode::plugin::types::RenderSlotRequest,
            ) -> Result<String, $crate::component::bindings::unode::plugin::types::PluginError>
            {
                let request = $crate::component::from_wit_render_slot(request);
                $crate::component::json_result(&($render_slot)(&request))
            }

            fn dispatch(
                request: $crate::component::bindings::unode::plugin::types::DispatchRequest,
            ) -> Result<
                $crate::component::bindings::unode::plugin::types::DispatchResponse,
                $crate::component::bindings::unode::plugin::types::PluginError,
            > {
                let request = $crate::component::from_wit_dispatch(request);
                Ok($crate::component::to_wit_dispatch_response(&($dispatch)(
                    &request,
                )))
            }
        }

        $crate::component::bindings::export_unode_plugin!(__UnodePluginComponent);
    };
    (
        manifest: $manifest:expr,
        load: $load:expr,
        render: $render:expr,
        dispatch: $dispatch:expr $(,)?
    ) => {
        $crate::export_plugin_component! {
            manifest: $manifest,
            load: $load,
            render: $render,
            dispatch: $dispatch,
            render_slot: |_request: &$crate::PluginRenderSlotRequest| {
                $crate::PluginRenderSlotResponse::default()
            },
        }
    };
}
