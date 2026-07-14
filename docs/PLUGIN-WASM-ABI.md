# Plugin WASM ABI

The plugin contract is now split into three layers:

## `unode-sdk`

Owns the ABI names and request envelopes:

- `UNODE_PLUGIN_ABI_VERSION`
- required exports like `plugin_manifest`, `plugin_load`, `plugin_render`
- request payloads for load/render/dispatch
- host-call envelope
- allocator export macro via `export_allocators!()`

## Host runtimes

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
runtimes consume. On the web, it runs beside `unode_web_host.wasm`; the two
modules do not instantiate each other.

## Why this matters

This keeps:

- one plugin artifact per plugin
- one permission model
- one host-function ABI
- one validation path before instantiation

while still allowing Web and TUI to expose different host capabilities and
render the same semantic screen differently.
