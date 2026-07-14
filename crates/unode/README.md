# unode

`unode` is the renderer-agnostic core protocol crate.

It defines the semantic UI AST, canonical normalization, host-owned state,
expression resolution, reactivity tracking, patch planning, compact IR,
permissions, transport envelopes, and small runtime contract types.

## Owns

- serializable UI data structures;
- normalization from plugin-authored `ScreenNode` to `CanonicalScreen`;
- `MemoryStateStore` and dot-path state access;
- `DefaultExprResolver` dependency tracking;
- patch planning for dirty node keys;
- IR lowering for renderers and web adapters;
- generic permission and route/manifest types.

## Does Not Own

- DOM, React, Svelte, terminal, or Ratatui rendering;
- plugin WASM instantiation;
- app/domain models such as works, chapters, or users;
- framework-specific bridge code.

Keep this crate portable across Web, TUI, and future hosts.
