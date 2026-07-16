# Unode Packages

This directory contains the TypeScript packages for the Unode web library.
Runnable web apps live under `examples/`.

The web examples use the same Rust/WASM core boundary:

- `plugins/web-counter` compiles to a raw plugin WASM module.
- `crates/unode-web-host` compiles to `unode_web_host.wasm` through
  `wasm-bindgen`.
- JavaScript instantiates both WASM modules, implements `host_call`, drains
  plugin state writes, and applies Rust-planned `IrPatchOp`s to a keyed store.

## Maintained Web Adapters

| Path | Role |
|---|---|
| `unode-core` | Shared browser runtime SDK primitives: plugin WASM instantiation, host-session wrapper, plugin registry, state-write sink, and dispatch loop. |
| `unode-renderer` | Shared TypeScript renderer SDK primitives: IR types, `ScreenStore`, patch application, node helpers, prop normalization, and fallback behavior. |
| `unode-react` | React mount target for the shared renderer, plus host-slot portal glue. |
| `unode-svelte` | Svelte mount target for the shared renderer, plus host-slot portal glue. |

The adapters are intentionally thin. Shared web runtime code loads plugins,
loads `unode_web_host.wasm`, dispatches actions, drains host calls, and applies
patches. Framework adapters should only subscribe to the keyed store, render IR,
and expose customization points for app design systems. Normalization, binding
tracking, state snapshots, and patch planning stay in Rust.
