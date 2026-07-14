//! Platform-neutral host session for the web runtime.
//!
//! This is the exact same pipeline the TUI host drives, minus any terminal or
//! DOM concern:
//!
//! ```text
//! plugin render (JSON)
//!   -> normalize_screen        (fill defaults, compute reactivity metadata)
//!   -> track_reactive_bindings (populate the path -> node dependency graph)
//!   -> lower_screen            (IR the renderer mounts)
//!
//! state write(s)
//!   -> resolver.subscribers_of (which node keys are dirty for each path)
//!   -> plan_patch_ops          (re-resolve ONLY those nodes)
//!   -> lower_patch_ops         (IR patch ops the renderer re-applies)
//! ```
//!
//! `render()` is never called again after `mount()`. Reactivity is pure
//! expression resolution against an updated [`MemoryStateStore`].
//!
//! Keeping this type free of `wasm-bindgen` lets it be unit-tested on the host
//! toolchain; the browser wrapper in `lib.rs` is a thin JSON shim over it.

use std::collections::{BTreeMap, BTreeSet};

use serde_json::Value as JsonValue;

use unode::core::ast::ScreenNode;
use unode::core::canonical::CanonicalScreen;
use unode::core::ir::{lower_patch_ops, lower_screen, IrPatchOp, IrScreen};
use unode::core::normalize::normalize_screen;
use unode::core::planner::plan_patch_ops;
use unode::core::reactive::{track_reactive_bindings, BindingSubscriptions};
use unode::core::resolver::{DefaultExprResolver, ResolverContext};
use unode::core::runtime::ResolvedRoute;
use unode::core::state::{MemoryStateStore, StateStore};

/// One mounted screen's worth of host state: the canonical tree, the store, and
/// the reactive dependency graph tying state paths to node keys.
pub struct WebSessionCore {
    locale: String,
    route: ResolvedRoute,
    state: MemoryStateStore,
    resolver: DefaultExprResolver,
    screen: Option<CanonicalScreen>,
    subscriptions: Option<BindingSubscriptions>,
}

impl WebSessionCore {
    pub fn new(locale: impl Into<String>) -> Self {
        Self {
            locale: locale.into(),
            route: ResolvedRoute::default(),
            state: MemoryStateStore::new(None),
            resolver: DefaultExprResolver::default(),
            screen: None,
            subscriptions: None,
        }
    }

    /// Set the resolved route used when resolving `param` expressions.
    pub fn set_route(&mut self, route: ResolvedRoute) {
        self.route = route;
    }

    pub fn locale(&self) -> &str {
        &self.locale
    }

    /// Normalize a freshly rendered screen, seed the store, and build the
    /// reactive dependency graph. Returns the IR tree the renderer mounts.
    ///
    /// State is seeded from `screen.initialState` first, then any explicit
    /// `seed_state` (e.g. state restored across a navigation) is layered on top.
    pub fn mount(
        &mut self,
        screen: ScreenNode,
        seed_state: BTreeMap<String, JsonValue>,
    ) -> Result<IrScreen, String> {
        // Tear down the previous screen's subscriptions and start clean so node
        // keys from an old tree never leak into the new dependency graph.
        if let Some(subs) = self.subscriptions.take() {
            subs.teardown(&mut self.state);
        }
        self.resolver = DefaultExprResolver::default();
        self.state = MemoryStateStore::new(None);

        let canonical = normalize_screen(screen)?;

        if let Some(initial) = &canonical.initial_state {
            self.state.merge_data(initial.clone());
        }
        if !seed_state.is_empty() {
            self.state.merge_data(seed_state);
        }

        // The tracking walk resolves bindings against a read snapshot while it
        // subscribes on the live store, so the two cannot be the same borrow.
        let read_state = MemoryStateStore::new(Some(self.state.snapshot()));
        let ctx = ResolverContext {
            state: &read_state,
            route: Some(&self.route),
            locale: &self.locale,
        };
        // Populates `resolver` with the path -> node dependency edges. We drive
        // patches manually, so the live subscription's callback is a no-op; we
        // keep the handle only to tear it down on the next mount.
        let subscriptions =
            track_reactive_bindings(&canonical, &mut self.resolver, &ctx, &mut self.state, |_| {})?;

        let ir = lower_screen(&canonical);
        self.screen = Some(canonical);
        self.subscriptions = Some(subscriptions);
        Ok(ir)
    }

    /// The initial resolution pass: patch ops that resolve every binding in the
    /// mounted tree against the seeded state.
    ///
    /// The IR from [`Self::mount`] keeps bindings symbolic (`{ "b": path }`), so
    /// the renderer applies these once right after mounting to get concrete
    /// values. It reuses the exact same planner as live updates — the first
    /// render is simply the first patch cycle.
    pub fn initial_patches(&mut self) -> Result<Vec<IrPatchOp>, String> {
        let screen = self
            .screen
            .as_ref()
            .ok_or_else(|| "initial_patches called before mount".to_string())?;

        let dirty: BTreeSet<String> = self
            .subscriptions
            .as_ref()
            .map(|subs| subs.path_to_nodes.values().flatten().cloned().collect())
            .unwrap_or_default();

        let ctx = ResolverContext {
            state: &self.state,
            route: Some(&self.route),
            locale: &self.locale,
        };
        let ops = plan_patch_ops(screen, &dirty, &mut self.resolver, &ctx);
        Ok(lower_patch_ops(&ops))
    }

    /// Apply a batch of state writes and return the patch ops the renderer must
    /// re-apply. Only nodes whose bindings depend on a written path are touched.
    pub fn apply_writes(
        &mut self,
        writes: BTreeMap<String, JsonValue>,
    ) -> Result<Vec<IrPatchOp>, String> {
        let screen = self
            .screen
            .as_ref()
            .ok_or_else(|| "apply_writes called before mount".to_string())?;

        let mut dirty: BTreeSet<String> = BTreeSet::new();
        for (path, value) in writes {
            // Ancestor-prefix aware: a write to `work.title` wakes nodes bound to
            // `work` or `work.title`.
            for node_key in self.resolver.subscribers_of(&path) {
                dirty.insert(node_key);
            }
            self.state.set(&path, value);
        }

        let ctx = ResolverContext {
            state: &self.state,
            route: Some(&self.route),
            locale: &self.locale,
        };
        let ops = plan_patch_ops(screen, &dirty, &mut self.resolver, &ctx);
        Ok(lower_patch_ops(&ops))
    }

    /// Current flat state snapshot — handed to the plugin as `state_snapshot`
    /// on the next `dispatch` so action handlers can read current values.
    pub fn state_snapshot(&self) -> BTreeMap<String, JsonValue> {
        flatten_snapshot(self.state.snapshot())
    }
}

fn flatten_snapshot(snapshot: BTreeMap<String, JsonValue>) -> BTreeMap<String, JsonValue> {
    let mut out = BTreeMap::new();

    for (key, value) in snapshot {
        flatten_value(&mut out, key, value);
    }

    out
}

fn flatten_value(out: &mut BTreeMap<String, JsonValue>, path: String, value: JsonValue) {
    match value {
        JsonValue::Object(map) => {
            if map.is_empty() {
                out.insert(path, JsonValue::Object(map));
                return;
            }

            for (key, value) in map {
                flatten_value(out, format!("{path}.{key}"), value);
            }
        }
        JsonValue::Array(values) => {
            if values.is_empty() {
                out.insert(path, JsonValue::Array(values));
                return;
            }

            for (index, value) in values.into_iter().enumerate() {
                flatten_value(out, format!("{path}.{index}"), value);
            }
        }
        value => {
            out.insert(path, value);
        }
    }
}
