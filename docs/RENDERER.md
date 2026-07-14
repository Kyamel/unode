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

## Web renderer (Svelte)

### Architecture

```
PluginScreenHost (Svelte)
  ├── loads plugin.wasm via WebAssembly.instantiate()
  ├── builds JS import object (host functions, gated by PermissionGuard)
  ├── calls plugin_load() → merges data into MemoryStateStore
  ├── calls plugin_render() → receives CanonicalScreen JSON
  ├── normalizes screen
  ├── creates SvelteStateAdapter (per-path Svelte stores)
  ├── calls trackReactiveBindings() with on_patch callback
  └── mounts PluginScreenLayout

PluginScreenLayout (Svelte)
  ├── slot layout (header.actions, sidebar.primary, etc.)
  └── CoreUiRenderer

CoreUiRenderer (Svelte)
  └── dispatches to node components by kind
      ├── CoreTextNode, CoreValueNode, CoreBadgeNode (leaf nodes)
      ├── CoreStackNode, CoreInlineNode, CoreGridNode (containers)
      ├── CoreListNode, CoreItemNode (collections)
      ├── CoreActionNode, CoreActionsNode (actions)
      ├── CoreDisclosureNode, CoreMenuNode (interactive composition)
      ├── CoreInputNode, CoreFormNode (inputs)
      ├── CoreConditionalNode (reactive branching)
      └── CoreSlotNode, CorePressableNode
```

### Per-path reactivity (fixes the global revision counter bug)

The previous implementation collapsed all state reactivity into one counter
(`rendererStateRevision`). Any state write caused every component to
re-evaluate.

The correct implementation uses per-path Svelte stores:

```typescript
class SvelteStateAdapter {
    private pathStores = new Map<string, Writable<unknown>>();

    getPathStore(path: string): Readable<unknown> {
        if (!this.pathStores.has(path)) {
            const store = writable(stateStore.get(path));
            stateStore.subscribe(path, value => store.set(value));
            this.pathStores.set(path, store);
        }
        return this.pathStores.get(path)!;
    }
}
```

Components subscribe only to the paths they read:

```svelte
<!-- Only re-renders when "work.title" changes -->
<script>
  const adapter = getStateAdapter();
  const titleStore = adapter.getPathStore("work.title");
  const title = $derived($titleStore ?? "");
</script>
<p>{title}</p>
```

Static nodes (`node._reactivity === "static"`) read from `_staticFields`
without any store subscription — they never re-render due to state changes.

### SvelteKit load integration

Screen resolution happens in `+page.ts load()`, not in `onMount`. This enables
`data-sveltekit-preload-data="hover"` to warm plugin screens before navigation,
making them feel as fast as native SvelteKit pages.

```typescript
// +page.ts
export const load: PageLoad = async ({ url }) => {
  await ensurePluginsActivated(); // cached after first call
  const screen = await wasmRuntime.resolveScreen(url.pathname, url.searchParams);
  return { screen };
};
```

### Plugin activation caching

`ensurePluginsActivated()` must be cached in session memory. The previous
implementation re-fetched `/plugins/registry.json` on every navigation, sidebar
render, and command palette open. This was a bug, not a design tradeoff.

```typescript
let activationPromise: Promise<void> | null = null;
let activated = false;

export async function ensurePluginsActivated(): Promise<void> {
    if (activated) return;
    if (activationPromise) return activationPromise;
    activationPromise = doActivation().then(() => { activated = true; });
    return activationPromise;
}
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

Both renderers use the same Rust crate for:

- `normalizeScreen()` — fill defaults, compute `_reactivity`
- `trackReactiveBindings()` — wire StateStore → ExprResolver subscriptions
- `resolveGap()`, `resolveGridColumns()`, `resolveAspectRatio()` — semantic layout helpers
- `formatNodeValue()` — locale-aware Intl formatting for `ValueNode`
- `PermissionGuard` — permission checking

The Web renderer calls these via the unode WASM module or a TypeScript
port. The TUI renderer calls them directly as Rust functions.

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
