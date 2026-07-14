# unode-web-runtime

`unode-web-runtime` contains browser-oriented WASM boundary helpers.

It is responsible for the plugin side of the web runtime boundary: validating
exports, reading and writing plugin linear memory, dispatching host calls, and
bridging raw plugin ABI calls in a browser-compatible shape.

## Owns

- plugin descriptor and loader types for web hosts;
- ABI bridge helpers for manifest/load/render/dispatch;
- linear-memory JSON read/write utilities;
- host-call dispatch abstractions;
- web-specific runtime wrapper types.

## Does Not Own

- the Rust core session exposed to JS (`unode-web-host` owns that);
- framework adapters such as React or Svelte;
- app/domain APIs;
- terminal or Wasmtime runtime behavior.

Think of this as web plugin-boundary infrastructure, while `unode-web-host` is
the browser-packaged core state/reactivity engine.
