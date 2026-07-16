# TypeScript To Rust/WASM Migration Status

Snapshot date: 2026-07-14.

This document describes the current repository state. It replaces the older
April snapshot that said the Rust runtime pieces were mostly placeholders.

## Goal

The migration is not a direct port of the legacy TypeScript implementation. The
goal is to make Unode a Rust-first, serializable, plugin-first UI protocol that
can run in Web and TUI environments through separate packages.

Unode should provide:

- a domain-agnostic core protocol;
- Rust SDK ergonomics for plugin authors;
- WebAssembly execution for sandboxing and non-Rust host compatibility;
- managed reactivity through host-owned state and patch planning;
- plugin anchors/slots where third parties can extend approved UI regions;
- a web bridge that embeds into React, Svelte, Vue, or another framework.

## Current Snapshot

- `crates/unode/src/core` now contains substantial Rust core behavior: AST, DSL,
  canonical normalization, state, resolver, reactivity tracking, patch planning,
  IR lowering, transport, permissions, runtime types, and tests.
- `crates/unode-plugin-sdk` exists and exposes plugin authoring pieces: manifest
  builders, permission helpers, i18n helpers, ABI envelopes, allocator exports,
  and core DSL reexports.
- `crates/unode-web-host` is the browser-side Rust core compiled through
  `wasm-bindgen`. It owns the web session pipeline: normalize, seed state, track
  dependencies, lower to IR, and plan patch ops.
- `packages/unode-web-core`, `packages/unode-web-renderer`, `packages/unode-react`, and
  `packages/unode-svelte` are the maintained browser packages.
- `examples/web-react` and `examples/web-svelte` are the maintained browser
  vertical slices. They instantiate both `plugin.wasm` and
  `unode_web_host.wasm`, wire host calls, store keyed IR, and render through
  framework adapters.
- `plugins/web-counter` is the end-to-end proof plugin for web reactivity.
- `crates/unode-web-runtime` and `crates/unode-tui-runtime` contain runtime
  boundary helpers for loading, memory, host calls, ABI bridges, and TUI plugin
  sessions.
- Domain bridge crates (`app-domain`, `app-sdk`) are still thin compared
  with the intended app-specific bridge.
- The old TypeScript implementation remains useful as reference material, but
  it should no longer be described as the active target architecture.

## What Changed Recently

The current tree has working React and Svelte web runtime slices:

- `crates/unode-web-host`
  - plain Rust `WebSessionCore` plus a `wasm-bindgen` `WebSession` wrapper;
  - `mount()` normalizes a raw plugin `ScreenNode`, seeds state, tracks
    dependencies, and returns `IrScreen`;
  - `initial_patches()` resolves symbolic bindings after mount;
  - `apply_writes()` applies state writes and returns targeted `IrPatchOp`s;
  - `state_snapshot()` feeds current host state back to plugin dispatch.
- `packages/unode-web-core`
  - JS plugin host using native `WebAssembly.instantiate`;
  - typed host-session wrapper over `unode-web-host`;
  - bridge that drains plugin host calls into state writes.
- `packages/unode-web-renderer`
  - keyed `ScreenStore`, IR helpers, recipe builder, DOM renderer, and host-slot
    contract.
- `packages/unode-react` and `packages/unode-svelte`
  - thin mount packages that connect the shared renderer to framework-native
    host-slot portals.
- `examples/web-react` and `examples/web-svelte`
  - validate that the same `plugins/web-counter` artifact is framework-neutral.
- `plugins/web-counter`
  - Rust WASM plugin that renders a reactive counter;
  - dispatch crosses the sandbox via `host_call("state.set", ...)`;
  - the host applies writes and patches only the bound node.

## Current Architecture Direction

The web runtime is split intentionally:

```text
plugin.wasm                  unode_web_host.wasm
manifest/load/render/dispatch  normalize/track/lower/plan
          \                    /
           JavaScript bridge and host-call dispatcher
                         |
                  framework adapter
              React and Svelte today, Vue/custom later
```

This keeps the semantics in Rust and the framework integration replaceable.
Adapters should consume IR and patch ops instead of porting normalization,
resolver, or patch planning into TypeScript.

## Remaining Work

- Lock the public boundary between raw plugin AST, canonical screen, and compact
  IR. Today the code has all three layers, but the long-term external contract
  should be documented more formally.
- Flesh out the app/domain bridge model with concrete Mugens domain APIs,
  method-level permission metadata, models, and domain UI sugar.
- Complete the web runtime package story so third-party apps can embed the web
  host and choose their adapter without depending on the demo layout.
- Continue the TUI runtime path from loader/session helpers into a full Ratatui
  renderer loop.
- Expand ABI validation and error reporting around plugin exports, host-call
  operations, and permission-denied cases.
- Add more golden tests for normalization, IR lowering, patch planning, and
  plugin ABI round-trips.
- Decide when the deprecated `ts-implementation/` reference can be deleted.

## Legacy TypeScript Risks To Avoid

The older TypeScript implementation demonstrated the product shape, but it also
mixed concerns that should stay separated in Rust:

- global renderer invalidation instead of path/key-scoped patches;
- screen resolution as component mount side effect;
- plugin loading through same-process ESM modules rather than a WASM sandbox;
- renderer code knowing too much about runtime, app shell, and domain APIs;
- permissions enforced as wrappers instead of absent capabilities.

The new Rust/WASM path addresses these by keeping state in the host, tracking
bindings centrally, planning patches in Rust, and making plugins communicate
through explicit ABI calls.

## Verification Commands

Useful checks for the current slice:

```sh
cargo test -p unode-web-host
cargo test --manifest-path plugins/web-counter/Cargo.toml
nix-shell --run 'node examples/web-react/scripts/smoke.mjs'
nix-shell --run 'cd examples/web-react && pnpm run typecheck'
nix-shell --run 'node examples/web-svelte/scripts/smoke.mjs'
nix-shell --run 'cd examples/web-svelte && pnpm run typecheck'
```
