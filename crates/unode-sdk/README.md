# unode-sdk

`unode-sdk` is the Rust authoring surface for plugin developers.

It reexports the core DSL, wraps manifest and permission construction, defines
plugin ABI request/response envelopes, provides allocator exports, and includes
plugin-owned i18n helpers.

## Owns

- plugin manifest builders;
- permission request builders;
- ABI constants and JSON envelopes for load/render/dispatch;
- `export_plugin!()` for generating raw WASM ABI exports;
- `export_allocators!()` for lower-level/manual ABI work;
- i18n catalog and translator helpers;
- a plugin-author-friendly prelude.

## Export Macro

Plugins should usually expose the ABI with `export_plugin!`:

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

The host still sees plain C ABI exports such as `plugin_manifest` and
`plugin_render`; plugin authors no longer have to write that boilerplate by hand.

## Does Not Own

- host runtime state;
- renderer adapters;
- domain-specific host APIs;
- Wasmtime or browser instantiation logic.

Plugins should depend on this crate plus any app-specific bridge SDK they need.
