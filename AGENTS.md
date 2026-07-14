# Agent Guide

This repository is a Rust-first monorepo for Unode, a plugin-first UI framework
that targets both web and terminal hosts. The project goal is to let plugins
describe serializable semantic UI once, run those plugins inside a WASM sandbox,
and render the result through environment-specific runtimes.

## Architecture At A Glance

- `crates/unode` is the core protocol. Keep it domain-agnostic and renderer-free.
- `crates/unode-sdk` is the Rust authoring surface for plugin developers.
- `crates/unode-runtime` contains shared host-runtime concepts.
- `crates/unode-web-host` is the browser-side Rust core compiled with
  `wasm-bindgen`; it owns normalization, state, reactivity tracking, and patch
  planning for web adapters.
- `crates/unode-web-runtime` and `crates/unode-tui-runtime` own host-specific
  WASM boundary concerns.
- `plugins/` contains example WASM plugins.
- `ts-implementation/` contains legacy TypeScript code plus the current React
  web-runtime slice that proves the web adapter model.
- `docs/` is the source of English project documentation.

## Design Constraints

- Keep Unode serializable. Cross boundaries with JSON/IR, not live objects.
- Keep core environment-agnostic. DOM, Svelte, React, terminal cells, and app
  domain models do not belong in `crates/unode`.
- Keep plugins sandboxed. Plugin capabilities must cross explicit host calls and
  permission checks.
- Keep the web bridge framework-agnostic. React is the current adapter; it should
  not leak into core semantics.
- Prefer stable explicit node IDs for interactive, stateful, or plugin-extension
  anchors.

## Documentation Expectations

Update `docs/` when changing protocol, runtime, ABI, permission, or reactivity
behavior. If an implementation detail in `ts-implementation/` is legacy, label it
as legacy instead of describing it as the target architecture.

## Verification

Run the narrowest useful checks for the files you touched. Common checks:

```sh
cargo fmt --all --check
cargo test --workspace
cargo test -p unode-web-host
cargo test --manifest-path plugins/web-counter/Cargo.toml
```
