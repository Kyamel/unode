---
title: Monorepo Layout
description: What each crate, runtime, and directory in the Unode monorepo is responsible for.
---

Unode is a Rust-first monorepo. Each package owns one boundary in the layer
model described in [Architecture](/concepts/architecture/).

## Crates

| Path                    | Purpose                                                                                          |
| ----------------------- | ------------------------------------------------------------------------------------------------ |
| `crates/unode`          | Core protocol: renderer-agnostic AST, normalization, state, resolver, IR, patch planning, transport. |
| `crates/unode-sdk`      | Rust plugin authoring SDK: DSL builders, manifests, permissions, ABI envelopes, allocators, i18n. |
| `crates/unode-runtime`  | Shared host-runtime concepts: registries, targets, permission-guarded runtime wrappers.           |
| `crates/unode-web-host` | Rust core pipeline compiled to WASM for browsers; normalizes screens, tracks reactivity, emits IR patches. |
| `crates/unode-web-runtime` | Web runtime boundary helpers: plugin loading, host calls, memory, bridge validation.           |
| `crates/unode-tui-runtime` | TUI runtime boundary helpers using Wasmtime-oriented plugin sessions and host calls.           |
| `crates/renderer`       | TUI rendering work using Ratatui.                                                                 |
| `crates/mgn`            | Example native Rust TUI application wiring runtime, renderer, and a domain bridge together.        |

## Runtimes

| Path                  | Purpose                                                                                         |
| --------------------- | ----------------------------------------------------------------------------------------------- |
| `runtimes/web-react`  | Maintained React web adapter and JS bridge for `plugin.wasm` + `unode_web_host.wasm`.            |
| `runtimes/web-svelte` | Maintained Svelte web adapter using the same plugin, host WASM, bridge shape, and keyed patch store. |

React and Svelte are maintained adapters over the same framework-agnostic
boundary — neither leaks into core semantics. A Vue or custom adapter consumes
the same IR and patch ops.

## Other directories

| Path                | Purpose                                                                    |
| ------------------- | -------------------------------------------------------------------------- |
| `plugins/`          | Rust WASM example plugins, including `web-counter` and `sanity-check`.      |
| `docs/`             | Source English project documentation (this site is generated alongside it). |
| `website/`          | This Starlight documentation site.                                          |
| `ts-implementation/`| Deprecated legacy TypeScript prototype, kept only as migration reference.   |

## Why the web has an extra host crate

The web stack has `unode-web-host` because the browser host is partly
JavaScript. That crate compiles the Rust core session to `unode_web_host.wasm`
and exposes it via `wasm-bindgen`, so React, Svelte, or another adapter can ask
Rust to normalize, track dependencies, and plan patches.

The TUI stack needs no matching `unode-tui-host` crate because the TUI host is
already native Rust: `unode-tui-runtime` calls `crates/unode` directly while it
manages Wasmtime instances, host calls, and terminal session lifecycle.

`crates/mgn` lives under `crates/` because it is a workspace binary that
integrates Unode for one app — it is **not** the reusable TUI runtime. That role
belongs to `crates/unode-tui-runtime` plus `crates/renderer`.

## Do not add work under `ts-implementation/`

`ts-implementation/` is deprecated reference material. Promote current web work
into `runtimes/web-react`, `runtimes/web-svelte`, or a future runtime package —
never back into the legacy prototype.
