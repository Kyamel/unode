# unode-plugin-sdk

`unode-plugin-sdk` is the Rust authoring surface for plugin developers.

It reexports the core DSL, wraps manifest and permission construction, defines
plugin ABI request/response envelopes, provides allocator exports, and includes
plugin-owned i18n helpers. It also wraps plugin-to-host calls so plugin authors
can express host intents without hand-writing WASM imports or pointer plumbing.

## Owns

- plugin manifest builders;
- permission request builders;
- ABI constants and JSON envelopes for load/render/dispatch;
- `export_plugin!()` for generating raw WASM ABI exports;
- `export_allocators!()` for lower-level/manual ABI work;
- host-call helpers such as `host::state_set()`, with native recording for
  plugin unit tests;
- i18n catalog and translator helpers;
- a plugin-author-friendly prelude.

## Export Macro

Plugins should usually expose the ABI with `export_plugin!`:

```rust
fn manifest() -> PluginManifestEnvelope { /* ... */ }
fn load(request: &PluginLoadRequest) -> serde_json::Value { /* ... */ }
fn render(request: &PluginRenderRequest) -> ScreenNode { /* ... */ }
fn dispatch(request: &PluginDispatchRequest) -> PluginDispatchResponse { /* ... */ }

unode_plugin_sdk::export_plugin! {
    manifest: manifest,
    load: load,
    render: render,
    dispatch: dispatch,
}
```

The host still sees plain C ABI exports such as `plugin_manifest` and
`plugin_render`; plugin authors no longer have to write that boilerplate by hand.

## Host Calls

Plugins should request host-owned state writes through the SDK:

```rust
use serde_json::json;
use unode_plugin_sdk::prelude as ui;

ui::host::state_set("ui.count", json!(1));
```

On WASM targets this uses the raw `unode.host_call` import. On native targets
the SDK records envelopes so dispatch code can be tested with ordinary
`cargo test`.

## Does Not Own

- host runtime state;
- renderer adapters;
- domain-specific host APIs;
- Wasmtime or browser instantiation logic.

Plugins should depend on this crate plus any app-specific bridge SDK they need.
