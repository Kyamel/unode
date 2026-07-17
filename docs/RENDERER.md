# unode Renderer

## Renderer role

The renderer is the platform adapter. It receives trusted host output
(`IrScreen`, `IrNode`, and `IrPatchOp`) and decides how semantic intent becomes
native UI: DOM, framework portals, terminal widgets, focus behavior, keyboard
mapping, and platform accessibility.

The renderer is not the security authority. The trusted host runtime loads
plugin WASM, injects or withholds host functions, enforces permissions, owns
state and resource policy, and recovers crashed plugin sessions. Renderer specs
do not receive plugin internals, raw WASM access, permission profiles, or
capability-enforcement hooks.

---

## Declaring a renderer (Rust hosts)

Rust hosts declare their renderer with the `unode-renderer` crate -- the same
recipe model as the web `defineRenderer()`. The core crate is presentation-
stack agnostic (a `Backend` picks the surface type), so the recipe machinery
also serves hand-rolled terminal writers. `unode-ratatui-renderer` ships the ratatui
backend with default recipes:

```rust
use unode_ratatui_renderer::{NodeKind, TuiRecipe, ratatui_renderer, rect};

let renderer = ratatui_renderer()              // ratatui defaults for every node
    // full replacement (typed: closures receive &TextNode, no annotations)
    .recipes([TuiRecipe::text(
        |_, node, width| measure_my_text(node, width),
        |ctx, node, area| paint_my_text(ctx.surface, rect(area), node),
    )])
    // restyle only the painting; the registered measure is kept
    .override_render(NodeKind::Badge, |ctx, node, area| { /* ... */ })
    // decorate the default instead of replacing it
    .wrap(NodeKind::Action, |inner, ctx, node, area| {
        // adornments before/after, then delegate:
        inner(ctx, node, area);
    })
    .build();
```

Each recipe pairs `measure` (rows needed at a width) with `render` (paint into
a `Region` of the backend surface); `RenderCtx` provides recursion into child
nodes and the focus cursor for interactive elements. Prefer `wrap` /
`override_render` for styling tweaks and full recipes for structural changes.

---

## Shared renderer responsibilities

Both host integrations -- Web and TUI -- must provide a renderer with:

1. An initial `IrScreen` produced by the trusted runtime
2. `IrPatchOp`s produced after state writes
3. Action dispatch callbacks supplied by the host runtime
4. Host-slot/portal hooks where the app intentionally renders native components
5. Focus and interaction state that never bypasses runtime permission checks

Neither renderer calls plugin `render()` directly. Only the host runtime may
call lifecycle exports, and `render()` is not called again in response to
ordinary state changes. Navigation, locale changes, or explicit refresh trigger
a new load/render cycle.

---

## Renderer SDK target

The renderer surface should be easy enough that an application team can define
how plugin UI looks in their product without rebuilding the plugin runtime. The
default package should already render a functional UI; applications should then
override only the semantic recipes that need to match their design system.

The target split is now starting to exist in `packages/`:

- **Web runtime core:** `packages/unode-web-core`, a shared TypeScript package
  for plugin WASM instantiation, host-session loading, plugin registries,
  state-write buffering, and action dispatch. This is runtime glue, not
  renderer customization or presentation authority.
- **Renderer core SDK:** `packages/unode-web-renderer` is *the* renderer. It is a
  single, framework-free TypeScript package: `IrScreen`/`IrNode`/`IrPatchOp`,
  the keyed `ScreenStore`, prop normalization, the ergonomic recipe context, the
  `defineRenderer()` recipe/builder/override API, the `h()`/`hostSlot()` virtual
  nodes, and a DOM backend that walks the IR, subscribes each node to its key,
  and reconciles recipe output into real DOM. Because recipes return neutral
  VNodes and the backend targets the DOM, the same renderer runs in any web
  context -- vanilla, React, Svelte, Vue.
- **Framework packages are mount targets, not renderers.** `packages/unode-react`
  and `packages/unode-svelte` are thin `<UnodeScreen>` wrappers plus portal glue.
  The wrapper mounts the DOM renderer into a host element and provides a
  **portal** that fulfills `hostSlot(name)` holes with the host app's own native
  components. There is no per-framework renderer to keep in sync; a Vue package
  would only add its own portal wrapper.
- **Examples are apps.** `examples/web-react` and `examples/web-svelte` wire the
  importable packages to `unode-web-core`, generated WASM artifacts, and the
  counter plugin (`plugins/counter`).
- **Application renderer spec:** app-owned mapping from semantic node types to
  design-system components. This is where the host decides how plugin-provided
  `text`, `section`, `action`, `list`, `input`, `status`, and other nodes appear.

The preferred customization path is recipe-based. Recipes are written once, in
the universal TS language, and receive both the raw node details and promoted
semantic helpers like `label`, `content`, `intent`, `disabled`, and `run()`.
They return neutral VNodes via `h()`, or defer to a host-native component via
`hostSlot()`:

```ts
import { defineRenderer, h, hostSlot } from "unode-web-renderer";

export const renderer = defineRenderer()
  .recipe("text", ({ content, role }) =>
    h("p", { class: `prose prose--${role}` }, content))
  .recipe("section", ({ title, children }) =>
    h("section", { class: "card" }, title ? h("h2", {}, title) : null, children))
  // Deep integration: this plugin node renders as the host app's OWN component.
  .recipe("action", ({ label, intent, disabled, run }) =>
    hostSlot("Button", { intent, disabled, onClick: run, children: label }))
  .build();
```

Built-in node names are typed. Use `.custom("app.node", recipe)` when an app
intentionally introduces a node type outside the core Unode set.

Mount it wherever the app lives. Frameworks supply the portal that maps
`hostSlot` names to native components:

```tsx
// React demo
<UnodeScreen renderer={renderer} store={store} onAction={onAction}
  components={{ Button: MyButton, Card: MyCard }} />
```

```ts
// Vanilla -- no framework, no portal needed for pure-DOM recipes
renderer.mount(document.getElementById("app")!, store, { onAction });
```

### `hostSlot` -- the deep-integration primitive

`hostSlot(name, props)` is a first-class VNode: a hole the renderer fills with a
host-provided component. The DOM backend creates a placeholder element and hands
it to the active `HostPortalAdapter`; the React wrapper turns that into a React
portal, the Svelte wrapper mounts a Svelte component, and a plain DOM app can
supply its own factory. This is how a plugin node becomes the host's own
design-system `Button` -- and, symmetrically, how plugin UI is placed into host
chrome regions (`renderer.mountNodes(headerEl, contributions, store)` for
`shell:header-actions`). The plugin still only expresses intent; the host owns
presentation and keeps a single renderer to maintain.

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

The maintained web proofs live in `examples/web-react` and
`examples/web-svelte`. React and Svelte are adapter choices, not core
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
cargo test --manifest-path plugins/counter/Cargo.toml
nix-shell --run 'node examples/web-react/scripts/smoke.mjs'
nix-shell --run 'node examples/web-svelte/scripts/smoke.mjs'
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
  2. Handle keyboard -> dispatch ActionRef to plugin
  3. Render: walk CanonicalScreen -> Ratatui widgets
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
> Mostrar mais detalhes        <- trigger line
                                <- nothing when collapsed

v Ocultar detalhes             <- trigger line
  248 páginas                   <- content when expanded
  Editora Panini
```

---

## Shared renderer utilities (Rust crate)

Host packages use the same Rust core for:

- `normalizeScreen()` -- fill defaults, compute `_reactivity`
- `trackReactiveBindings()` -- wire StateStore -> ExprResolver subscriptions
- `PermissionGuard` -- permission checking

The Web host calls these through `unode_web_host.wasm`. The TUI runtime calls
them directly as Rust functions. Framework adapters should not port these
semantics.

---

## Keyboard and focus

| Key | Web behavior | TUI behavior |
|---|---|---|
| Tab / Arrow Down | Focus next focusable | Move cursor to next item |
| Shift+Tab / Arrow Up | Focus previous | Move cursor to previous item |
| Enter | Activate focused element | Activate focused element |
| Escape | Close overlay / navigate back | Close overlay / navigate back |
| Arrow <-> | Move inside inline containers | Move inside inline containers |

No keyboard events are consumed while focus is inside an editable input.

---

## Platform parity

Cross-platform means preserving intent, not pixel parity.

| unode node | Web | TUI |
|---|---|---|
| `grid` with `columns: {base:1, md:3}` | CSS grid, 3 cols at ≥768px | 1 col on narrow terminals, 3 cols at ≥120 chars |
| `media` | `<img>` with aspect ratio | Kitty/Sixel image or bordered placeholder |
| `disclosure` | Animated chevron + collapse | `>`/`v` prefix + inline expand |
| `menu` | Popover dropdown | Modal list overlay |
| `badge` | Rounded pill chip | `[LABEL]` prefix |
| `value format="date"` | `Intl.DateTimeFormat` | Same via Rust `chrono` |
| `action intent="danger"` | Red button | Red text (if terminal supports color) |
