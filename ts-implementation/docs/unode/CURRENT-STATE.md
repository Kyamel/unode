# Current State vs Target Architecture

This document compares the target `unode` architecture with what already exists in this repository.

## What is already strong

The current implementation already gives us important building blocks.

### 1. A real plugin runtime exists

Useful pieces already live in:

- `src/lib/unode/runtime`
- `src/lib/unode/registries`
- `src/lib/unode/api`

This means the architecture is not starting from zero.

### 2. The UI model is already declarative

The repository now authors directly against the core DSL and canonical AST.

The older `src/lib/unode/api/ui-types.ts` and `src/lib/unode/api/ui.ts` bridge was a useful precursor, but it has now been removed.

The older `src/lib/widgets/app-plugin-renderer` path has also been removed. The active renderer now lives entirely under
`src/lib/widgets/app-plugin-renderer-core`.

### 3. The Web renderer is real

The Svelte renderer under:

- `src/lib/widgets/app-plugin-renderer-core`
- `src/lib/widgets/app-plugin-shell`

already proves the plugin UI approach can work in the product.

### 4. Node identity is now explicit

The normalizer no longer falls back to structural tree paths for node identity.

Today:

- canonical `_key` comes only from explicit `key` or `id`
- renderer keyed blocks no longer fall back to array index
- missing identity fails fast during normalization

That is a healthier default for portability and debugging, because identity drift is now visible at authoring time instead of surfacing later as environment-specific UI bugs.

### 5. Keyboard navigation is ahead of the architecture docs

`src/lib/shared/keyboard` already contains serious work on:

- roving focus
- grid/list navigation
- nested navigation behavior
- editable target protection

This is valuable and should directly influence the generic renderer contract.

### 6. An app bridge already exists

`src/lib/plugins-bridge` is already acting like an app-specific extension layer.

That is exactly the right architectural direction.

## Main gaps

The main issue is not missing concepts. It is that some concerns are still mixed together.

### 1. The renderer is still Web-specific even though the state/runtime core is now portable

Many behaviors are implemented directly in the Svelte renderer and Web environment.

Examples:

- `window.innerWidth`
- `IntersectionObserver`
- DOM click synthesis
- Svelte component dispatch

This is acceptable for the Web renderer, but those behaviors still need TUI-specific implementations rather than being treated as core runtime concepts.

### 2. The AST is richer than the likely future core

Current nodes include several presentation-shaped or compound widgets such as:

- `card`
- `banner_card`
- `tabs`
- `image_reader`

These are useful, but they likely belong in sugar or app-level layers rather than the canonical core AST.

### 3. Reactivity is now locally state-driven, but route refresh is still an important second mode

Today, the repo has a portable `MemoryStateStore` driving local renderer bindings and interactions, alongside route/query-driven rerender for screen-level data reloads.

That is closer to the target architecture, but the two-mode contract still needs to be made more explicit in docs and APIs.

### 4. The load/render split is not yet explicit enough

The target architecture benefits from a clear separation:

- `load()` for data
- `render()` for immutable AST

The current route contract often fetches and builds the final screen in one place.

### 5. `ctx.events` is now a true shared bus, but permissions and contracts still need more refinement

The runtime now routes plugin events through the host bus, which matches the intended architecture much better.

What still needs work is the precision of permissioning and capability modeling around that bus.

### 6. Permissions are meaningful but still coarse

The current guard layer already protects host APIs, but mostly by broad API group.

The target model should move toward:

- explicit built-in capability permissions
- method-level metadata for domain permissions
- default-deny behavior where possible

### 7. Core and bridge boundaries can be cleaner

The repo already has the right folders, but the responsibilities are still not perfectly separated.

The desired split is:

- `unode core`
  generic
- `plugins bridge`
  app-specific
- `renderer`
  platform-specific

## Architectural implication

The right next move is not a rewrite. It is consolidation.

Recommended direction:

1. shrink and clarify the canonical core
2. keep useful current Web behavior
3. move compounds into sugar or bridge layers
4. introduce a real local state + expression contract
5. treat the current Svelte renderer as one renderer implementation

## Summary

The current codebase is already a good prototype of the architecture you want.

The main evolution is:

- from Web-first runtime to generic core plus renderer adapters
- from broad declarative nodes to a smaller canonical AST plus sugar
- from partial UI state to a clear two-mode reactivity model
- from implicit host coupling to an explicit app bridge
