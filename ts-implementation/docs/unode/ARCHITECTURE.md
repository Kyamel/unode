# unode Architecture

## Vision

`unode` should be a generic plugin UI architecture that supports:

- Web renderers
- TUI renderers
- app-domain agnostic core contracts
- a serializable semantic AST
- mandatory immutability at the AST boundary
- local SPA-like reactivity
- full route-driven reloads

The plugin author writes declarative UI once. Each platform renderer decides how to display, theme, and safely execute it.

## Core principles

- Intent over presentation
  Plugins describe what something is, not how many pixels it has.
- Small canonical AST
  The core node set should stay minimal and portable.
- Sugar over AST inflation
  Rich authoring APIs should usually compile down to simpler core nodes.
- Immutable output
  Plugin AST output should be read-only and frozen so renderers and plugins share a predictable contract.
- Renderer ownership
  Renderers own theme, style, spacing interpretation, focus, keyboard behavior, and sandboxing.
- Domain isolation
  `unode` knows nothing about mangas, posts, users, or any app-specific entities.
- Serializable state boundaries
  The AST and action payloads must stay JSON-safe.
- Explicit reactivity modes
  Local state updates should not imply full rerender. Route changes may.

## Layer model

### 1. unode core

The core owns:

- canonical AST types
- expression model for reactive bindings and route params
- route/load/render contracts
- generic built-in capabilities such as state, storage, events, and navigation
- core i18n contracts and helpers
- core registries such as routes, actions, commands, navigation, and providers
- generic permission concepts for those built-ins
- ergonomic DSL and helpers

The core does not own:

- CSS
- terminal drawing
- app domain APIs
- host shell layout
- app navigation chrome

### 2. app bridge

Each app defines a typed bridge on top of `unode`.

The bridge may expose:

- domain APIs such as `catalog`, `library`, or `reader`
- domain permission strings
- app-level sugar helpers built on the core AST

The bridge is where app-specific capability contracts live. The core stays generic.

### 3. renderer

Each platform implements a renderer for the same AST:

- Web renderer
- TUI renderer
- potentially desktop/native renderers later

The renderer is responsible for:

- output
- focus and keyboard behavior
- styling/theme decisions
- sandbox enforcement
- permission checks for built-in core capabilities
- patching or redrawing strategy

### 4. plugins

Plugins import:

- `unode`
- the app bridge package

Plugins should not import:

- the host app internals
- Web-specific UI libraries
- TUI-specific libraries

## Lifecycle

The intended lifecycle is:

1. Host installs or loads a plugin module.
2. Plugin declares routes, actions, commands, navigation items, providers, and any slot contributions.
3. Renderer resolves the current route.
4. Renderer creates a fresh per-screen state store.
5. `load(ctx)` fetches serializable data.
6. `render(data, ctx)` returns a semantic AST.
7. Renderer mounts the AST.
8. Local state updates patch affected nodes without re-running `render()`.
9. Route changes or explicit refresh/invalidation trigger a new load/render cycle.
10. Unmount tears down subscriptions and ephemeral state.

## Two update modes

### Local SPA-like reactivity

Use this when:

- the user is typing
- toggling disclosure state
- selecting filters within the current screen
- updating optimistic UI after an action

Expected behavior:

- state lives in a per-screen store
- the AST structure remains stable for that load cycle
- only subscribed nodes are re-evaluated or patched

### Route-driven rerender

Use this when:

- navigation changes path or query
- a deep link should be shareable
- the host needs a fresh `load()` cycle
- a mutation invalidates the current screen data

Expected behavior:

- route changes are platform-addressable
- the renderer may create a fresh screen store
- `load()` and `render()` run again

These two modes are complementary, not competing.

## Minimal core, rich composition

The core should prefer primitive semantic building blocks.

Good candidates for the canonical AST:

- layout primitives
- text/value/media primitives
- list/action/input primitives
- conditional/slot primitives
- feedback primitives

Things that should usually be sugar or app-level compounds instead of core node kinds:

- tab bars
- banner cards
- image readers
- tables
- app-specific detail cards
- page headers
- domain widgets

If something can be expressed as composition of simpler nodes, prefer composition.

## Immutability

Immutability is a strong fit for `unode` because it makes:

- reasoning about plugin output easier
- debugging renderer behavior easier
- caching and memoization simpler
- accidental mutation bugs rarer

The recommended model is:

- canonical AST is read-only by type
- authoring DSL returns frozen objects always
- renderers may build mutable internal mount structures, but they must never mutate plugin AST values
- plugin and renderer contracts should assume `Object.freeze` semantics, not just TypeScript readonly types

Immutability is not the goal by itself. It is a tool to keep the architecture predictable.

## Trust boundary

The renderer is the trust boundary.

That means the renderer, not the plugin, decides:

- which built-in capabilities are available
- which domains or origins may be accessed
- how storage is namespaced
- how events cross plugin boundaries
- how styles and themes are applied
- how focus and keyboard navigation work

The plugin only declares intent and symbolic actions.
