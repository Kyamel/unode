# unode Architecture

## Core principles

**Intent over presentation.** Plugins declare what something *is* — a heading, a
danger action, a cover image — never how many pixels it occupies or what color it
should be. The renderer translates intent into platform-specific output.

**Protocol first.** The canonical AST is JSON. Every component in the system —
plugin, host, renderer — communicates through this JSON protocol. No component
receives a live object from another; they receive serialized data.

**WASM as the execution boundary.** Plugins compile to `.wasm`. The host
instantiates the module, provides host functions, and receives the AST as a JSON
string. This enforces isolation without a separate process.

**Renderer as trust boundary.** The renderer owns sandboxing, theming, focus,
keyboard behavior, and permission enforcement. The plugin only declares intent.

**Domain isolation.** `unode` knows nothing about works, chapters, users, or any
app concept. Domain knowledge lives entirely in the app bridge.

---

## Layer model

### 1. unode core (Rust crate, compiled to .wasm)

Owns:
- Canonical AST types and normalization
- Expression model (`binding`, `param`, `literal`)
- StateStore contract and `MemoryStateStore` implementation
- `ExprResolver` with reactive dependency tracking
- Permission types and `PermissionGuard`
- JSON transport layer
- Plugin contract types (`PluginManifest`, `PluginRoute`, `ActionRef`)

Does not own:
- Any rendering output (DOM, terminal cells)
- Domain models or domain APIs
- App navigation chrome
- Locale resolution (only the locale contract)

### 2. App bridge (Rust crate, compiled to .wasm)

Owns:
- Typed domain API exposed to plugins (`MugenHostApi`)
- Domain permission strings (`catalog.read`, `library.write`)
- Permission metadata mapping each API method to its required permission
- Domain models (`WorkSummary`, `ChapterSummary`)
- Domain-specific UI sugar built from core AST primitives
- Locale provider: exposes the app's current locale to plugins

The bridge answers the question: "what can a plugin do in *this* app?"

### 3. Host runtime + renderer adapter (platform-specific)

Two implementations, one contract:

**Web host (Rust core compiled to WASM + JavaScript glue)**
- Instantiates plugin `.wasm` via `WebAssembly.instantiate()`
- Instantiates `unode_web_host.wasm` via `wasm-bindgen`
- Implements the `host_call` import and framework bridge in JavaScript
- Runs normalization, dependency tracking, and patch planning in Rust
- Emits IR and `IrPatchOp`s to a framework adapter
- Enforces permissions before providing host functions or host-call operations

**Web framework adapters (React and Svelte in the current slices)**
- Consumes the IR returned by `unode-web-host`
- Maintains a keyed `ScreenStore`
- Subscribes components by node key and applies patch ops
- Can be extended with Vue or another adapter without changing the core

**TUI renderer (Rust)**
- Instantiates plugin `.wasm` via Wasmtime
- Implements host functions as Rust closures
- Receives `CanonicalScreen` JSON, drives Ratatui + ratatui-image
- Manages reactive updates in Rust
- Enforces permissions in Rust before any host function executes

### 4. Plugins (Rust → .wasm)

Plugins import:
- `unode-sdk` — DSL builders, `PluginManifest`, `ActionRef`
- The app bridge crate — `MugenHostApi` type, domain sugar

Plugins must not import:
- Renderer internals
- DOM APIs
- Terminal APIs
- Direct networking (all network goes through `ctx.http`)

---

## Lifecycle

```
1. Host loads plugin.wasm
2. Host reads manifest from WASM export
3. Host checks PermissionProfile — reject if required permissions not granted
4. Host instantiates WASM module, injects host functions gated by PermissionGuard
5. Route match → host calls plugin.load(route, query) → receives JSON data
6. Host merges data into StateStore
7. Host calls plugin.render(data, state_json) → receives CanonicalScreen JSON
8. Host calls normalizeScreen(json) → fills defaults, computes _reactivity
9. Host calls trackReactiveBindings(screen, resolver, state, on_patch)
10. Host lowers the canonical screen to IR
11. Renderer adapter mounts the IR
12. User interaction → ActionRef dispatched → plugin action handler called (WASM)
13. Action handler calls ctx.state.set() → crosses WASM boundary → StateStore updates
14. StateStore writes are planned into patch ops → renderer patches only affected nodes
15. Navigation → teardown subscriptions, reset StateStore, repeat from step 5
```

Steps 1–4 happen once per plugin. Steps 5–11 happen on each navigation. Steps
12–14 happen on each user interaction. Step 15 restarts the cycle.

---

## Two update modes

These modes are complementary and serve different purposes.

### Local reactive updates (SPA-like)

When: user types, toggles a disclosure, clicks a favorite button.

Behavior:
- StateStore receives a write via `unode.setState` action or direct `ctx.state.set()`
- `ExprResolver.subscribersOf(path)` identifies affected node keys
- Only those nodes are re-evaluated and patched
- `render()` is never called again

### Route-driven reload

When: user navigates to a new URL, query string changes, explicit refresh.

Behavior:
- Current StateStore is torn down
- New StateStore created, seeded from `screen.initialState`
- Plugin `load()` called again
- Plugin `render()` called again with fresh data
- Full screen re-mounted

---

## Key design decisions

### Why WASM instead of JS modules

JS modules in the same process share memory and prototype chains. A plugin can
reach host objects through `window`, `globalThis`, or import side-effects. WASM
linear memory is isolated — a plugin cannot read host memory without an explicit
host function that grants access. Permissions are enforced before execution, not
by trusting the plugin to behave.

### Why Rust for unode

Rust's type system can express the AST precisely — discriminated enums, required
fields, no nulls unless explicitly `Option<T>`. Serde handles JSON serialization
without runtime overhead. The WASM output is small and starts fast. The same crate
compiles to `.wasm` for the plugin SDK and to native code for the TUI renderer.

### Why web adapters are framework-agnostic

Unode should be easy to embed in an existing React, Svelte, Vue, or custom web
application. The browser integration is therefore split in two: `unode-web-host`
owns the Rust core pipeline and JavaScript owns framework glue. The maintained
React and Svelte slices prove that keyed IR patches are not tied to one
framework. Additional adapters should consume the same IR and patch ops instead
of reimplementing normalization or reactivity.

### Why Ratatui for TUI instead of Notcurses

Ratatui is pure Rust with no FFI. It integrates cleanly with the Wasmtime-based
plugin runtime because everything is in one process with one async executor. Image
support via `ratatui-image` covers Kitty Protocol and Sixel. Notcurses would
require FFI from Rust to C, adding complexity at the boundary where the plugin
runtime already has enough moving parts.
