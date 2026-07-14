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
| `plugins/` | Rust WASM example plugins, including `web-counter`. |
| `ts-implementation/` | Legacy TypeScript implementation and current web React runtime slice used to prove browser integration. |
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

## Useful Commands

```sh
cargo test --workspace
cargo test -p unode-web-host
cargo test --manifest-path plugins/web-counter/Cargo.toml
nix-shell --run ./ts-implementation/web-react-runtime/build.sh
```

See `docs/README.md` for the document map and `AGENTS.md` for contributor
orientation.
