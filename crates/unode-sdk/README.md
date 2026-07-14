# unode-sdk

`unode-sdk` is the Rust authoring surface for plugin developers.

It reexports the core DSL, wraps manifest and permission construction, defines
plugin ABI request/response envelopes, provides allocator exports, and includes
plugin-owned i18n helpers.

## Owns

- plugin manifest builders;
- permission request builders;
- ABI constants and JSON envelopes for load/render/dispatch;
- `export_allocators!()` for raw WASM plugins;
- i18n catalog and translator helpers;
- a plugin-author-friendly prelude.

## Does Not Own

- host runtime state;
- renderer adapters;
- domain-specific host APIs;
- Wasmtime or browser instantiation logic.

Plugins should depend on this crate plus any app-specific bridge SDK they need.
