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
- keyed React and Svelte adapters over `IrScreen` and `IrPatchOp`.

React and Svelte are maintained adapters, not framework requirements. Vue or a
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

- Promote the React and Svelte slices from proofs-of-concept into reusable
  package shapes.
- Keep framework adapters thin: IR in, patch ops applied, user actions out.
- Add documentation for embedding in React/Svelte and for writing alternate
  adapters.
- Avoid reimplementing core semantics in TypeScript.

### 4. Refine Reactivity Granularity

The current reactivity model is intentionally closer to Solid-style
targeted updates than to classic virtual DOM diffing:

- plugin UI is rendered once into a serializable AST/IR;
- `expr::binding("path")` records a dependency from state path to node key;
- `state.set("path", value)` wakes only subscribers of that path;
- patch planning re-resolves affected nodes and lowers them to compact IR patch
  ops.

This is a strong baseline, but there are known granularity limits to track:

- **Path breadth:** bindings to broad objects such as `work` will wake on writes
  to nested paths such as `work.title`. Prefer narrow bindings where possible,
  and consider typed `StatePath` helpers to make intent clearer.
- **Node-level re-resolution:** patches currently target node fields, not
  arbitrary subexpressions inside a field. Composite text or richer computed
  props may re-resolve more than the exact changed fragment.
- **Explicit bindings only:** values computed inside plugin Rust and emitted as
  literals are opaque to the host. Host-side reactivity requires dependencies to
  remain visible in the AST as expressions/bindings.
- **Indexed list paths:** paths such as `items.0.title` are useful but fragile
  when insertion or reordering changes indices. Lists need a stronger keyed
  identity story, similar in spirit to React `key`, Svelte keyed `each`, or
  Solid list helpers.
- **Stable node IDs:** precise patches depend on stable node keys. Generated
  keys can work, but interactive/stateful/plugin-extension anchors should keep
  explicit IDs.

Known framework parallels:

- Solid gets very fine updates by tracking signal reads at computation time; it
  still benefits from splitting large objects into smaller signals.
- Svelte compiles assignments into direct updates; complex object mutation still
  needs careful state shape and reassignment discipline.
- Vue proxies can track nested properties, but object/array shape and identity
  still affect update precision.
- React usually re-renders component subtrees and relies on reconciliation and
  `key`s; Unode should avoid that full-tree diff path where the binding graph can
  produce direct patches.

Future work should keep the protocol serializable while improving authoring
ergonomics around paths, computed bindings, and keyed collections.

### 5. Build The Domain Bridge Pattern

- Flesh out app-specific bridge crates such as `mugens-domain` and `mugens-sdk`.
- Add domain models, method-level permission metadata, and host-call bindings.
- Keep domain UI sugar out of `crates/unode`.
- Document plugin anchors and shell slots as app-owned extension points.

### 6. Continue The TUI Runtime

- Connect `unode-tui-runtime` session/loading helpers to a full Ratatui loop.
- Render the same IR/canonical semantics in terminal form.
- Share permission and state behavior with the web host.
- Verify that the same plugin `.wasm` can drive both environments.

## Legacy TypeScript Role

`ts-implementation/` is deprecated reference and migration material. The current
web React and Svelte runtime slices live in `runtimes/web-react` and
`runtimes/web-svelte`. The old same-process TypeScript plugin runtime should not
be treated as the target security model because it does not provide the WASM
sandbox boundary Unode needs.

## What Should Not Change

- Plugins describe semantic UI, not DOM or terminal layout.
- Host state owns reactivity; plugin render is not called for ordinary state
  writes.
- Permissions are enforced by the host boundary.
- The core remains domain-agnostic and renderer-agnostic.
- Web embedding remains framework-agnostic.
