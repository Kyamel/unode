# unode Reactivity

## The problem the previous implementation had

The TypeScript web renderer collapsed all state reactivity into a single revision
counter (`rendererStateRevision += 1`). Any write to any state path caused every
component that read any state to re-evaluate. A toggle on a disclosure re-rendered
the entire screen. This was a known bug in the TypeScript implementation, not an
architectural property of unode.

The Rust implementation fixes this at the foundation.

---

## How granular reactivity works

After `normalizeScreen()`, the host calls `trackReactiveBindings()`:

```
normalizeScreen(raw_json)
  -> CanonicalScreen with _reactivity metadata per node

trackReactiveBindings(screen, resolver, state, on_patch)
  -> walks the tree once
  -> calls resolver.track(node_key, path) for each binding found
  -> subscribes to each path in StateStore
  -> returns BindingSubscriptions { path_to_nodes, teardown }
```

When a state path changes:

```
state.set("ui.favorited", true)
  -> StateStore notifies subscribers of "ui.favorited"
  -> resolver.subscribers_of("ui.favorited") -> [node_key_A, node_key_B]
  -> on_patch({node_key_A, node_key_B}) called
  -> renderer re-evaluates ONLY those nodes
  -> renderer patches only the affected output (DOM nodes / terminal cells)
```

`render()` is never called again. The AST structure is fixed for the load cycle.
Reactivity is purely expression resolution against an updated StateStore.

---

## ExprResolver

The resolver maintains two maps:

```
node_to_path: Map<NodeKey, Set<StatePath>>
  "which paths does this node read?"

path_to_node: Map<StatePath, Set<NodeKey>>
  "which nodes need re-evaluation when this path changes?"
```

Key operations:

```
track(node_key, path)
  -> adds node_key -> path and path -> node_key to both maps

clear_tracking(node_key)
  -> removes node_key from both maps
  -> called before re-evaluating a node so stale subscriptions are cleaned up

dependencies_of(node_key) -> [path, ...]
  -> used after initial walk to set up StateStore subscriptions

subscribers_of(path) -> [node_key, ...]
  -> called on state change to find affected nodes
  -> includes ancestor prefix matching:
     a change at "work.title" wakes nodes subscribed to "work" or "work.title"
```

### In Rust

```rust
pub struct ExprResolver {
    node_to_path: HashMap<NodeKey, HashSet<StatePath>>,
    path_to_node: HashMap<StatePath, HashSet<NodeKey>>,
}

impl ExprResolver {
    pub fn track(&mut self, node_key: &str, path: &str) { ... }
    pub fn clear_tracking(&mut self, node_key: &str) { ... }
    pub fn dependencies_of(&self, node_key: &str) -> Vec<&str> { ... }
    pub fn subscribers_of(&self, path: &str) -> Vec<&str> {
        // includes ancestor prefix matching
    }

    pub fn resolve_string(&mut self, expr: &StringOrExpr, ctx: &ResolverCtx, node_key: &str) -> String { ... }
    pub fn resolve_bool(&mut self, expr: &BoolOrExpr, ctx: &ResolverCtx, node_key: &str) -> bool { ... }
    pub fn resolve_primitive(&mut self, expr: &PrimitiveOrExpr, ctx: &ResolverCtx, node_key: &str) -> JsonValue { ... }
}
```

---

## StateStore

The `MemoryStateStore` is the single source of truth for local screen state.
It lives in the host (Rust or TypeScript), not inside the plugin WASM.

```rust
pub struct MemoryStateStore {
    data: HashMap<String, JsonValue>,
    exact_listeners: HashMap<String, Vec<StateListener>>,
    prefix_listeners: HashMap<String, Vec<StateListener>>,
    pending_paths: HashSet<String>,
    batch_depth: usize,
}

impl StateStore for MemoryStateStore {
    fn get(&self, path: &str) -> Option<&JsonValue>;
    fn get_primitive(&self, path: &str, fallback: Primitive) -> Primitive;
    fn set(&mut self, path: &str, value: JsonValue);
    fn merge_data(&mut self, data: HashMap<String, JsonValue>);
    fn batch(&mut self, f: impl FnOnce(&mut Self));
    fn subscribe(&mut self, path: &str, listener: StateListener) -> Unsubscribe;
    fn subscribe_prefix(&mut self, prefix: &str, listener: StateListener) -> Unsubscribe;
    fn snapshot(&self) -> HashMap<String, JsonValue>;
    fn reset(&mut self);
}
```

Key behaviors:

- Paths are dot-separated: `"work.title"`, `"ui.favorited"`, `"items.0.name"`
- `subscribe_prefix("")` subscribes to all paths (used only for debugging)
- Batch writes collapse into a single notification cycle
- `reset()` restores the initial seed state

---

## Static subtree optimization

Nodes with `_subtreeReactivity == "static"` have no reactive descendants.
The renderer can skip them entirely during the tracking walk and during
re-evaluation passes. This is the most impactful optimization for typical
plugin screens where the majority of content is static text and media.

```
walk(screen):
  if node._subtreeReactivity == "static":
    return  // skip -- nothing reactive in this subtree

  if node._reactivity != "static":
    resolve_node_expressions(node, resolver, ctx)

  for child in children(node):
    walk(child)
```

---

## WASM boundary and state

Plugin action handlers run inside WASM. When a handler calls `ctx.state.set()`,
this crosses the WASM boundary via a host function:

```
Plugin WASM
  ctx.state.set("ui.favorited", true)
    -> host function: state_set("ui.favorited", "true")

Host (Rust or JS)
  -> StateStore.set("ui.favorited", true)
  -> subscriber notifications
  -> renderer patches affected nodes
```

This means each `set()` call in an action handler is a WASM host function call.
For typical interactions (one to three state writes per action), this overhead
is imperceptible. Tight loops with many writes should batch them:

```rust
// Plugin side -- batch multiple writes
ctx.dispatch(ActionRef {
    type_: "unode.batchState".into(),
    params: json!({ "isFavorited": true, "favoriteCount": 42 }),
    ..Default::default()
});
```

---

## Locale reactivity

The locale is not stored in the StateStore -- it is part of the `ResolverContext`
and exposed to plugins via the app bridge (see `I18N.md`). When the locale
changes, the host triggers a full re-render of the current screen by calling
`render()` again with updated context. This is intentional: locale changes are
rare and typically affect most visible text, making a full re-render cheaper
than tracking per-string locale dependencies.
