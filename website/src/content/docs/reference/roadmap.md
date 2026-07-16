---
title: Roadmap
description: Where the Rust/WASM migration stands, near-term priorities, and the invariants that will not change.
---

Unode is moving from a legacy TypeScript prototype to a Rust/WASM architecture.
The Rust core is no longer just a schema port — it now includes normalization,
state, resolver tracking, patch planning, IR lowering, permissions, transport,
and a working browser host slice.

:::note[Alpha status]
Unode is in **alpha**. Protocol boundaries, the ABI, and runtime packaging are
still being stabilized. Expect changes between releases.
:::

## Current state

The current web proof runs a Rust plugin compiled to `wasm32-unknown-unknown`
alongside `unode_web_host.wasm`, with JavaScript glue wiring `host_call`
operations and keyed **React and Svelte** adapters over `IrScreen` and
`IrPatchOp`. React and Svelte are maintained adapters, not requirements — a Vue
or custom adapter consumes the same IR contract.

## Near-term priorities

1. **Stabilize core protocol boundaries.** Document the roles of raw
   `ScreenNode`, `CanonicalScreen`, and `IrScreen`; decide which layers are
   public ABI vs. host-internal; expand golden tests for normalization, IR
   lowering, and patch planning.
2. **Harden the plugin WASM ABI.** Validate required exports and ABI versions
   consistently, improve host-call error envelopes, and add tests for
   permission-denied and missing-host-function behavior.
3. **Package the web host model.** Promote the React and Svelte slices from
   proofs of concept into reusable packages; keep adapters thin (IR in, patch
   ops applied, actions out); document embedding and writing alternate adapters.
4. **Refine reactivity granularity.** Improve authoring ergonomics around path
   breadth, computed bindings, and keyed collections — see the known limits in
   [Reactivity](/concepts/reactivity/).
5. **Build the domain bridge pattern.** Flesh out app-specific bridge crates
   with domain models, method-level permission metadata, and host-call bindings;
   keep domain UI sugar out of the core.
6. **Continue the TUI runtime.** Connect the `unode-tui-runtime` session/loading
   helpers to a full Ratatui loop and verify the same plugin `.wasm` drives both
   web and terminal.

## Legacy TypeScript

`ts-implementation/` is deprecated reference and migration material. The current
web runtime packages live under `packages/`, with runnable React and Svelte demos
under `examples/`. The old same-process TypeScript runtime is **not** the target
security model — it does not provide the WASM sandbox boundary Unode needs.

## Invariants — what will not change

- Plugins describe semantic UI, not DOM or terminal layout.
- Host state owns reactivity; plugin `render()` is not called for ordinary state
  writes.
- Permissions are enforced by the host boundary.
- The core stays domain-agnostic and renderer-agnostic.
- Web embedding stays framework-agnostic.
