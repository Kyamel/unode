---
title: Plugin WASM ABI
description: The raw plugin exports, the SDK macro that generates them, and how host packages validate the contract.
---

The plugin contract is split into three layers: the SDK that authors use, the
raw ABI it generates, and the host packages that validate and drive it.

## `unode-plugin-sdk` -- what you write

The SDK owns the ABI names and request envelopes: `UNODE_PLUGIN_ABI_VERSION`,
the required exports, the load/render/dispatch request payloads, the host-call
envelope, and the export macros.

Plugin authors write ordinary Rust functions and expose them with the macro:

```rust
fn manifest() -> PluginManifestEnvelope { /* ... */ }
fn load(request: &PluginLoadRequest) -> serde_json::Value { /* ... */ }
fn render(request: &PluginRenderRequest) -> ScreenNode { /* ... */ }
fn render_slot(request: &PluginRenderSlotRequest) -> PluginRenderSlotResponse { /* ... */ }
fn dispatch(request: &PluginDispatchRequest) -> PluginDispatchResponse { /* ... */ }

unode_plugin_sdk::export_plugin! {
    manifest: manifest,
    load: load,
    render: render,
    render_slot: render_slot,
    dispatch: dispatch,
}
```

The macro generates the raw C ABI exports. Plugins without slot UI can omit
`render_slot`; the macro still exports `plugin_render_slot` and returns
`{ "nodes": [] }`. A lower-level `export_allocators!()` macro is also
available.

## Raw exports -- what the macro generates

Every plugin module exports these symbols. You should not write them by hand for
ordinary plugins:

```text
unode_alloc(len) -> ptr
unode_dealloc(ptr, len)

plugin_abi_version() -> ptr

plugin_manifest() -> ptr
plugin_manifest_len() -> len

plugin_load(request_ptr, request_len) -> ptr
plugin_load_result_len() -> len

plugin_render(request_ptr, request_len) -> ptr
plugin_render_result_len() -> len

plugin_render_slot(request_ptr, request_len) -> ptr
plugin_render_slot_result_len() -> len

plugin_dispatch(request_ptr, request_len) -> ptr
plugin_dispatch_result_len() -> len
```

Data crosses as `(ptr, len)` pairs into WASM linear memory -- see
[WASM Sandbox](/concepts/wasm-sandbox/) for the memory protocol.

## Host packages -- what validates and drives it

**`unode-web-runtime`** validates ABI version compatibility, required export
presence, and the permission profile, then handles browser-side instantiation
via `WebAssembly.instantiate`.

**`unode-web-host`** runs the browser-side core pipeline after a plugin renders a
raw screen: normalize the `ScreenNode`, seed and own the `MemoryStateStore`,
track bindings with the resolver, lower to IR, and plan `IrPatchOp`s after state
writes. It is compiled with `wasm-bindgen` and consumed by JS adapters. It does
**not** instantiate plugins -- JavaScript instantiates both WASM modules and
wires the boundary.

**`unode-tui-runtime`** validates the same ABI contract and is where Wasmtime
integration lives for terminal hosts.

## Build flow

Plugins compile to `wasm32-unknown-unknown`:

```sh
cargo build --target wasm32-unknown-unknown --release
```

The resulting `.wasm` is a single artifact that **both** the web and TUI
packages consume. On the web it runs beside `unode_web_host.wasm`; the two
modules do not instantiate each other.

## Why one ABI

This keeps one plugin artifact per plugin, one permission model, one
host-function ABI, and one validation path before instantiation -- while still
letting web and TUI expose different host capabilities and render the same
semantic screen differently.

## Component Model path

The raw pointer/length ABI remains the production path. In parallel, the WIT
contract at `wit/unode-plugin.wit` is `unode:plugin@0.3.0`. It types manifests,
routes, permissions, lifecycle requests, dispatch outcomes, and core capability
imports while keeping the recursive `ScreenNode` tree as JSON. Raw modules and
components should both flow into the same host runtime, normalization, state,
IR, and renderer pipeline.
