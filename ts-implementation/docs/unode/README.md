# unode

`unode` is the target architecture for a generic plugin UI system that works on Web and TUI.

The core idea is simple:

- plugins declare intent, not presentation
- the AST is serializable and immutable
- local SPA-like reactivity is allowed
- full route-driven reloads are also allowed
- renderers are platform-specific and own theme, style, focus, and sandboxing
- app/domain features live in a host bridge, not inside the core

This folder now documents the architecture more explicitly than before. The older pseudo-TypeScript specs were useful to explore the space, but they looked more final than the implementation really is. The new docs are meant to be the source of truth for the direction of `unode vNext`.

## What unode is

- A renderer-agnostic semantic UI protocol.
- A small canonical AST plus ergonomic syntax sugar on top.
- A runtime contract for routes, actions, state, permissions, i18n, and plugin isolation.
- A foundation that app-specific bridges can extend with domain APIs.

## What unode is not

- Not a renderer implementation.
- Not a design system.
- Not a CSS abstraction.
- Not a domain model layer.
- Not a replacement for Svelte in the host app.

The host app can still use Svelte, React, terminal widgets, or anything else internally. `unode` exists to make plugin authoring simpler and more portable than hand-writing host-native views.

## Authoring goal

`unode` should be easier to author than raw host UI code for the plugin use case.

That means:

- most plugin authors should work through sugar builders, not raw AST objects
- common layouts should need less code than equivalent host-native views
- plugins should mostly think in terms of intent and data flow, not rendering details
- the sugar layer may be rich, as long as the emitted AST stays small and serializable

## Design stance

- The canonical AST should stay small.
- Richer UI should come from composition and sugar, not from endlessly adding new node kinds.
- A renderer may style the same AST very differently on Web and TUI.
- The renderer is the trust boundary.
- The core should remain app-domain agnostic.
- Immutability is a default architectural rule, not an optional optimization.

## Document map

- `ARCHITECTURE.md`
  General model, layers, goals, and lifecycle.
- `AUTHORING.md`
  Preferred plugin authoring API, including `definePlugin(...)`, `route(...).load(...).render(...)`, and `msg(...)`.
- `AST.md`
  Canonical AST constraints, expression model, node taxonomy, and sugar policy.
- `RUNTIME.md`
  Route lifecycle, actions, local state, reload semantics, and runtime contracts.
- `RENDERER.md`
  Renderer responsibilities, platform parity, keyboard/focus, and sandboxing.
- `I18N.md`
  Core i18n model based on plugin-provided JSON catalogs registered with the runtime.
- `HOST-BRIDGE.md`
  How apps extend `unode` with domain APIs, permissions, and host capabilities.
- `CURRENT-STATE.md`
  Comparison between the target architecture and what already exists in this repo.
- `IMPLEMENTATION-STATUS.md`
  Log of what parts of the target architecture have already been applied in code.
- `ROADMAP.md`
  Suggested migration path from the current runtime to a more robust cross-platform core.

## Short version

The target architecture has four layers:

1. `unode core`
   Defines the AST, runtime contracts, generic built-in capabilities, core registries, i18n, and authoring DSL.
2. `app bridge`
   Defines typed domain APIs and domain permissions.
3. `renderer`
   Implements Web or TUI output, sandboxing, styling, focus, navigation, and patching.
4. `plugins`
   Import `unode` plus the app bridge package, but do not import the host implementation directly.

## Relationship to the current repo

The current codebase already contains useful pieces:

- a serializable plugin UI model in `src/lib/unode`
- a working Web renderer in `src/lib/widgets/app-plugin-renderer-core`
- keyboard navigation in `src/lib/shared/keyboard`
- an app-specific bridge in `src/lib/plugins-bridge`

The main gap is not capability, but separation of concerns. Today the runtime is still fairly Web-first, some AST nodes are too presentation-shaped, and renderer/runtime boundaries are looser than the target model documented here.
