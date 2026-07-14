//! Web host for unode: the browser-side glue that runs the **same** Rust core
//! pipeline as the TUI, exposed to JavaScript through `wasm-bindgen`.
//!
//! Architecture of the web vertical slice:
//!
//! ```text
//!  ┌─────────────────────────┐        ┌──────────────────────────────┐
//!  │ plugin.wasm (C ABI)     │        │ unode_web_host.wasm (bindgen) │
//!  │  render/dispatch → JSON │        │  normalize · track · plan     │
//!  └───────────▲─────────────┘        └───────────────▲──────────────┘
//!              │ instantiate + host_call              │ WebSession
//!  ┌───────────┴──────────────────────────────────────┴──────────────┐
//!  │ JS glue (pluginHost.ts)  ──drives──▶  React adapter (UnodeScreen) │
//!  └──────────────────────────────────────────────────────────────────┘
//! ```
//!
//! Both wasm modules are instantiated by JS (no nested instantiation). This
//! module is the second one: it owns normalization, the reactive dependency
//! graph, and patch planning, so none of that is ever re-implemented in TS.
//!
//! The heavy lifting lives in [`session::WebSessionCore`], which is plain Rust
//! and unit-testable without the wasm toolchain. The `#[wasm_bindgen]` wrapper
//! below is only a JSON-in / JSON-out shim and is compiled solely for
//! `wasm32`, so `cargo test` on the host toolchain needs no wasm-bindgen.

pub mod session;

pub use session::WebSessionCore;

#[cfg(target_arch = "wasm32")]
mod wasm {
    use std::collections::BTreeMap;

    use serde_json::Value as JsonValue;
    use wasm_bindgen::prelude::*;

    use unode::core::ast::ScreenNode;
    use unode::core::runtime::ResolvedRoute;

    use crate::session::WebSessionCore;

    fn json_err(err: impl std::fmt::Display) -> JsValue {
        JsValue::from_str(&err.to_string())
    }

    /// JS-facing handle to a mounted screen. One per active plugin screen.
    #[wasm_bindgen]
    pub struct WebSession {
        core: WebSessionCore,
    }

    #[wasm_bindgen]
    impl WebSession {
        #[wasm_bindgen(constructor)]
        pub fn new(locale: String) -> WebSession {
            WebSession {
                core: WebSessionCore::new(locale),
            }
        }

        /// Set the active route (JSON `ResolvedRoute`) before `mount`.
        #[wasm_bindgen(js_name = setRoute)]
        pub fn set_route(&mut self, route_json: &str) -> Result<(), JsValue> {
            let route: ResolvedRoute = serde_json::from_str(route_json).map_err(json_err)?;
            self.core.set_route(route);
            Ok(())
        }

        /// Normalize + track a rendered screen. Returns the IR screen JSON the
        /// React adapter mounts.
        ///
        /// - `screen_json`: the plugin's `render()` output (a raw `ScreenNode`).
        /// - `seed_json`: a flat `{ "path": value }` map, or `"{}"`.
        pub fn mount(&mut self, screen_json: &str, seed_json: &str) -> Result<String, JsValue> {
            let screen: ScreenNode = serde_json::from_str(screen_json).map_err(json_err)?;
            let seed: BTreeMap<String, JsonValue> =
                serde_json::from_str(seed_json).map_err(json_err)?;
            let ir = self.core.mount(screen, seed).map_err(json_err)?;
            serde_json::to_string(&ir).map_err(json_err)
        }

        /// Initial resolution pass — JSON array of IR patch ops resolving every
        /// binding against the seeded state. Apply once right after `mount`.
        #[wasm_bindgen(js_name = initialPatches)]
        pub fn initial_patches(&mut self) -> Result<String, JsValue> {
            let ops = self.core.initial_patches().map_err(json_err)?;
            serde_json::to_string(&ops).map_err(json_err)
        }

        /// Apply a flat batch of state writes; returns a JSON array of IR patch
        /// ops (`{ o, k, f?, v?, n?, c? }`) for the renderer to re-apply.
        #[wasm_bindgen(js_name = applyWrites)]
        pub fn apply_writes(&mut self, writes_json: &str) -> Result<String, JsValue> {
            let writes: BTreeMap<String, JsonValue> =
                serde_json::from_str(writes_json).map_err(json_err)?;
            let ops = self.core.apply_writes(writes).map_err(json_err)?;
            serde_json::to_string(&ops).map_err(json_err)
        }

        /// Flat snapshot of current state, to feed the plugin as
        /// `state_snapshot` on the next dispatch.
        #[wasm_bindgen(js_name = stateSnapshot)]
        pub fn state_snapshot(&self) -> Result<String, JsValue> {
            serde_json::to_string(&self.core.state_snapshot()).map_err(json_err)
        }
    }
}
