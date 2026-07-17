# unode AST

## Hard constraints

Every node in the canonical AST must be:

- **JSON-serializable** -- no functions, closures, class instances, or symbols
- **Semantically typed** -- `kind` discriminates the union; field names describe
  intent, not appearance
- **Renderer-agnostic** -- no CSS class names, pixel values, or terminal escapes
- **Domain-agnostic** -- no references to works, chapters, users, or any app concept

## Two authoring layers

### Canonical AST

The small portable protocol every renderer implements. Must stay stable,
explicit, and easy to serialize and diff.

### Sugar DSL

The ergonomic surface plugin authors actually write. The DSL produces frozen
canonical nodes. Adding DSL helpers never changes the canonical protocol.

---

## Expression model

Three expression kinds cover all reactive and route-driven values:

```
literal  -> static scalar, resolved at normalization time
binding  -> dot-path into the screen StateStore, resolved at render time
param    -> route param or query string key, resolved at mount time
```

The expression model is intentionally minimal. i18n key lookup is not an
expression kind; plugins resolve translations themselves using the locale
string from `ctx.locale()` and their own catalogs.

### In Rust

```rust
#[derive(Serialize, Deserialize, Clone)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum UiExpr {
    Literal { value: serde_json::Value },
    Binding { path: String },
    Param    { name: String },
}

pub type StringOrExpr = Either<String, UiExpr>;
pub type BoolOrExpr   = Either<bool,   UiExpr>;
pub type NumberOrExpr = Either<f64,    UiExpr>;
```

---

## Node taxonomy

### Structure

| Node | Purpose | TUI equivalent |
|---|---|---|
| `screen` | Root of a plugin view | Full-screen view |
| `section` | Semantic grouping | Labelled block |
| `stack` | Vertical composition | Lines |
| `inline` | Horizontal flow, wrappable | Same line |
| `grid` | Multi-column layout | 1–N columns by terminal width |
| `scroll` | Overflow region | Scrollable pane |

### Content

| Node | Purpose | TUI equivalent |
|---|---|---|
| `text` | Textual content with semantic role | Styled text |
| `value` | Locale-formatted scalar | Formatted string |
| `icon` | Semantic icon by name | Unicode glyph / ASCII |
| `badge` | Short label with tone | `[LABEL]` prefix |
| `divider` | Visual/semantic separator | `─────` line |
| `media` | Image/cover/avatar via typed ref | Kitty/Sixel/placeholder |

### Collections

| Node | Purpose |
|---|---|
| `list` | Navigable list of `item` nodes |
| `item` | Structured row: leading, primary, secondary, trailing |

`ItemNode.id` is required: items appear in dynamic collections where order
can change and the renderer needs stable identity for reconciliation.

### Actions

| Node | Purpose |
|---|---|
| `action` | User-triggerable symbolic action |
| `actions` | Container for grouped actions |

### Input

| Node | Purpose |
|---|---|
| `input` | Form field, discriminated by `inputKind` |
| `form` | Input container with submit action |

### Feedback

| Node | Purpose |
|---|---|
| `status` | Inline feedback with severity |
| `empty` | Absence-of-content placeholder |
| `loading` | Progress indicator |

### Composition

| Node | Purpose |
|---|---|
| `conditional` | Visibility gate driven by a binding |
| `disclosure` | Collapsible region, trigger + content |
| `menu` | Popup/contextual menu |
| `slot` | Named injection point for other plugins |
| `tabs` | In-page tabbed content that does not change route |
| `pressable` | Makes any node an interactive region |

---

## Node identity

Every node has an optional `id` field that serves two purposes:

1. **Reconciliation identity** -- the renderer uses `id` to track nodes across
   reactive updates (equivalent to React's `key`). Must be unique among siblings
   in dynamic collections.

2. **Semantic/accessibility identity** -- becomes the DOM element `id` in the web
   renderer, the focus target label in TUI, and the target for `initialFocus`.

### Structural fallback

If `id` is absent, the normalizer derives a structural key from the node's
position in the tree (e.g. `screen.c0.c1`). This is stable within one load
cycle and safe for static structures.

**Dynamic collections** (list items, grid children with `continuation`) should
always provide explicit `id` values derived from the underlying data record.
The normalizer emits a dev warning when a grid with `continuation` has children
without explicit ids.

### Global uniqueness

`id` is validated as globally unique across the entire normalized tree. The
normalizer throws at normalization time if a duplicate is detected.

---

## Normalization metadata

After `normalizeScreen()`, every node carries additional fields used by renderers
for granular updates. These are prefixed with `_` to signal that they are
infrastructure, not plugin-authored data.

```
_key                - resolved identity (id or structural fallback)
_reactivity         - "static" | "reactive" | "conditional" for this node
_subtreeReactivity  - aggregate reactivity for this node and all descendants
_staticFields       - primitive fields already resolved (literals collapsed)
```

The `_subtreeReactivity` field allows renderers to skip entire static subtrees
without descending into them.

---

## Media references

`MediaNode` uses a typed reference instead of a raw URL:

```rust
pub enum MediaRef {
    Url { src: String },
    AtBlob { did: String, cid: String },  // AT Protocol blob
    Asset { name: String },               // host-managed static asset
    Placeholder { kind: Option<MediaKind>, label: Option<String> },
}
```

The renderer or host resolves the reference to actual bytes. Plugins never
hold raw URLs for authenticated or protocol-specific resources.

---

## `screen` is a root, not a child

`ScreenNode` does not appear in the `UiNode` union. It is the root of a plugin
view and cannot be nested inside another node. This simplifies renderer
assumptions and prevents invalid trees.

```rust
pub enum RootNode {
    Screen(ScreenNode),
}

// UiNode does NOT include ScreenNode
pub enum UiNode {
    Section(SectionNode),
    Stack(StackNode),
    // ... all non-screen nodes
}
```

---

## Serialization policy

The canonical AST is always valid JSON. The transport layer (`screenToJson`,
`screenFromJson`) adds a versioned envelope:

```json
{
  "type": "unode-screen",
  "v": "1.0.0",
  "ts": "2026-03-22T12:00:00Z",
  "screenKind": "catalog.browse",
  "screen": { ... }
}
```

The major version must match between sender and receiver. Minor and patch
versions are always compatible.

---

## What is not a core node kind

These are useful but belong in sugar or app-level layers:

- `card`, `banner_card` -- composition patterns, not semantic primitives
- route tabs/navigation tabs -- application chrome derived from manifest route
  groups, owned by the app shell
- `table` -- out of scope for now; TUI mapping is non-trivial
- `image_reader` -- app-specific media widget
- `page_header` -- shell chrome

If a feature can be expressed as composition of simpler nodes, prefer composition.
