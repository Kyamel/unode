# unode Rust Implementation Guide

This document describes how to implement unode in Rust from the ground up.
It covers the core crate, the plugin SDK crate, the web renderer integration,
and the TUI renderer.

---

## Crate structure

```
unode/                          ← workspace root
  crates/
    unode/                 ← AST, normalize, state, resolver, transport
    unode-sdk/           ← DSL builders, plugin manifest, host function wrappers
    unode-web-runtime/          ← JS-facing WASM entry points for web renderer
    unode-web-host/             ← wasm-bindgen host session for normalize/track/patch
    mugens-sdk/                ← Mugen-specific domain API and host functions
    renderer/           ← TUI renderer (Ratatui + Wasmtime)
```

---

## Phase 1 — unode

### 1.1 AST types

Define all node types as Rust enums/structs with Serde derive macros.

```rust
// Cargo.toml
[dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"

// src/ast.rs
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum UiNode {
    Section(SectionNode),
    Stack(StackNode),
    Inline(InlineNode),
    Grid(GridNode),
    Scroll(ScrollNode),
    Text(TextNode),
    Value(ValueNode),
    Icon(IconNode),
    Badge(BadgeNode),
    Divider(DividerNode),
    Media(MediaNode),
    Pressable(PressableNode),
    Item(ItemNode),
    List(ListNode),
    Action(ActionNode),
    Actions(ActionsNode),
    Disclosure(DisclosureNode),
    Menu(MenuNode),
    Input(InputNode),
    Form(FormNode),
    Status(StatusNode),
    Empty(EmptyStateNode),
    Loading(LoadingNode),
    Conditional(ConditionalNode),
    Slot(SlotNode),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ScreenNode {
    pub id: Option<String>,
    pub title: Option<StringOrExpr>,
    pub subtitle: Option<StringOrExpr>,
    pub initial_focus: Option<String>,
    pub initial_state: Option<HashMap<String, JsonValue>>,
    pub children: Vec<UiNode>,
    pub meta: Option<HashMap<String, JsonValue>>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum UiExpr {
    Literal { value: serde_json::Value },
    Binding { path: String },
    Param    { name: String },
}

// StringOrExpr serializes as either a plain string or a UiExpr object
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(untagged)]
pub enum StringOrExpr {
    String(String),
    Expr(UiExpr),
}
```

All node types follow the same pattern. Fields use `Option<T>` for optional
values and match the JSON field names via `#[serde(rename_all = "camelCase")]`.

### 1.2 Canonical metadata

```rust
// src/normalize.rs
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CanonicalMeta {
    pub _key: String,
    pub _reactivity: NodeReactivity,
    pub _subtree_reactivity: NodeReactivity,
    pub _static_fields: HashMap<String, serde_json::Value>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum NodeReactivity {
    Static,
    Reactive,
    Conditional,
}
```

### 1.3 MemoryStateStore

```rust
// src/state.rs
pub struct MemoryStateStore {
    data: HashMap<String, serde_json::Value>,
    subscribers: HashMap<String, Vec<Box<dyn Fn(serde_json::Value) + Send>>>,
    prefix_subscribers: HashMap<String, Vec<Box<dyn Fn(serde_json::Value, String) + Send>>>,
}

impl MemoryStateStore {
    pub fn get(&self, path: &str) -> Option<&serde_json::Value> { ... }
    pub fn set(&mut self, path: &str, value: serde_json::Value) { ... }
    pub fn merge_data(&mut self, data: HashMap<String, serde_json::Value>) { ... }
    pub fn subscribe(&mut self, path: &str, f: impl Fn(serde_json::Value) + Send + 'static)
        -> impl FnOnce() { ... }
    pub fn snapshot(&self) -> HashMap<String, serde_json::Value> { ... }
    pub fn reset(&mut self) { ... }
}
```

Paths are dot-separated. `set("work.title", value)` traverses the nested
map and creates intermediate maps as needed. `subscribe` returns an unsubscribe
closure. Notifications batch across a single `set()` call.

### 1.4 ExprResolver

```rust
// src/resolver.rs
pub struct ExprResolver {
    node_to_path: HashMap<String, HashSet<String>>,
    path_to_node: HashMap<String, HashSet<String>>,
}

impl ExprResolver {
    pub fn track(&mut self, node_key: &str, path: &str) { ... }
    pub fn clear_tracking(&mut self, node_key: &str) { ... }
    pub fn dependencies_of(&self, node_key: &str) -> Vec<&str> { ... }
    pub fn subscribers_of(&self, path: &str) -> Vec<String> {
        // Include ancestor prefix matching:
        // a change at "work.title" wakes nodes subscribed to "work"
        self.path_to_node.iter()
            .filter(|(p, _)| path == p.as_str() || path.starts_with(&format!("{}.", p)))
            .flat_map(|(_, nodes)| nodes.iter().cloned())
            .collect()
    }
    pub fn resolve_string(&mut self, expr: &StringOrExpr, ctx: &ResolverCtx, node_key: &str) -> String { ... }
    pub fn resolve_bool(&mut self, expr: &BoolOrExpr, ctx: &ResolverCtx, node_key: &str) -> bool { ... }
}
```

### 1.5 normalizeScreen

```rust
// src/normalize.rs
pub fn normalize_screen(screen: ScreenNode) -> CanonicalScreen {
    let mut ctx = NormalizeCtx::root();
    normalize_screen_node(screen, &mut ctx)
}

struct NormalizeCtx {
    path: String,
    crumbs: Vec<String>,
    seen_ids: HashMap<String, String>,
}
```

Normalization fills defaults (`gap: "md"` if absent, etc.), computes
`_reactivity` and `_subtree_reactivity`, validates `id` uniqueness, collapses
`literal` expressions to their values, and assigns structural `_key` fallbacks
when `id` is absent.

The dev warning for grids with `continuation` but children without `id`:

```rust
#[cfg(debug_assertions)]
fn check_grid_ids(node: &GridNode, ctx: &NormalizeCtx) {
    if node.continuation.is_some() {
        let missing: usize = node.children.iter().filter(|c| c.id().is_none()).count();
        if missing > 0 {
            eprintln!(
                "[unode] GridNode at {} has continuation but {} child(ren) without explicit id.",
                ctx.path, missing
            );
        }
    }
}
```

### 1.6 trackReactiveBindings

```rust
// src/reactive.rs
pub struct BindingSubscriptions {
    pub path_to_nodes: HashMap<String, HashSet<String>>,
    pub teardown: Box<dyn FnOnce()>,
}

pub fn track_reactive_bindings(
    screen: &CanonicalScreen,
    resolver: &mut ExprResolver,
    ctx: &ResolverCtx,
    state: &mut MemoryStateStore,
    on_patch: impl Fn(HashSet<String>) + Send + 'static,
) -> BindingSubscriptions {
    // Phase 1: walk tree, call resolver.track() for each binding
    walk_canonical(screen, resolver, ctx);

    // Phase 2: collect all tracked paths
    let mut all_paths = HashSet::new();
    collect_tracked_paths(screen, resolver, &mut all_paths);

    // Phase 3: subscribe to each path
    let mut unsubs: Vec<Box<dyn FnOnce()>> = vec![];
    let mut path_to_nodes = HashMap::new();

    for path in &all_paths {
        let subscribers: HashSet<String> = resolver.subscribers_of(path).into_iter().collect();
        path_to_nodes.insert(path.clone(), subscribers.clone());

        let on_patch = on_patch.clone();  // on_patch: Arc<dyn Fn>
        let unsub = state.subscribe(path, move |_| {
            on_patch(subscribers.clone());
        });
        unsubs.push(Box::new(unsub));
    }

    BindingSubscriptions {
        path_to_nodes,
        teardown: Box::new(move || { for u in unsubs { u(); } }),
    }
}
```

### 1.7 Transport layer

```rust
// src/transport.rs
#[derive(Serialize, Deserialize)]
pub struct ScreenEnvelope {
    #[serde(rename = "type")]
    pub type_: String,     // "unode-screen"
    pub v: String,         // AST version
    pub ts: String,        // ISO 8601 timestamp
    pub screen_kind: Option<String>,
    pub screen: CanonicalScreen,
}

pub fn screen_to_json(screen: &CanonicalScreen, opts: SerializeOpts) -> String {
    let envelope = ScreenEnvelope { ... };
    if opts.pretty {
        serde_json::to_string_pretty(&envelope).unwrap()
    } else {
        serde_json::to_string(&envelope).unwrap()
    }
}

pub fn screen_from_json(json: &str) -> Result<ScreenEnvelope, TransportError> {
    let envelope: ScreenEnvelope = serde_json::from_str(json)?;
    if !version_compatible(&envelope.v, UNODE_AST_VERSION) {
        return Err(TransportError::VersionMismatch { ... });
    }
    Ok(envelope)
}
```

---

## Phase 2 — unode-sdk

The SDK crate is compiled to `.wasm` alongside the plugin. It provides:

1. **DSL builders** — Rust functions that construct `UiNode` values
2. **PluginManifest** declaration helpers
3. **PluginContext** — wraps host function calls
4. **WASM entry points** — `plugin_manifest`, `plugin_load`, `plugin_render`, `plugin_dispatch`

```rust
// SDK DSL builders
pub mod ui {
    pub fn screen(id: impl Into<String>, children: Vec<UiNode>) -> ScreenNode { ... }
    pub fn stack(gap: Option<Gap>, children: Vec<UiNode>) -> UiNode { ... }
    pub fn text(content: impl Into<StringOrExpr>) -> UiNode { ... }
    pub fn action(label: impl Into<StringOrExpr>, action: ActionRef) -> UiNode { ... }
    // ... all node types
}

// PluginContext wraps host function calls
pub struct PluginContext {
    route: ResolvedRoute,
}

impl PluginContext {
    pub fn locale(&self) -> String {
        // calls locale_get() host function
        unsafe { __unode_locale_get() }
    }

    pub fn state_get(&self, path: &str) -> Option<serde_json::Value> {
        // calls state_get() host function
    }

    pub fn state_set(&self, path: &str, value: serde_json::Value) {
        // calls state_set() host function
    }
}

// Host function declarations (implemented by the host)
extern "C" {
    fn __unode_state_get(path_ptr: *const u8, path_len: usize) -> *const u8;
    fn __unode_state_set(path_ptr: *const u8, path_len: usize, val_ptr: *const u8, val_len: usize);
    fn __unode_locale_get() -> *const u8;
    fn __unode_http_fetch(url_ptr: *const u8, url_len: usize) -> *const u8;
    // ... domain-specific functions declared by each app's bridge crate
}
```

---

## Phase 3 — mugens-sdk

The bridge implements:

1. **Domain models** as Rust structs with Serde
2. **Host functions** for each domain API method
3. **PermissionGuard** integration — each host function checks its required permission
4. **Locale provider** — reads from user preferences

```rust
// Host function registration for the web renderer
pub fn register_web_host_functions(
    imports: &mut js_sys::Object,
    guard: Arc<PermissionGuard>,
    catalog: Arc<dyn CatalogApi>,
) {
    let catalog_clone = catalog.clone();
    let guard_clone = guard.clone();

    let catalog_get_work = Closure::wrap(Box::new(move |id_ptr: u32, id_len: u32| {
        guard_clone.assert("catalog.read").expect("permission denied");
        let id = read_wasm_string(id_ptr, id_len);
        // returns future — handled via wasm-bindgen Promise
        catalog_clone.get_work(&id)
    }) as Box<dyn Fn(u32, u32)>);

    js_sys::Reflect::set(imports, &"catalog_get_work".into(), catalog_get_work.as_ref()).unwrap();
    catalog_get_work.forget();
}

// Host function registration for the TUI renderer
pub fn register_tui_host_functions(
    linker: &mut Linker<TuiState>,
    guard: Arc<PermissionGuard>,
) -> Result<()> {
    linker.func_wrap_async("mugen", "catalog_get_work", {
        let guard = guard.clone();
        move |mut caller: Caller<'_, TuiState>, id_ptr: i32, id_len: i32| {
            let guard = guard.clone();
            Box::new(async move {
                guard.assert("catalog.read")?;
                let id = read_string(&mut caller, id_ptr, id_len)?;
                let work = caller.data().catalog.get_work(&id).await?;
                let json = serde_json::to_string(&work)?;
                write_string(&mut caller, &json)
            })
        }
    })?;
    Ok(())
}
```

---

## Phase 4 — Web host integration

The web integration is split between two WASM modules and a thin framework
adapter. JavaScript instantiates the plugin WASM and `unode_web_host.wasm`.
The plugin owns `manifest/load/render/dispatch`; `unode-web-host` owns
normalize, state seeding, binding tracking, IR lowering, and patch planning.

```rust
// crates/unode-web-host/src/session.rs
let mut session = WebSessionCore::new("en-US");
session.set_route(route);
let ir = session.mount(raw_screen, restored_state)?;
let initial = session.initial_patches()?;
let patches = session.apply_writes(writes)?;
```

The JavaScript bridge follows this shape:

```typescript
const plugin = await PluginInstance.instantiate(pluginWasmUrl, hostCalls);
const session = await HostSession.create(webHostModule, webHostWasmUrl, "en-US");

session.setRoute(route);
const screen = plugin.render({ route, data, stateSnapshot: {} });
const ir = session.mount(screen);
store.mount(ir);
store.apply(session.initialPatches());

const response = plugin.dispatch({ route, action, stateSnapshot: session.stateSnapshot() });
const writes = hostCallSink.drainStateWrites();
store.apply(session.applyWrites(writes));
```

The maintained adapters are React (`runtimes/web-react`) and Svelte
(`runtimes/web-svelte`). A Vue or custom adapter should consume the same IR and
patch ops.

---

## Phase 5 — TUI renderer

```
renderer/src/
  main.rs          — terminal setup, event loop
  app.rs           — top-level App struct, navigation state
  plugin_host.rs   — Wasmtime instance management, host function registration
  renderer/
    mod.rs         — entry point: walk CanonicalScreen → Ratatui widgets
    layout.rs      — terminal cell layout engine
    nodes/
      text.rs, value.rs, badge.rs, ...   — leaf node rendering
      stack.rs, inline.rs, grid.rs, ...  — container rendering
      list.rs, item.rs                   — collection rendering
      disclosure.rs, conditional.rs      — reactive composition
      media.rs                           — Kitty/Sixel via ratatui-image
  focus.rs         — focus ring management
  input.rs         — keyboard/mouse event handling
```

### Main event loop

```rust
loop {
    terminal.draw(|frame| {
        if let Some(screen) = &app.current_screen {
            renderer.render(screen, &state_store, frame);
        }
    })?;

    if event::poll(Duration::from_millis(16))? {
        match event::read()? {
            Event::Key(key) => {
                if let Some(action) = focus_manager.handle_key(key) {
                    plugin_host.dispatch(&app.active_plugin, action).await?;
                }
            }
            Event::Resize(_, _) => {
                // Recompute layout on next draw
            }
        }
    }
}
```

### ratatui-image setup

```toml
# Cargo.toml
[dependencies]
ratatui = "0.29"
ratatui-image = { version = "3", features = ["crossterm"] }
```

```rust
// In App initialization
let picker = Picker::from_query_stdio()?;
// picker.protocol_type() → Kitty, Sixel, or HalfBlocks fallback
```

---

## Implementation order

**Now** — stabilize protocol boundaries, ABI validation, web host packaging, and
test coverage around normalization, IR lowering, and patches.

**Next** — flesh out the app/domain bridge crates and method-level permission
metadata.

**Then** — connect the TUI runtime helpers to a full Ratatui loop and verify the
same plugin WASM across web and terminal hosts.
