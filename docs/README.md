# unode

A renderer-agnostic semantic UI protocol for plugin-based applications.

## What it is

- A canonical serializable AST that describes plugin UI as intent, not presentation
- A runtime contract for routes, actions, local state, permissions, and plugin isolation
- A WASM-first execution model where plugins compile to `.wasm` and run on any host
- A foundation that app-specific bridges extend with domain APIs

## What it is not

- Not a renderer — renderers are platform-specific implementations
- Not a design system — themes live in the renderer
- Not a domain model — works, chapters, users belong in the app bridge
- Not tied to any language — the protocol is JSON, the reference implementation is Rust

## Architecture in one diagram

```
Plugin (Rust → .wasm)
  └── uses unode-plugin-sdk (Rust crate)
        ├── DSL builders  → produces CanonicalScreen JSON
        └── host functions ← ctx.api.*, ctx.state.*, ctx.http

unode (Rust, compiled to .wasm)
  ├── AST types + normalization
  ├── StateStore
  ├── ExprResolver (reactive binding tracking)
  ├── PermissionGuard
  └── Transport (JSON serialization)

App Bridge (Rust, compiled to .wasm)
  ├── MugenHostApi (catalog, library, reader)
  ├── Domain models (WorkSummary, ChapterSummary)
  └── Permission metadata per method

Web Host + Adapters
  ├── Loads plugin .wasm via WebAssembly.instantiate()
  ├── Loads unode_web_host.wasm via wasm-bindgen
  ├── Runs normalize, dependency tracking, and patch planning in Rust
  ├── Mounts IR through maintained framework adapters (React and Svelte today)
  └── Dispatches ActionRef back to the plugin WASM

TUI Renderer (Rust)
  ├── Loads plugin .wasm via Wasmtime
  ├── Implements host functions in Rust
  ├── Receives CanonicalScreen JSON
  ├── Ratatui + ratatui-image for rendering
  └── Dispatches ActionRef back to WASM
```

## Document map

| File | Contents |
|---|---|
| `ARCHITECTURE.md` | Layer model, design principles, lifecycle |
| `UI-FLOW.md` | Mermaid diagram of plugin UI data, host normalization, renderer IR, actions, and patches |
| `AST.md` | Node taxonomy, expression model, serialization |
| `RUNTIME.md` | Plugin contract, route lifecycle, state model |
| `REACTIVITY.md` | Granular reactive updates, ExprResolver, StateStore |
| `RENDERER.md` | Renderer responsibilities, Web and TUI parity |
| `WASM.md` | WASM execution model, sandboxing, host functions |
| `PLUGIN-WASM-ABI.md` | ABI names, request envelopes, and runtime validation path |
| `COMPONENT-MODEL.md` | Incremental Component Model/WIT compatibility plan for Rust and TypeScript SDKs |
| `TUI-RUNTIME-SESSIONS.md` | Why TUI uses activation-scoped guest sessions and how to optimize safely |
| `PERMISSIONS.md` | Two-layer permission system |
| `I18N.md` | Locale contract, plugin-owned translations |
| `HOST-BRIDGE.md` | App bridge shape, domain APIs |
| `IMPLEMENTATION.md` | Step-by-step guide for implementing unode in Rust |
| `RUNTIME-BOUNDARIES.md` | Division between `unode`, SDKs, packages, and renderers |
| `MIGRATION-STATUS.md` | Current snapshot of the Rust/WASM migration and remaining risks |
| `ROADMAP.md` | Forward migration path from the legacy TypeScript prototype to Rust |
