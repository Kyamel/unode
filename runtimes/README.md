# Unode Runtimes

This directory contains maintained browser runtime slices.

Both web runtimes use the same Rust/WASM core boundary:

- `plugins/web-counter` compiles to a raw plugin WASM module.
- `crates/unode-web-host` compiles to `unode_web_host.wasm` through
  `wasm-bindgen`.
- JavaScript instantiates both WASM modules, implements `host_call`, drains
  plugin state writes, and applies Rust-planned `IrPatchOp`s to a keyed store.

## Maintained Web Adapters

| Path | Role |
|---|---|
| `web-react` | React adapter using `useSyncExternalStore` for per-node subscriptions. |
| `web-svelte` | Svelte adapter using `createSubscriber` for per-node subscriptions. |

The adapters are intentionally thin. Framework code should consume IR, apply
patch ops, and dispatch actions; normalization, binding tracking, state
snapshots, and patch planning stay in Rust.
