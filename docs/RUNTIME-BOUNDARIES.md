# Runtime Boundaries

This repo now separates runtime concerns into four layers:

## `unode`

Owns renderer-agnostic protocol and semantics:

- AST
- canonical tree
- normalization
- IR + patch ops
- permission data structures
- screen chrome contracts such as `routeTabs`

## `unode-sdk`

Owns plugin authoring ergonomics:

- DSL reexports
- manifest builders
- generic permission builders
- generic i18n helpers
- screen chrome helpers for plugin authors

## `unode-runtime`

Owns shared host-runtime concepts that both Web and TUI need:

- route registry
- navigation registry
- command registry
- custom action registry
- shell query context
- runtime sandbox wrapper around `PermissionGuard`

This crate deliberately does not know about Svelte, Ratatui, or Mugens.

## Host-specific runtimes

### `unode-web-runtime`

Owns:

- Web/WASM instantiation
- JS host-function exposure
- i18next adapter
- browser navigation integration

### `unode-tui-runtime`

Owns:

- terminal host lifecycle
- Wasmtime/native runtime integration
- keyboard/command palette wiring
- terminal-specific navigation loop

## Renderers

Renderers sit below the runtime and should not own security decisions:

- a Svelte renderer reads screen/chrome data and applies patches
- a Ratatui renderer does the same in terminal form

Renderers consume:

- `CanonicalScreen` / IR
- `PatchOp`
- screen chrome metadata such as `routeTabs`

Renderers do not own:

- permission enforcement
- sandbox policy
- plugin activation
- registry state
