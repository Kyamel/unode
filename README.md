# Unode

Unode is an experimental UI framework for plugin-first applications. It aims to
make UI simple, elegant, serializable, and environment-agnostic: the same plugin
should describe a screen once and let a web or TUI runtime render it in the
right native shape.

The core protocol is written in Rust. Plugins compile to WebAssembly so hosts can
sandbox third-party extensions and expose only explicit capabilities. The
maintained SDK is Rust, while the wire format stays JSON/IR so Unode can be
embedded in applications written with React, Svelte, Vue, or another framework.

## Monorepo Layout

| Path | Purpose |
|---|---|
| `crates/unode` | Renderer-agnostic AST, normalization, state, resolver, IR, patch planning, transport, and core runtime types. |
| `crates/unode-sdk` | Rust plugin authoring SDK: DSL reexports, manifests, permissions, ABI envelopes, allocators, and i18n helpers. |
| `crates/unode-runtime` | Shared host-runtime concepts such as registries, targets, and permission-guarded runtime wrappers. |
| `crates/unode-web-host` | Rust core pipeline compiled to WASM for browser hosts; normalizes screens, tracks reactivity, and emits IR patches. |
| `crates/unode-web-runtime` | Web runtime boundary helpers for plugin loading, host calls, memory, and bridge validation. |
| `crates/unode-tui-runtime` | TUI runtime boundary helpers using Wasmtime-oriented plugin sessions and host calls. |
| `crates/renderer` | TUI rendering work using Ratatui concepts. |
| `crates/mugens-domain` / `crates/mugens-sdk` | Example/domain bridge crates for app-specific models, permissions, and UI sugar. |
| `crates/mgn` | Example native Rust TUI application that wires the runtime, renderer, and Mugens bridge together. |
| `plugins/` | Rust WASM example plugins, including `web-counter`. |
| `runtimes/web-react` | Current React web adapter and JS bridge for `plugin.wasm + unode_web_host.wasm`. |
| `ts-implementation/` | Deprecated legacy TypeScript prototype kept only as migration reference. |
| `docs/` | Architecture, runtime, ABI, reactivity, permissions, and migration documentation. |

## Current Web Slice

The current browser proof of concept runs two WASM modules side by side:

```text
plugin.wasm              unode_web_host.wasm
render/dispatch -> JSON  normalize -> track -> plan patches
        \                 /
         JavaScript bridge -> keyed ScreenStore -> React adapter
```

React is the first adapter, not a hard requirement. The intended boundary is
framework-agnostic: web adapters consume IR and patch ops, while Rust owns the
core semantics.

## Why Web Has `unode-web-host`

The web stack has an extra crate because the browser host is partly JavaScript.
`unode-web-host` compiles the Rust core session to `unode_web_host.wasm` and
exposes it through `wasm-bindgen`, so React, Svelte, Vue, or another adapter can
ask Rust to normalize, track dependencies, and plan patches.

The TUI stack does not need a matching `unode-tui-host` crate because the TUI
host is already native Rust. `unode-tui-runtime` can call `crates/unode`
directly while it manages Wasmtime plugin instances, host calls, and terminal
session lifecycle.

`crates/mgn` is in `crates/` because it is a Rust workspace package/binary that
integrates Unode for one app. It is not the reusable TUI runtime itself; that
role belongs to `crates/unode-tui-runtime` plus `crates/renderer`.

## Useful Commands

```sh
cargo test --workspace
cargo test -p unode-web-host
cargo test --manifest-path plugins/web-counter/Cargo.toml
nix-shell --run ./runtimes/web-react/build.sh
```

See `docs/README.md` for the document map and `AGENTS.md` for contributor
orientation.
