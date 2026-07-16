# unode Runtime

## Plugin contract

A plugin is a WASM module that exports:

```
plugin_manifest()           → manifest JSON string
plugin_load(request_json)    → data JSON string
plugin_render(request_json)  → ScreenNode JSON string
plugin_render_slot(request_json) → PluginRenderSlotResponse JSON string
plugin_dispatch(request_json) → PluginDispatchResponse JSON string
```

In Rust, plugin authors use `unode-sdk` which hides the WASM export
boilerplate:

```rust
use unode_plugin_sdk::prelude::*;

#[unode::plugin]
fn manifest() -> PluginManifest {
    plugin_manifest("com.mugenx.catalog", "Catalog")
        .permission(permission("catalog.read").required(true).reason("Load works"))
        // One plugin can declare multiple screens. The host registers each
        // declared route and dispatches matching navigations back through
        // `plugin_render`, which branches on the resolved `route.pattern`.
        //
        // Grouped routes share a navigation intent. `tabs()` is a hint, not a
        // command: hosts that support tabs derive the tab bar (and the active
        // tab) from the group + matched route via `route_tabs_view`; hosts
        // that don't simply present the members as separate routes. Labels
        // and badges accept state bindings for dynamic values.
        .route_group(route_group("browse").tabs())
        .routes([
            route("/app/mangas/hot")
                .group("browse")
                .label("Hot"),
            route("/app/mangas/recent")
                .group("browse")
                .label("Recent")
                .badge(expr::binding::<String>("mangas.recentCount")),
            route("/app/mangas/:work_id/meta").screen_kind("com.mugenx.catalog.work-meta"),
        ])
        .build()
}

#[unode::load("/app/mangas/browse")]
async fn load_browse(ctx: &PluginContext) -> BrowseData {
    let works = ctx.api::<CatalogApi>().list_works().await.unwrap_or_default();
    BrowseData { works }
}

#[unode::render("/app/mangas/browse")]
fn render_browse(data: &BrowseData, ctx: &PluginContext) -> ScreenNode {
    let t = ctx.i18n.translator();
    ui::screen("browse-screen", vec![
        ui::grid(GridOpts { columns: responsive(1, 2, 3, 4, 5), gap: Gap::Lg },
            data.works.iter().map(|w| work_banner(&w.view_model())).collect()
        )
    ])
}
```

---

## Registries

The plugin runtime maintains registries that plugins populate at activation:

- **Routes** — pattern → (load fn, render fn). Declared in the manifest as
  `routes: [RouteDecl]` (`pattern`, optional `screenKind`, `priority`, plus
  navigation metadata: `label`, `badge`, `group`). Hosts register them at load
  time (`RouteRegistry::register_manifest_routes`) so a single plugin can own
  multiple screens; the matched pattern and params come back to the plugin in
  `PluginRenderRequest.route`. Plugins without declared routes keep whatever
  route the host assigns them.
- **Route groups** — `routeGroups: [{ id, intent }]` name a set of declared
  routes and hint at their presentation (`tabs` | `pages`). The renderer
  decides: hosts with tab support call `route_tabs_view(manifest, pattern,
  state)` (Rust) / `routeTabsView(...)` (TS) to derive a ready-to-render tab
  set — active tab comes from the matched route, and labels/badges may be
  state bindings for dynamic values. Hosts without tabs treat the group as
  separate routes. Screens carry no tab metadata; navigation chrome is
  host-derived.
- **Actions** — type string → handler fn
- **Commands** — id → (title, category, handler)
- **Navigation** — id → (label, path, priority)
- **Providers** — capability → provider fn
- **Slots** — target slot name → contribution fn

These registries are generic (no domain knowledge). The host queries them to
build navigation menus, command palettes, and slot contributions.

---

## Route lifecycle (detailed)

```
1. Route match
   RouteRegistry.resolve(pathname) → (plugin_id, pattern, params)

2. Plugin instantiation (if not cached)
   WasmRuntime.get_or_instantiate(plugin_id, permission_profile)
   → validates required permissions
   → injects host functions gated by PermissionProfile
   → calls plugin.init() if exported

3. Fresh StateStore
   MemoryStateStore::new()
   // seeded from ScreenNode.initialState after render

4. load()
   plugin.load(route_json)
   → receives { pattern, params, query }
   → returns data_json
   → host calls state.merge_data(data_json)

5. render()
   plugin.render(data_json)
   → receives the data from step 4
   → returns raw CanonicalScreen JSON (before normalization)

6. Normalize
   normalize_screen(raw_json)
   → fills defaults
   → computes _reactivity, _subtreeReactivity, _staticFields
   → validates id uniqueness
   → assigns structural _key fallbacks
   // Also merges screen.initialState into StateStore

7. Resolve slots
   resolve_slots(canonical_screen, slot_registry, ctx, renderer)
   → evaluates manifest-declared slot contribution `when` expressions
   → calls contributing plugins through plugin_render_slot()
   → normalizes returned UiNodes
   → preserves contributor origin for action/capability routing

8. Track bindings
   track_reactive_bindings(screen, resolver, state, on_patch)
   → walks static-skipping tree
   → registers path subscriptions

9. Lower + mount
   lower_screen(canonical_screen)
   renderer.mount(ir_screen)
   → Web: keyed framework adapter (React in the current vertical slice)
   → TUI: Ratatui widget layout

10. Reactive loop
   on user interaction:
     → dispatch ActionRef to plugin.dispatch()
     → plugin calls host functions (state.set, navigate, etc.)
     → host applies buffered state writes
     → resolver.subscribers_of(path) finds dirty node keys
     → plan_patch_ops(...) produces targeted patch ops
     → renderer applies only those patches

11. Teardown on navigation
    subscriptions.teardown()
    state.reset()
    // WASM instance may be kept in a pool for reuse
```

---

## Built-in actions

These are handled by the renderer before reaching the plugin's action registry:

| Action type | Params | Effect |
|---|---|---|
| `unode.setState` | `{ path, value }` | Writes to StateStore |
| `unode.navigate` | `{ to, mode?, query? }` | Triggers navigation |
| `unode.refresh` | `{}` | Triggers full load/render cycle |
| `unode.batchState` | `{ [path]: value }` | Batches multiple StateStore writes |

All other action types are dispatched to plugin action handlers.

---

## State model

### Local screen state (SPA-like)

- Ephemeral, screen-scoped
- Driven by `unode.setState` actions and direct `ctx.state.set()` calls
- Lives in `MemoryStateStore` on the host
- Destroyed on unmount (unless explicitly persisted via `ctx.storage`)
- Drives `binding` expression evaluation

Typical uses: disclosure expanded/collapsed, filter selections, draft input,
optimistic UI flags.

### Route state (URL-like)

- Addressable and shareable
- Web: maps to URL path + query string
- TUI: breadcrumb stack + params object
- Survives navigation history (back/forward)
- Accessible via `UiExpr::Param { name }`

Typical uses: selected work id, search query, active page.

### Persistence across sessions

Via `ctx.storage` (session or persistent scope, namespaced per plugin):

```rust
ctx.storage.set("persistent", "last_read_chapter", chapter_id)?;
let last = ctx.storage.get("persistent", "last_read_chapter")?;
```

---

## Action handler context

Plugin action handlers receive a context that includes the current state and
route, but NOT the host API directly — they access it via host functions:

```rust
#[unode::action("catalog.addFavorite")]
async fn add_favorite(ctx: &PluginContext, params: &ActionParams) {
    let work_id = params.get_str("workId")?;

    // Optimistic update first
    ctx.state.set("ui.favorited", json!(true));

    // Call domain API via host function
    match ctx.api::<LibraryApi>().add_favorite(work_id).await {
        Ok(()) => {}
        Err(_) => {
            // Rollback
            ctx.state.set("ui.favorited", json!(false));
            ctx.state.set("ui.favoriteError", json!("Failed. Try again."));
        }
    }
}
```

---

## Plugin caching

WASM modules are expensive to instantiate (parse + compile + link). The runtime
keeps a pool of instantiated modules:

```rust
struct PluginPool {
    instances: HashMap<String, Vec<WasmInstance>>,
    max_per_plugin: usize,
}

impl PluginPool {
    fn acquire(&mut self, plugin_id: &str) -> Option<WasmInstance> {
        self.instances.get_mut(plugin_id)?.pop()
    }

    fn release(&mut self, plugin_id: &str, instance: WasmInstance) {
        // Reset instance state before returning to pool
        instance.call_export("plugin_reset").ok();
        self.instances.entry(plugin_id.into()).or_default().push(instance);
    }
}
```

Instances are reset between uses. Only the StateStore is recreated per
navigation — the WASM module itself is reused.
