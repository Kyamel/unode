# Plugin WASM ABI

The plugin contract is now split into three layers:

## `unode-sdk`

Owns the ABI names and request envelopes:

- `UNODE_PLUGIN_ABI_VERSION`
- required exports like `plugin_manifest`, `plugin_load`, `plugin_render`
- request payloads for load/render/dispatch
- host-call envelope
- full plugin export macro via `export_plugin!()`
- lower-level allocator export macro via `export_allocators!()`

Plugin authors should normally write regular Rust functions and expose them with
the SDK macro:

```rust
fn manifest() -> PluginManifestEnvelope { /* ... */ }
fn load(request: &PluginLoadRequest) -> serde_json::Value { /* ... */ }
fn render(request: &PluginRenderRequest) -> ScreenNode { /* ... */ }
fn dispatch(request: &PluginDispatchRequest) -> PluginDispatchResponse { /* ... */ }

unode_sdk::export_plugin! {
    manifest: manifest,
    load: load,
    render: render,
    dispatch: dispatch,
}
```

The macro generates the raw C ABI exports. The host contract does not change.

## Host packages

### `unode-web-runtime`

Validates:

- ABI version compatibility
- required export presence
- permission profile associated with the plugin instance

and then is responsible for browser-side instantiation through
`WebAssembly.instantiate`.

### `unode-web-host`

Runs the browser-side core pipeline after a plugin has rendered a raw screen:

- normalize the `ScreenNode`
- seed and own the `MemoryStateStore`
- track bindings with `DefaultExprResolver`
- lower the canonical screen to IR
- plan `IrPatchOp`s after state writes

This crate is compiled with `wasm-bindgen` and consumed by JavaScript adapters.
It does not instantiate plugins itself; JS instantiates both WASM modules and
wires the boundary.

### `unode-tui-runtime`

Validates the same ABI contract, but is the place where Wasmtime integration
will live for terminal hosts.

## Build flow

Plugins should compile to:

```text
wasm32-unknown-unknown
```

Recommended command:

```bash
cargo build --target wasm32-unknown-unknown --release
```

The resulting plugin `.wasm` file is the same artifact that both Web and TUI
packages consume. On the web, it runs beside `unode_web_host.wasm`; the two
modules do not instantiate each other.

## Why this matters

This keeps:

- one plugin artifact per plugin
- one permission model
- one host-function ABI
- one validation path before instantiation

while still allowing Web and TUI to expose different host capabilities and
render the same semantic screen differently.

## Future: TypeScript-authored plugins

The current ABI is already mostly language-neutral. Hosts look for named WASM
exports, pass JSON request envelopes through guest linear memory, receive JSON
responses, and expose host capabilities through explicit imports. None of that
requires the plugin source language to be Rust.

What is Rust-specific today is the authoring surface:

- `unode-sdk` reexports the Rust AST and DSL builders;
- `export_plugin!()` generates the raw C ABI exports;
- helper functions such as `host::state_set()` wrap the `host_call` import;
- Rust tests can call plugin lifecycle functions directly before export.

Supporting TypeScript should therefore preserve the protocol and add a second
authoring SDK rather than redefining Unode semantics. The TypeScript SDK would
need to provide:

- generated or hand-maintained TypeScript types for `PluginManifestEnvelope`,
  lifecycle requests/responses, `ScreenNode`, `ActionRef`, and AST nodes;
- builder helpers that mirror the Rust DSL closely enough for examples to look
  familiar;
- a small ABI shim that implements `plugin_manifest`, `plugin_load`,
  `plugin_render`, `plugin_dispatch`, result-length exports, allocation, JSON
  encoding/decoding, and `host_call` wrappers;
- golden tests proving a Rust-authored plugin and a TypeScript-authored plugin
  can produce equivalent JSON for the same screen.

There are three realistic implementation paths:

### AssemblyScript SDK

[AssemblyScript](https://www.assemblyscript.org/introduction.html) compiles a
TypeScript-like language directly to WebAssembly. This is the closest fit for
the current raw ABI because it can export functions and manage linear memory
like a normal WASM module.

Pros:

- likely smallest and closest to the existing `wasm32-unknown-unknown` ABI;
- no embedded JavaScript engine inside every plugin;
- host packages can keep the same validation model with minimal changes.

Costs:

- AssemblyScript is not full TypeScript or Node.js;
- common npm packages will often not work unless they are AssemblyScript-safe;
- we still need to write a parallel AssemblyScript/TypeScript DSL and ABI shim.

This is the best near-term experiment if the goal is "TypeScript-flavored plugin
authoring while preserving today's host model."

### JavaScript/TypeScript inside a WASM JS engine

Tools such as [Javy](https://github.com/bytecodealliance/javy) and the
[Extism JavaScript PDK](https://github.com/extism/js-pdk#readme) package
JavaScript into WASM by embedding a JavaScript engine such as QuickJS. TypeScript
can be bundled down to JavaScript before packaging.

Pros:

- much closer to real JavaScript/TypeScript authoring;
- many pure JavaScript npm packages can work after bundling;
- less need to teach plugin authors a TypeScript subset.

Costs:

- the resulting plugin carries a JS engine or depends on dynamic linking;
- startup, binary size, and memory usage are higher than Rust or AssemblyScript;
- the generated module's ABI will not automatically match Unode's raw C ABI;
- browser and TUI hosts may need extra WASI/Extism/Javy integration work.

This is attractive for developer adoption, but it is probably a separate runtime
profile unless we intentionally adapt Unode to that toolchain.

### WebAssembly Component Model / WIT

The long-term multi-language direction is to describe the plugin contract in
WIT and generate bindings for Rust, TypeScript/JavaScript, and other languages.
[ComponentizeJS](https://github.com/bytecodealliance/ComponentizeJS) is one
possible JavaScript component path, but it is still an experimental project.

Pros:

- cleanest language-neutral contract;
- generated bindings reduce hand-written ABI shims;
- aligns with future WASM ecosystem direction.

Costs:

- requires a larger ABI migration from today's raw pointer/length exports;
- JavaScript component tooling is still evolving;
- both Web and TUI hosts would need component-model loading support.

This should be tracked as a future ABI evolution, not the fastest way to prove
TypeScript plugin authoring.

### Recommendation

Do not try to reuse the Rust DSL implementation directly from TypeScript. Reuse
the Rust protocol, normalizer, renderer IR, patch planner, permission model, and
host-call semantics. Then create a TypeScript-family authoring SDK that emits
the same JSON.

The lowest-risk path is:

1. Treat the canonical JSON AST and plugin lifecycle envelopes as the source of
   truth.
2. Generate or maintain TypeScript types from those shapes.
3. Build a small `@unode/plugin-sdk` builder API that mirrors the Rust DSL.
4. Prototype an AssemblyScript plugin that satisfies today's raw ABI.
5. Only after that, evaluate Javy/Extism/Component Model for fuller JavaScript
   compatibility.

## Component Model compatibility

Unode should evolve toward Component Model/WIT compatibility without breaking the
raw ABI above. The first step is a JSON-preserving WIT contract that mirrors the
current lifecycle:

```wit
manifest: func() -> string
load: func(request-json: string) -> string
render: func(request-json: string) -> string
dispatch: func(request-json: string) -> string
```

This keeps `ScreenNode`, request envelopes, host-call envelopes, and dispatch
responses as the same JSON shapes the host already validates and normalizes.
Component Model support can then be added as a second loading path, not as an
immediate replacement for pointer/length raw modules.

See `COMPONENT-MODEL.md` and `../wit/unode-plugin.wit` for the staged plan.
