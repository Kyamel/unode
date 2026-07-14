# Unode Roadmap

## Current State

Unode is moving from a legacy TypeScript prototype to a Rust/WASM architecture.
The Rust core is no longer only a schema port: it now includes normalization,
state, resolver tracking, patch planning, IR lowering, permissions, transport,
and a working browser host slice.

The current web proof runs:

- a Rust plugin compiled to `wasm32-unknown-unknown`;
- `unode_web_host.wasm`, built from `crates/unode-web-host`;
- JavaScript glue that wires plugin `host_call` operations;
- a keyed React adapter over `IrScreen` and `IrPatchOp`.

React is the first adapter, not the framework requirement. Svelte, Vue, or a
custom adapter should consume the same IR contract.

## Near-Term Priorities

### 1. Stabilize Core Protocol Boundaries

- Document the roles of raw `ScreenNode`, `CanonicalScreen`, and `IrScreen`.
- Decide which layer is public ABI and which layers are host-internal.
- Expand golden tests for normalization, IR lowering, and patch planning.
- Keep the protocol fully serializable.

### 2. Harden The Plugin WASM ABI

- Validate required exports and ABI versions consistently.
- Improve host-call error envelopes.
- Add tests for permission-denied and missing-host-function behavior.
- Keep one plugin artifact usable by both Web and TUI runtimes.

### 3. Package The Web Host Model

- Promote the React slice from proof-of-concept into a reusable package shape.
- Keep framework adapters thin: IR in, patch ops applied, user actions out.
- Add documentation for embedding in React and for writing alternate adapters.
- Avoid reimplementing core semantics in TypeScript.

### 4. Build The Domain Bridge Pattern

- Flesh out app-specific bridge crates such as `mugens-domain` and `mugens-sdk`.
- Add domain models, method-level permission metadata, and host-call bindings.
- Keep domain UI sugar out of `crates/unode`.
- Document plugin anchors and shell slots as app-owned extension points.

### 5. Continue The TUI Runtime

- Connect `unode-tui-runtime` session/loading helpers to a full Ratatui loop.
- Render the same IR/canonical semantics in terminal form.
- Share permission and state behavior with the web host.
- Verify that the same plugin `.wasm` can drive both environments.

## Legacy TypeScript Role

`ts-implementation/` is now reference and migration material, plus the home of
the current web React runtime slice. The old same-process TypeScript plugin
runtime should not be treated as the target security model because it does not
provide the WASM sandbox boundary Unode needs.

## What Should Not Change

- Plugins describe semantic UI, not DOM or terminal layout.
- Host state owns reactivity; plugin render is not called for ordinary state
  writes.
- Permissions are enforced by the host boundary.
- The core remains domain-agnostic and renderer-agnostic.
- Web embedding remains framework-agnostic.
