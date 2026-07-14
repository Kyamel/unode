# unode-runtime

`unode-runtime` contains shared host-runtime concepts that are useful to both
web and terminal hosts.

It sits above `unode` core and below platform-specific runtime crates.

## Owns

- generic route, action, command, navigation, and provider registry concepts;
- runtime target metadata;
- permission-guarded hosted runtime wrappers;
- host/runtime coordination types that do not mention browser or terminal APIs.

## Does Not Own

- plugin WASM memory access;
- browser `WebAssembly.instantiate` glue;
- Wasmtime sessions;
- renderer component trees or terminal widgets;
- app-specific domain APIs.

If code needs DOM, Ratatui, Wasmtime, or `wasm-bindgen`, it belongs in a more
specific crate.
