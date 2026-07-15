# unode Renderer

## Renderer role

The renderer is the platform adapter and the trust boundary. It receives a
canonical JSON AST and decides how to draw it, how users interact with it,
and what capabilities the plugin is allowed to use.

The renderer is the last line of defense. Even if a plugin declares a
permission it was not granted, the host function is not injected and the
capability is unavailable.

---

## Shared renderer responsibilities

Both renderers — Web and TUI — must:

1. Instantiate the plugin WASM module with filtered host functions
2. Call `plugin_load()` → receive data JSON → merge into StateStore
3. Call `plugin_render()` → receive `CanonicalScreen` JSON
4. Call `normalizeScreen()` on the JSON
5. Call `trackReactiveBindings()` to wire state → node subscriptions
6. Mount the canonical screen
7. On state change: call `resolver.subscribers_of(path)` → patch only affected nodes
8. On action: dispatch to plugin via `plugin_dispatch()`
9. On navigation: teardown, reset StateStore, repeat from step 2

Neither renderer calls `render()` again in response to state changes. Only
navigation or explicit refresh triggers a new load/render cycle.

---

## Renderer SDK target

The renderer surface should become easy enough that an application team can
define how plugin UI looks in their product without rebuilding the plugin
runtime. The application should import a TypeScript SDK, provide component
mappings for semantic nodes, and keep the hard runtime work in Unode.

The target split is now starting to exist in `packages/`:

- **Web runtime core:** `packages/unode-core`, a shared TypeScript package
  for plugin WASM instantiation, host-session loading, plugin registries,
  state-write buffering, and action dispatch. This is runtime glue, not
  renderer customization.
- **Renderer core SDK:** `packages/unode-renderer`, a shared TypeScript
  package with `IrScreen`, `IrNode`, `IrPatchOp`, `ScreenStore`, patch
  application, node-key helpers, value unwrapping, prop normalization,
  unknown-node fallback behavior, and diagnostics.
- **Framework adapters:** `packages/web-react` now exposes
  `createReactRenderer()`. `packages/web-svelte` consumes the shared runtime and
  renderer cores. Vue or other clients should follow the same shape: subscribe
  to node keys, mount children, and expose framework idioms, but do not own
  Unode semantics.
- **Application renderer spec:** app-owned mapping from semantic node types to
  design-system components. This is where the host decides how plugin-provided
  `text`, `section`, `action`, `list`, `input`, `status`, and other nodes appear.

Conceptually, a React host should be able to write something like:

```tsx
import { createReactRenderer } from "unode-react";
import { Button, Card, Text } from "./design-system";

export const { UnodeScreen } = createReactRenderer({
  nodes: {
    text({ props }) {
      return <Text tone={props.tone} role={props.role}>{props.content}</Text>;
    },
    section({ props, children }) {
      return <Card title={props.title}>{children}</Card>;
    },
    action({ props, dispatch }) {
      return (
        <Button
          intent={props.intent}
          disabled={props.disabled}
          onClick={() => dispatch(props.action)}
        >
          {props.label}
        </Button>
      );
    },
  },
});
```

The exact API can change, but these constraints should not:

- Apps control presentation, density, design-system components, and platform
  interaction details.
- Plugins control only semantic intent and symbolic actions.
- The SDK owns patch correctness, keyed subscriptions, fallback rendering, and
  common safety checks.
- Framework-specific packages stay thin enough that new adapters can be written
  without copying runtime logic.
- Renderer specs do not receive plugin internals, raw WASM access, or permission
  enforcement hooks.

This matches the broader Unode architecture: the plugin declares intent, the
host runtime enforces capabilities, and the renderer decides how intent becomes
native UI.

---

## Web host + framework adapters

### Architecture

```
plugin.wasm
  ├── exports plugin_manifest/load/render/dispatch
  └── imports unode.host_call

unode_web_host.wasm
  ├── WebSession.mount(screen_json, seed_json)
  ├── WebSession.initialPatches()
  ├── WebSession.applyWrites(writes_json)
  └── WebSession.stateSnapshot()

JavaScript bridge
  ├── instantiates both WASM modules
  ├── implements host_call
  ├── dispatches user actions to plugin_dispatch()
  ├── drains state writes
  └── applies returned IrPatchOps to the adapter store

Framework adapter
  ├── React adapter is maintained today
  ├── Svelte adapter is maintained today
  ├── Vue/custom adapters can consume the same IR contract
  └── renderer components subscribe by node key, not by global revision
```

The maintained web proofs live in `packages/web-react` and
`packages/web-svelte`. React and Svelte are adapter choices, not core
dependencies. The Rust web host owns normalization, dependency tracking, state
snapshots, and patch planning.

### Keyed reactivity

The previous implementation collapsed all state reactivity into one counter
(`rendererStateRevision`). Any state write caused every component to
re-evaluate.

The current web slice uses a keyed `ScreenStore`. Each node subscribes to its own
key through the framework adapter. When `unode-web-host` returns a `SetProp`,
`ReplaceNode`, or `ReplaceChildren` patch, the store wakes only the affected key.

```typescript
const patches = session.applyWrites({ "ui.countLabel": "Count: 1" });
screenStore.apply(patches);

function UnodeNode({ nodeKey }: { nodeKey: string }) {
  const node = useSyncExternalStore(
    (wake) => screenStore.subscribe(nodeKey, wake),
    () => screenStore.get(nodeKey),
  );
  return renderNode(node);
}
```

Bindings are still tracked by state path inside Rust; the JavaScript adapter sees
only the resulting dirty node keys and IR patch operations.

### Framework integration

A production web integration should resolve screens before mounting the visual
component tree where the host framework supports that pattern. In SvelteKit this
means `+page.ts load()`. In React apps this may mean route loaders, suspense
resources, or an app-specific data layer.

Plugin activation should be cached per browser session. The legacy implementation
could re-fetch plugin registries too often; the target runtime should treat
activation as host state, not component-local state.

### Current web verification

```sh
cargo test -p unode-web-host
cargo test --manifest-path plugins/web-counter/Cargo.toml
nix-shell --run 'node packages/web-react/scripts/smoke.mjs'
nix-shell --run 'node packages/web-svelte/scripts/smoke.mjs'
```

---

## TUI renderer (Rust + Ratatui)

### Architecture

```
TuiApp (Rust, main thread)
  ├── Wasmtime engine + plugin instances
  ├── MemoryStateStore (Rust)
  ├── ExprResolver (Rust)
  ├── FocusManager (Rust)
  ├── Navigator (Rust)
  └── Ratatui terminal loop

Per frame:
  1. Read input events
  2. Handle keyboard → dispatch ActionRef to plugin
  3. Render: walk CanonicalScreen → Ratatui widgets
  4. terminal.draw(frame)
```

### Single-threaded rendering

All Ratatui calls happen on the main Rust thread. Plugin WASM executes
synchronously when called (Wasmtime supports async, but for action handlers
the sync model is simpler and sufficient). Network calls from plugins go
through Tokio async on the main thread via host functions.

### Image support via ratatui-image

`MediaNode` with Kitty Protocol support:

```rust
fn render_media_node(
    node: &CanonicalNode<MediaNode>,
    area: Rect,
    frame: &mut Frame,
    image_state: &mut ImageState,
) {
    match &node.ref_ {
        MediaRef::AtBlob { did, cid } => {
            // Fetch async, cache, render when ready
            if let Some(image) = image_cache.get(&(did, cid)) {
                let image_widget = StatefulImage::new(image.clone());
                frame.render_stateful_widget(image_widget, area, image_state);
            } else {
                // Show placeholder while loading
                let placeholder = Block::bordered().title(node.alt.as_str());
                frame.render_widget(placeholder, area);
                spawn_image_fetch(did, cid, image_cache.clone());
            }
        }
        MediaRef::Placeholder { label, .. } => {
            let block = Block::bordered().title(label.as_deref().unwrap_or(""));
            frame.render_widget(block, area);
        }
        _ => { /* url and asset refs */ }
    }
}
```

### Layout engine

The TUI layout engine translates unode semantic layout to terminal cell
coordinates. Breakpoints use character columns instead of pixels:

```rust
fn resolve_grid_columns(
    columns: &ResponsiveGridColumns,
    terminal_cols: u16,
    breakpoints: &RendererBreakpoints,
) -> u16 {
    if terminal_cols >= breakpoints.xl { return columns.xl.unwrap_or(1); }
    if terminal_cols >= breakpoints.lg { return columns.lg.unwrap_or(1); }
    if terminal_cols >= breakpoints.md { return columns.md.unwrap_or(1); }
    if terminal_cols >= breakpoints.sm { return columns.sm.unwrap_or(1); }
    columns.base.unwrap_or(1)
}
```

### DisclosureNode in TUI

`DisclosureNode` expands inline, pushing content below downward. The TUI
renderer tracks expanded state in the local StateStore and recomputes layout
when `disclosure.binding` changes:

```
▶ Mostrar mais detalhes        ← trigger line
                                ← nothing when collapsed

▼ Ocultar detalhes             ← trigger line
  248 páginas                   ← content when expanded
  Editora Panini
```

---

## Shared renderer utilities (Rust crate)

Host packages use the same Rust core for:

- `normalizeScreen()` — fill defaults, compute `_reactivity`
- `trackReactiveBindings()` — wire StateStore → ExprResolver subscriptions
- `PermissionGuard` — permission checking

The Web host calls these through `unode_web_host.wasm`. The TUI runtime calls
them directly as Rust functions. Framework adapters should not port these
semantics.

---

## Keyboard and focus

| Key | Web behavior | TUI behavior |
|---|---|---|
| Tab / Arrow ↓ | Focus next focusable | Move cursor to next item |
| Shift+Tab / Arrow ↑ | Focus previous | Move cursor to previous item |
| Enter | Activate focused element | Activate focused element |
| Escape | Close overlay / navigate back | Close overlay / navigate back |
| Arrow ←→ | Move inside inline containers | Move inside inline containers |

No keyboard events are consumed while focus is inside an editable input.

---

## Platform parity

Cross-platform means preserving intent, not pixel parity.

| unode node | Web | TUI |
|---|---|---|
| `grid` with `columns: {base:1, md:3}` | CSS grid, 3 cols at ≥768px | 1 col on narrow terminals, 3 cols at ≥120 chars |
| `media` | `<img>` with aspect ratio | Kitty/Sixel image or bordered placeholder |
| `disclosure` | Animated chevron + collapse | `▶`/`▼` prefix + inline expand |
| `menu` | Popover dropdown | Modal list overlay |
| `badge` | Rounded pill chip | `[LABEL]` prefix |
| `value format="date"` | `Intl.DateTimeFormat` | Same via Rust `chrono` |
| `action intent="danger"` | Red button | Red text (if terminal supports color) |
