# unode AST

## Hard constraints

The canonical `unode` AST should obey these rules:

- JSON-serializable
- immutable by type
- frozen at runtime
- semantic rather than presentation-driven
- renderer-agnostic
- app-domain agnostic

The AST should not contain:

- functions
- class instances
- symbols
- DOM nodes
- terminal escape sequences
- CSS classes
- raw theme tokens
- direct host object references

## Two layers: canonical AST and sugar

`unode` should have two authoring layers.

### 1. Canonical AST

This is the small portable protocol shared by every renderer.

It must stay:

- stable
- explicit
- easy to serialize
- easy to inspect
- easy to diff

### 2. Sugar DSL

This is the ergonomic authoring surface plugin authors actually use.

It may provide:

- helper builders
- common compositions
- shorthand defaults
- strongly typed wrappers
- app-level compounds built from core nodes

The sugar layer exists so writing plugins feels easier than writing raw Svelte, while the renderer still receives a simpler underlying AST.

## Expression model

The AST should support simple typed expressions for values that depend on runtime context.

Recommended expression families:

- `literal`
  Static scalar value.
- `binding`
  Reads from the current screen state store.
- `param`
  Reads from the resolved route params or query.

This is enough to enable:

- local reactive UI
- route-driven UI
- predictable renderer evaluation

The expression model should stay intentionally small.

The expression model should not embed translation lookup directly. Core i18n should resolve before or during AST authoring through explicit i18n helpers, leaving the emitted AST as plain semantic data plus runtime expressions for state and route.

## Proposed canonical node families

The core AST should bias toward primitives like these.

### Structure

- `screen`
- `section`
- `stack`
- `inline`
- `grid`
- `scroll`

### Content

- `text`
- `value`
- `icon`
- `badge`
- `divider`
- `media`

### Collections

- `list`
- `item`

### Actions

- `action`
- `actions`

### Input

- `input`
- `form`

### Feedback

- `status`
- `empty`
- `loading`

### Composition

- `conditional`
- `slot`

This is not meant to be the final literal API surface, but it captures the intended shape: small, composable, and portable.

## What should usually not be core node kinds

The current runtime already contains some richer nodes that are useful on Web, but they should not automatically define the future canonical AST.

Examples:

- `card`
- `banner_card`
- `tabs`
- `table`
- `image_reader`
- `page_header`
- app-specific media browsers

Why this matters:

- they are often presentation-shaped
- they are harder to map cleanly to TUI
- they tend to multiply renderer edge cases
- they can often be expressed as sugar over simpler nodes

If a feature is mostly a composition pattern, prefer a builder helper instead of a new base node kind.

## Screen root vs child nodes

`screen` is the root protocol object, not a normal child node.

That means:

- `ScreenNode` should only appear at the root
- `UiNode` should represent renderable child nodes only
- nested children should never be another `screen`

This keeps renderer assumptions simpler and prevents invalid trees.

## Keys, ids, and meta

The AST should distinguish between:

- `key`
  Reconciliation identity used by the renderer.
- `id`
  Semantic or accessibility identity exposed to the rendered output.
- `meta`
  Optional renderer hints that do not change the business meaning of the node.

Rules:

- `key` should be stable among siblings when order may change
- `id` may be used for focus restoration and accessibility
- at normalization time, every node must provide at least one of `key` or `id`
- the type keeps both fields optional because either one may satisfy the identity requirement
- `key` is currently validated for uniqueness among siblings in the same render group
- `id` is currently validated for uniqueness across the whole normalized tree
- no structural fallback to tree path or array index should exist
- `meta` must stay optional and ignorable
- core semantics must not depend on `meta`

Practical guidance:

- use `key` for internal renderer identity when the node does not need to be referenced externally
- use `id` when the node must be addressable by focus, accessibility, or DOM relationships
- use both only when a node genuinely has both an internal reconciliation identity and a public semantic identity

`screen` may also expose:

- `initialFocus`
  an optional node `id` hint for the renderer's initial focus target

## Semantic, not visual

The AST may expose semantic hints like:

- tone
- intent
- alignment
- emphasis
- density

But it should avoid telling the renderer exactly how to draw.

Bad examples for the core AST:

- `padding: 12`
- `fontSize: 18`
- `className: "rounded-xl bg-zinc-900"`
- `terminalColor: "brightMagenta"`

Good examples:

- `intent: "primary"`
- `tone: "warning"`
- `role: "title"`
- `columns: { base: 1, md: 3, xl: 5 }`
- `expandable: true`

## Serialization policy

The AST should be valid to:

- serialize for snapshots
- inspect in tests
- transport across boundaries if needed
- cache safely

The sugar layer may do richer work during authoring, but the emitted AST must return to a clean serializable form.

## Portable degradation rules

Some hints are allowed even if renderers degrade them differently.

- `media.expandable`
  Web may open fullscreen or a lightbox-style detail view; TUI may open a larger panel or ignore the hint.
- `grid.columns`
  TUI may ignore breakpoint-specific values and use `base` only.
- `grid.continuation`
  TUI may degrade this to a linear continuation flow even if Web renders a denser card grid.

## Immutability policy

Immutability is mandatory for the emitted AST.

That means:

- canonical nodes are readonly by type
- emitted AST objects are frozen at runtime
- nested objects and arrays are frozen too
- neither plugins nor renderers should mutate AST nodes after creation

This rule exists to simplify:

- renderer reasoning
- change detection
- caching
- plugin author expectations
- debugging of cross-platform behavior

## Practical consequence for the current codebase

The existing AST in `src/lib/unode/api/ui-types.ts` is a useful precursor, but the future canonical AST should likely become smaller and more semantic than the current Web-first node set.
