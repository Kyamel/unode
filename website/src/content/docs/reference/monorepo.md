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
| `crates/unode-plugin-sdk`      | Rust plugin authoring SDK: DSL builders, manifests, permissions, ABI envelopes, allocators, i18n. |
| `crates/unode-runtime`  | Shared host-runtime concepts: registries, targets, permission-guarded runtime wrappers.           |
| `crates/unode-web-host` | Rust core pipeline compiled to WASM for browsers; normalizes screens, tracks reactivity, emits IR patches. |
| `crates/unode-web-runtime` | Web runtime boundary helpers: plugin loading, host calls, memory, bridge validation.           |
| `crates/unode-tui-runtime` | TUI runtime boundary helpers using Wasmtime-oriented plugin sessions and host calls.           |
| `crates/unode-renderer`       | Backend-agnostic renderer SDK (recipes, measure/render, focus). |
| `crates/unode-ratatui-renderer` | Ratatui specialization: default recipes and screen painting.                                                                 |
| `crates/tui-playground`            | Example native Rust TUI application wiring runtime, manifest routes, and the ratatui renderer together.        |

## Runtimes

| Path                  | Purpose                                                                                         |
| --------------------- | ----------------------------------------------------------------------------------------------- |
| `packages/unode-web-core` | Shared browser runtime for plugin WASM loading, host sessions, state writes, and dispatch.       |
| `packages/unode-web-renderer` | Shared framework-free renderer, keyed store, recipes, DOM backend, and host-slot contract.   |
| `packages/unode-react` | React mount target and host-slot portal glue.                                                   |
| `packages/unode-vue` | Vue 3 mount package with host-slot portal glue. |
| `packages/unode-solid` | SolidJS mount package with host-slot portal glue. |
| `packages/unode-svelte` | Svelte mount target and host-slot portal glue.                                                 |
| `examples/web-react`  | Maintained React demo app for `plugin.wasm` + `unode_web_host.wasm`.                            |
| `examples/web-svelte` | Maintained Svelte demo app using the same plugin, host WASM, bridge shape, and keyed patch store. |
| `examples/web-vue` | Vue 3 demo app over the same runtime and counter plugin. |
| `examples/web-solid` | SolidJS demo app over the same runtime and counter plugin. |
| `examples/web-vanilla` | Framework-free demo (DOM renderer + Unode's own reactivity). |
| `examples/tui-ratatui` | Minimal ratatui host demo mirroring the web examples. |

React and Svelte are maintained adapters over the same framework-agnostic
boundary -- neither leaks into core semantics. A Vue or custom adapter consumes
the same IR and patch ops.

## Other directories

| Path                | Purpose                                                                    |
| ------------------- | -------------------------------------------------------------------------- |
| `plugins/`          | Rust WASM example plugins, including `counter` and `sanity-check`.      |
| `docs/`             | Engineering design docs; the website carries the public curated subset. |
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

`crates/tui-playground` lives under `crates/` because it is a workspace binary that
integrates Unode for one app -- it is **not** the reusable TUI runtime. That role
belongs to `crates/unode-tui-runtime` plus `crates/unode-ratatui-renderer`.

## Do not add work under `ts-implementation/`

`ts-implementation/` is deprecated reference material. Promote current web work
into `packages/` or runnable demos under `examples/` -- never back into the
legacy prototype.
