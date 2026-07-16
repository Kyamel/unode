# Component Model Compatibility

Unode should support two official plugin authoring paths over time:

- Rust plugins through the current `unode-sdk`;
- TypeScript-family plugins through a future SDK.

The WebAssembly Component Model and WIT are the best long-term contract shape for
that goal, but the migration should be incremental. The current raw WASM ABI is
working and should remain supported while a component ABI matures beside it.

## Why Component Model

The current raw ABI is language-neutral in practice, but it is manually
specified:

```text
plugin_render(ptr, len) -> ptr
plugin_render_result_len() -> len
plugin_render_slot(ptr, len) -> ptr
plugin_render_slot_result_len() -> len
```

Every SDK must implement the same pointer/length memory convention, JSON
encoding/decoding, result-length exports, allocator exports, and `host_call`
import wrapper.

WIT gives Unode a typed interface contract that can be shared by Rust,
TypeScript/JavaScript, and future SDKs. The main benefit is not that it makes
JavaScript small or magically compiles TypeScript. The benefit is a stable,
tool-readable boundary for lifecycle calls, host capabilities, generated
bindings, compatibility checks, and documentation.

## Runtime weight

Component Model/WIT does not inherently require a heavy runtime inside every
plugin. The runtime weight depends on the source language and toolchain:

- Rust component: no embedded JavaScript engine.
- AssemblyScript-style component: no full JavaScript engine, but not full
  TypeScript/Node.js either.
- JavaScript/TypeScript via Javy, Extism JS PDK, or ComponentizeJS: likely
  embeds or links a JavaScript engine such as QuickJS or SpiderMonkey.

So WIT is a contract layer. Heavy runtime costs come from "real JavaScript inside
WASM", not from WIT itself.

## Migration strategy

### Phase 0: raw ABI remains stable

Keep today's ABI as the production path:

- `plugin_manifest`
- `plugin_load`
- `plugin_render`
- `plugin_render_slot`
- `plugin_dispatch`
- `*_result_len`
- `unode_alloc`
- `unode_dealloc`
- `host_call`

Rust plugins keep using `unode-sdk::export_plugin!()`.

### Phase 1: JSON-preserving WIT

Add an experimental WIT contract that preserves today's JSON envelopes:

```wit
package unode:plugin@0.2.0;

interface lifecycle {
    manifest: func() -> string;
    load: func(request-json: string) -> string;
    render: func(request-json: string) -> string;
    render-slot: func(request-json: string) -> string;
    dispatch: func(request-json: string) -> string;
}

interface host {
    call: func(envelope-json: string) -> string;
}

world unode-plugin {
    import host;
    export lifecycle;
}
```

The working draft lives in `wit/unode-plugin.wit`.

This phase deliberately avoids modeling every AST node in WIT. The canonical
JSON protocol remains the source of truth. Component hosts only gain a cleaner
way to call lifecycle functions and host capabilities.

Benefits:

- both Rust and TypeScript SDKs can target the same lifecycle shape;
- the host can support raw modules and components in parallel;
- the AST can keep evolving without rewriting WIT records for every node;
- golden tests can compare raw-ABI Rust plugins and component plugins by their
  JSON output.

Costs:

- WIT does not yet validate the structure of `ScreenNode` or `ActionRef`;
- SDKs still need JSON schema/types generated from Rust or maintained alongside
  Rust;
- host packages need component-loading paths in addition to raw module loading.

### Phase 2: typed WIT for stable envelopes

Once the lifecycle has proven itself, type the stable outer envelopes first:

- `resolved-route`;
- `plugin-manifest`;
- `permission-request`;
- `dispatch-outcome`;
- `host-call-envelope`.

Keep `screen-json`, `data-json`, `params-json`, and arbitrary metadata as JSON
strings until those shapes are stable enough to justify WIT records.

This gives better generated bindings while avoiding premature type churn in the
semantic UI tree.

### Phase 3: typed WIT for AST subsets

Only after the AST is more stable should Unode consider WIT definitions for
common node subsets such as `text`, `action`, `section`, `stack`, `list`, and
`status`.

Do this for SDK ergonomics and compatibility checks, not because the renderer
needs it. The host can continue normalizing JSON into `CanonicalScreen` and
lowering to IR.

## Host compatibility shape

Hosts should eventually accept both plugin formats:

```text
plugin.wasm raw module
  -> current pointer/length ABI
  -> RawPluginInstance

plugin.wasm component
  -> WIT lifecycle interface
  -> ComponentPluginInstance

RawPluginInstance and ComponentPluginInstance
  -> common PluginInstance trait
  -> same host runtime, permissions, state, normalization, IR, renderer
```

The shared host abstraction should expose:

- `manifest() -> PluginManifestEnvelope`;
- `load(request) -> JsonValue`;
- `render(request) -> ScreenNode`;
- `render_slot(request) -> PluginRenderSlotResponse`;
- `dispatch(request) -> PluginDispatchResponse`;
- host-call/capability dispatch through the existing permission model.

Everything after `render()` should stay unchanged:

```text
ScreenNode JSON -> normalize -> resolve slots -> CanonicalScreen -> track -> IrScreen -> renderer
```

## SDK guidance

Rust SDK:

- keep the current raw ABI macro;
- later add a component export path behind a new feature or package;
- keep producing the same JSON shapes.

TypeScript SDK:

- start with generated TypeScript types for the JSON protocol;
- provide builders that mirror the Rust DSL;
- target the JSON-preserving WIT lifecycle first;
- choose one toolchain profile explicitly:
  - AssemblyScript for small TypeScript-like WASM;
  - JS-engine-backed WASM for fuller JavaScript/npm compatibility.

The SDKs do not need identical internals. They need identical protocol output.

## Compatibility rule

The official compatibility contract should be:

1. A plugin is valid if it can produce the same lifecycle JSON envelopes.
2. The host owns validation, normalization, permission checks, state, and IR
   lowering.
3. Renderers never depend on whether a plugin came from Rust, TypeScript, raw
   WASM, or Component Model.
4. Raw ABI and Component ABI can coexist until the component path is proven in
   both Web and TUI hosts.
