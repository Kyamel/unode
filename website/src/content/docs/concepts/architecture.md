---
title: Architecture
description: The layer model, lifecycle, and key design decisions behind Unode.
---

Unode is built as four cooperating layers, each with a strict boundary. Data
crosses those boundaries as serialized JSON/IR -- never as live objects.

## Layer model

### 1. Unode core

The renderer-agnostic Rust crate (`crates/unode`), compiled to `.wasm` for the
plugin SDK and to native code for the TUI. It **owns**:

- Canonical AST types and normalization
- The expression model (`binding`, `param`, `literal`)
- The `StateStore` contract and `MemoryStateStore`
- `ExprResolver` with reactive dependency tracking
- Permission types and `PermissionGuard`
- The JSON transport layer
- Plugin contract types (`PluginManifest`, `PluginRoute`, `ActionRef`)

It **does not own** any rendering output, domain models, app navigation chrome,
or locale resolution (only the locale contract).

### 2. App bridge

A per-app Rust crate compiled to `.wasm`. It answers "what can a plugin do in
*this* app?" and owns:

- The typed domain API exposed to plugins
- Domain permission strings (e.g. `catalog.read`, `library.write`)
- Permission metadata mapping each API method to its required permission
- Domain models and domain-specific UI sugar built from core AST primitives
- The locale provider that exposes the app's current locale to plugins

### 3. Host runtime + renderer adapter

Two implementations, one contract:

**Web host** -- Rust core compiled to WASM (`unode-web-host`) plus JavaScript
glue. It instantiates the plugin `.wasm` via `WebAssembly.instantiate()` and
`unode_web_host.wasm` via `wasm-bindgen`, runs normalization, dependency
tracking, and patch planning in Rust, and emits IR and patch ops to a framework
adapter. The trusted web runtime owns plugin loading, host-call routing,
permission enforcement, state, and plugin-fault policy. **React, Svelte, Vue,
Solid, and vanilla DOM** examples consume the same IR contract through thin
mount packages.

**TUI runtime + renderer** -- native Rust. `unode-tui-runtime` instantiates
plugin `.wasm` via Wasmtime, implements host functions as Rust closures,
validates permissions, and treats mounted guest sessions as disposable isolation
units. The Ratatui renderer receives normalized/lowered UI and drives Ratatui +
`ratatui-image` for rendering.

### 4. Plugins

Rust compiled to `.wasm`. Plugins import `unode-plugin-sdk` (DSL builders, manifest,
`ActionRef`) and the app bridge crate. They must **not** import renderer
internals, DOM APIs, terminal APIs, or direct networking -- all network access
goes through `ctx.http`.

## Lifecycle

```text
1.  Host loads plugin.wasm
2.  Host reads the manifest from a WASM export
3.  Host checks the PermissionProfile -- reject if required grants are missing
4.  Host instantiates the module, injecting host functions gated by PermissionGuard
5.  Route match -> plugin.load(route, query) -> data JSON
6.  Host merges the data into the StateStore
7.  Host calls plugin.render(data, state) -> CanonicalScreen JSON
8.  Host normalizes the screen (fills defaults, computes _reactivity)
9.  Host resolves slot contributions and preserves contributor origin
10. Host tracks reactive bindings
11. Host lowers the canonical screen to IR
12. Renderer adapter mounts the IR
13. User interaction -> ActionRef dispatched -> plugin action handler runs (WASM)
14. Handler requests state writes -> crosses the WASM boundary -> StateStore updates
15. Writes are planned into patch ops -> only affected nodes are patched
16. Navigation -> tear down subscriptions, reset StateStore, repeat from step 5
```

Steps 1–4 happen once per plugin activation/cache policy. Steps 5–12 happen on
each navigation. Steps 13–15 happen on each interaction. Step 16 restarts the
cycle.

## Two update modes

These are complementary, not alternatives.

**Local reactive updates (SPA-like)** -- a user types, toggles a disclosure, or
clicks a favorite. A write hits the StateStore, `ExprResolver` finds the
affected node keys, and only those nodes are re-evaluated and patched.
`render()` is never called again.

**Route-driven reload** -- the user navigates to a new URL, the query changes, or
a refresh is requested. The current StateStore is torn down, a fresh one is
seeded from `screen.initialState`, and `load()` + `render()` run again for a
full re-mount.

## Key design decisions

**Why WASM instead of JS modules.** JS modules share memory and prototype chains
in-process; a plugin could reach host objects through `globalThis` or import
side effects. WASM linear memory is isolated -- a plugin cannot read host memory
without an explicit host function. Permissions are enforced before execution,
not by trusting the plugin.

**Why Rust for the core.** Rust's type system expresses the AST precisely --
discriminated enums, required fields, no accidental nulls. Serde handles JSON
without runtime overhead, the WASM output is small and starts fast, and the same
crate compiles to `.wasm` for plugins and native for the TUI.

**Why framework-agnostic web adapters.** Unode should embed easily in an
existing React, Svelte, or Vue app. `unode-web-host` owns the Rust pipeline and
JavaScript owns framework glue. The React and Svelte slices prove that keyed IR
patches are not tied to one framework; new adapters consume the same IR and
patch ops rather than reimplementing normalization or reactivity.

**Why Ratatui for the TUI.** Ratatui is pure Rust with no FFI, so it integrates
cleanly with the Wasmtime-based plugin runtime -- one process, one async
executor. Image support via `ratatui-image` covers the Kitty protocol and Sixel.
