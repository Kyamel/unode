# unode Slot System

There are two distinct slot systems in unode. They solve different problems,
work differently under the hood, and have different implications for the TUI
and Web renderers.

---

## System 1 — SlotNode (intra-screen injection)

A `SlotNode` is a named placeholder inside a plugin's rendered screen. Other
plugins — or the host — can inject UI nodes into that placeholder at mount time.

### Use case

Plugin A renders a work detail screen. It declares a slot at the bottom:

```rust
ui::slot("catalog.work-detail:footer")
```

Plugin B wants to add a "Add to reading list" button to every work detail
screen. It registers a contribution to that slot. The user sees Plugin A's
screen with Plugin B's button at the bottom, without Plugin A knowing Plugin B
exists.

### How it works

```
1. Plugin A renders CanonicalScreen containing SlotNode { name: "catalog.work-detail:footer" }

2. Host normalizes the screen — SlotNode is present with optional fallback

3. Before mounting, host queries SlotRegistry:
   slot_registry.resolve("catalog.work-detail:footer", screen_ctx)
   → [{ plugin_id: "com.mugenx.reading-list", priority: 10 }, ...]

4. For each contributing plugin, host calls:
   plugin_b.render_slot("catalog.work-detail:footer", ctx_json)
   → returns UiNode JSON

5. Host injects the returned nodes into the SlotNode position in the tree,
   replacing or wrapping the fallback if present

6. Renderer mounts the full tree including injected nodes
```

### SlotNode in the AST

```rust
pub struct SlotNode {
    pub id: Option<String>,
    pub name: String,
    pub fallback: Option<Box<UiNode>>,  // rendered if no contributions
}
```

The slot name is a namespaced string by convention:
`"<plugin-id>:<slot-identifier>"` — e.g. `"catalog.work-detail:footer"`.

### Contribution declaration

Plugins declare slot contributions in their manifest. The declaration is
serializable host metadata; the rendered UI still comes from
`plugin_render_slot` at mount time.

```rust
PluginManifest {
    id: "com.mugenx.reading-list".into(),
    slot_contributions: vec![SlotContributionDecl {
        id: "reading-list.add-button".into(),
        target: "catalog.work-detail:footer".into(),
        priority: 10,
        when: None,
    }],
    ..Default::default()
}
```

`when` uses the same serializable boolean expression shape as the AST. It is not
a Rust closure, because manifests must cross host and WASM boundaries as JSON.

### WASM protocol for SlotNode

Each slot contribution render call crosses the WASM boundary:

```rust
// Host calls this export on the contributing plugin
pub extern "C" fn plugin_render_slot(request_ptr: u32, request_len: u32) -> u32;
pub extern "C" fn plugin_render_slot_result_len() -> u32;
```

The request is `PluginRenderSlotRequest` JSON and includes `contributionId`,
`slotName`, `route`, `stateSnapshot`, and `locale`. The response is
`PluginRenderSlotResponse` JSON:

```json
{ "nodes": [] }
```

An empty node list means the contribution opts out dynamically. The host
collects all contributions, deserializes each response, normalizes the returned
`UiNode`s, annotates them with the contributor origin, resolves nested slots,
and assembles the final tree before passing it to the renderer.

### SlotNode in a Web adapter

The Web adapter receives the already-injected tree or IR. A slot renderer renders
the injected children as normal Unode nodes. If the slot has no contributions
and no fallback, it renders nothing.

```tsx
function SlotNode({ node }: { node: IrNode }) {
  const children = injectedChildrenFor(node) ?? node.c;
  return children.map((child) => <UnodeNode key={child.p._k} node={child} />);
}
```

Injection normally happens before the framework tree is constructed, so the
adapter receives an already-resolved tree rather than unresolved placeholders.

### SlotNode in the TUI renderer (Ratatui)

Same model — the host injects before mounting. The Ratatui renderer sees a
fully resolved tree with no unresolved SlotNodes. The layout engine allocates
space for the injected nodes as if they were authored inline.

If a slot has no contributions and no fallback, the layout engine treats it
as zero-height — it occupies no terminal cells.

---

## System 2 — Shell slots (host chrome injection)

Shell slots are not AST nodes. They are a mechanism for plugins to contribute
UI to regions of the host app's own chrome — the navigation sidebar, the header
action bar, the bottom navigation on mobile.

These regions exist outside any plugin screen. They are owned by the app shell
(the web application's layout components, or the `mgn` TUI app frame). Plugins
declare UI slot contributions in the manifest, while data-only shell entries
such as navigation items live in host-owned registries.

### Use case

The catalog plugin wants to appear in the app's sidebar navigation. It
registers a navigation item:

```rust
ctx.navigation.register(NavigationItem {
    id: "catalog.browse.nav".into(),
    label: msg("nav_label"),
    to: "/app/mangas/browse".into(),
    icon: Some("library".into()),
    priority: 100,
    ..Default::default()
});
```

The web app sidebar queries the navigation registry and renders these items using
its own components. The plugin does not control how the sidebar looks — it only
declares that it wants to be present and what navigating to it means.

### Shell slot targets

Shell slot targets are defined by the host app, not by unode. For Mugen:

| Target | Location | What plugins contribute |
|---|---|---|
| `shell:sidebar-nav` | Main sidebar | Navigation item with icon and label |
| `shell:header-actions` | Screen header right | Action button (e.g. search, filter) |
| `shell:mobile-nav` | Bottom bar on mobile | Navigation item |
| `shell:command-palette` | Command palette | Searchable command entry |

### How shell slots work

Unlike `SlotNode`, shell slot contributions are not rendered via WASM calls on
each screen mount. They are registered once at activation and queried by the
host shell components whenever they need to render.

```
1. Plugin activates
   → registers NavigationItem, CommandDefinition, SlotContribution

2. Host shell renders sidebar
   → queries NavigationRegistry.get_available(ctx)
   → receives list of NavigationItems from all active plugins
   → renders them using its own framework components

3. User navigates to plugin route
   → normal route lifecycle (load, render, mount)
   → shell stays in place, only the main content area changes
```

The shell does not call into plugin WASM to render navigation items. The items
are data (label string, path, icon name) — the shell decides how to render them.

### Shell slots that DO call into WASM

One exception: `shell:header-actions` and similar content-area slots where the
plugin needs to contribute actual UiNodes (not just data). These work like
`SlotNode` but target the shell chrome instead of another plugin's screen:

```rust
SlotContributionDecl {
    id: "catalog.header-search".into(),
    target: "shell:header-actions".into(),
    priority: 10,
    when: None,
}
```

At mount time the host calls `plugin_render_slot` with a
`PluginRenderSlotRequest` for each plugin that declared this target, collects
the UiNode JSON responses, and renders them via the active framework adapter in
the header.

---

## Comparison

| | SlotNode | Shell slots (data) | Shell slots (UI) |
|---|---|---|---|
| Defined by | Plugin that owns the screen | Host app | Plugin that contributes |
| Target location | Inside another plugin's screen | App chrome | App chrome |
| Contribution type | UiNode (rendered via WASM) | Data (label, path, icon) | UiNode (rendered via WASM) |
| Render call per mount | Yes | No (registered once) | Yes |
| WASM boundary crossed | Yes, per contribution | No | Yes, per contribution |
| Renderer sees | Injected into canonical tree | Queried by shell components | Passed to CoreUiRenderer |
| TUI equivalent | Same — injected before Ratatui render | Sidebar/nav items in TUI layout | Same — injected into TUI frame |

---

## Web adapter implementation

### SlotNode

Injection happens in the runtime before the framework component tree is created:

```typescript
// After normalize and before lower/mount.
const injectedScreen = await injectSlots(canonicalScreen, slotRegistry, ctx);
// injectedScreen has no unresolved SlotNodes — all replaced with contributions
mountScreen(injectedScreen);
```

An adapter-level slot component exists only as a fallback for slots that were not
injected, for example because a contributing plugin loaded after mount. Dynamic
slot injection after mount is possible but rare.

### Shell slots (navigation)

The web shell queries the registry reactively:

```tsx
function Sidebar() {
  const navItems = navigationRegistry.getAvailable({ host: hostApi });
  return navItems.map((item) => <SidebarNavItem key={item.id} item={item} />);
}
```

This is pure data rendering — no WASM call per render, no `CoreUiRenderer`.

### Shell slots (UI contributions)

```typescript
// In PluginScreenHost, reads header action contributions
const headerActions = await slotRegistry.resolve("shell:header-actions", ctx);
// headerActions: CanonicalUiNode[] — each came from a plugin_render_slot call
```

```tsx
function HeaderActions({ actions }: { actions: IrNode[] }) {
  return actions.map((node) => <UnodeNode key={node.p._k} node={node} />);
}
```

---

## TUI renderer implementation

### SlotNode

Same injection model as Web — happens before Ratatui render:

```rust
// In TuiApp, after normalize
let injected = inject_slots(canonical_screen, &slot_registry, &ctx).await;
renderer.mount(injected, &state_store);
```

The Ratatui renderer never sees unresolved `SlotNode`s.

### Shell slots (navigation)

The TUI app frame has a fixed layout: sidebar on the left, main content on
the right. The sidebar queries the `NavigationRegistry`:

```rust
// In TUI sidebar render
fn render_sidebar(
    registry: &NavigationRegistry,
    ctx: &NavigationContext,
    area: Rect,
    frame: &mut Frame,
) {
    let items = registry.get_available(ctx);
    // renders as a selectable list using Ratatui List widget
    let list = List::new(
        items.iter().map(|item| ListItem::new(item.label.as_str()))
    ).highlight_style(Style::default().bold());
    frame.render_widget(list, area);
}
```

### Shell slots (UI contributions)

The TUI frame has designated regions for plugin-contributed chrome. The app
calls `plugin_render_slot` for each, receives UiNode JSON, normalizes, and
passes to the Ratatui renderer:

```rust
// In TuiApp frame render
let header_nodes = slot_registry
    .resolve_ui("shell:header-actions", &ctx)
    .await;  // calls plugin_render_slot on each contributing plugin

for node in &header_nodes {
    renderer.render_node(node, header_area, frame, &state_store);
}
```

---

## Key difference: who controls rendering

| System | Who renders | How |
|---|---|---|
| `SlotNode` | Plugin declares slot; contributors render their part; renderer handles the whole tree | WASM call per contribution |
| Shell nav slots | Host app renders navigation items | Data from registry, host's own components |
| Shell UI slots | Contributors render their part; host places it in the frame | WASM call per contribution |

The navigation sidebar in both Web and TUI is always rendered by the host using
its own visual style. A plugin cannot change how the sidebar looks — it can only
say "I want to be in the sidebar with this label and this path." The host decides
whether that becomes a rounded button, a flat list item, or a tab bar entry.

This is the same "intent over presentation" principle that governs the AST — it
applies to shell slot contributions just as much as to screen nodes.

---

## Registration summary

Everything a plugin contributes to slots is declared in its manifest:

```rust
PluginManifest {
    slot_contributions: vec![
        // Shell UI contribution (WASM called to render)
        SlotContributionDecl {
            id: "catalog.header-search".into(),
            target: "shell:header-actions".into(),
            priority: 10,
            when: None,
        },
        // Intra-screen: contributes to another plugin's SlotNode
        SlotContributionDecl {
            id: "catalog.reading-list-button".into(),
            target: "catalog.work-detail:footer".into(),
            priority: 10,
            when: None,
        },
    ],
    ..Default::default()
}
```

Navigation items and command palette entries still live in their own registries.
Slot UI contributions use `slotContributions`. The distinction between shell and
intra-screen slot UI is encoded in the target name convention: `"shell:*"`
targets are shell chrome, everything else is a `SlotNode` in another plugin's
screen.

## Origin and permissions

Injected nodes belong to the contributing plugin, not to the plugin that
declared the slot. The host preserves this as internal origin metadata and
lowers it into renderer IR so action dispatch and capability checks can use the
contributor namespace. Equal local node IDs from different contributors are
namespaced before normalization.

## ABI compatibility

Raw ABI `0.2.0` makes `plugin_render_slot` and
`plugin_render_slot_result_len` required exports. Plugins that do not declare
slot contributions normally return `{ "nodes": [] }`; the Rust SDK macro
generates that default handler when no `render_slot` function is supplied.
