---
title: Roadmap
description: Where the Rust/WASM migration stands, near-term priorities, and the invariants that will not change.
---

Unode is moving from a legacy TypeScript prototype to a Rust/WASM architecture.
The Rust core is no longer just a schema port -- it now includes normalization,
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
`IrPatchOp`. React and Svelte are maintained adapters, not requirements -- a Vue
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
   breadth, computed bindings, and keyed collections -- see the known limits in
   [Reactivity](/concepts/reactivity/).
5. **Build the domain bridge pattern.** Flesh out app-specific bridge crates
   with domain models, method-level permission metadata, and host-call bindings;
   keep domain UI sugar out of the core.
6. **Continue the TUI runtime.** Connect the `unode-tui-runtime` session/loading
   helpers to a full Ratatui loop and verify the same plugin `.wasm` drives both
   web and terminal.

## Decided next steps

- **Component Model/WIT.** Keep the raw ABI stable while adding a parallel
  `unode:plugin@0.3.0` loading path. The WIT contract types envelopes and
  capabilities, while the recursive `ScreenNode` remains JSON.
- **Renderer.** Add a theme/token layer below recipe overrides and generate
  typed recipe contexts from the Rust node definitions where possible.
- **Protocol.** Add derived expressions, route `when`, localized manifest
  labels, an `Overlay` node, and content `Tabs` for in-page tabs.
- **Plugin surfaces.** Zero declared routes means no plugin-owned screen
  surface. Headless/service plugins act through slots, host-dispatched actions,
  and future capabilities.
- **Capabilities.** Cross-plugin communication stays host-brokered. Providers
  declare capabilities; consumers request them; the host routes calls with
  caller identity and timeouts.

## Systemic gaps

The remaining architecture gaps are tracked in
[Architectural Gaps](/reference/architectural-gaps/). The highest-priority
contracts are async host calls, resource limits, state namespacing, crash
isolation, persistent storage, lifecycle events, distribution/trust,
conformance, accessibility, and form validation.

## Invariants -- what will not change

- Plugins describe semantic UI, not DOM or terminal layout.
- Host state owns reactivity; plugin `render()` is not called for ordinary state
  writes.
- Permissions are enforced by the host boundary.
- The core stays domain-agnostic and renderer-agnostic.
- Web embedding stays framework-agnostic.
