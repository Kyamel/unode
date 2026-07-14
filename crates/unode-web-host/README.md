# unode-web-host

`unode-web-host` is the Rust core session packaged for browser hosts.

It exists because a web application is partly JavaScript/framework code. The
crate compiles to `unode_web_host.wasm` through `wasm-bindgen` and gives that JS
world access to the same Rust normalization, state, reactivity, and patch
planning pipeline used by native hosts.

## Owns

- `WebSessionCore`, a native-testable Rust session;
- the `wasm-bindgen` `WebSession` JSON-in/JSON-out wrapper;
- mounting raw plugin screens into IR;
- initial binding resolution patches;
- applying host state writes and returning IR patch ops;
- flat state snapshots for the next plugin dispatch.

## Does Not Own

- plugin WASM instantiation;
- `host_call` dispatch from plugin memory;
- React, Svelte, Vue, or DOM rendering;
- permission policy beyond the core types it consumes.

JavaScript loads both `plugin.wasm` and `unode_web_host.wasm`, then wires them
through the web runtime bridge.
