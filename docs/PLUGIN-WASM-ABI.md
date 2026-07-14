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

The resulting `.wasm` file is the same artifact that both Web and TUI runtimes
consume.

## Why this matters

This keeps:

- one plugin artifact per plugin
- one permission model
- one host-function ABI
- one validation path before instantiation

while still allowing Web and TUI to expose different host capabilities and
render the same semantic screen differently.
