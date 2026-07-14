# mgn

`mgn` is the example terminal application that wires the Unode pieces together
for the Mugens domain.

It is an application crate, not a reusable framework crate. It depends on the
domain bridge, the TUI renderer, and `unode-tui-runtime` to prove that a plugin
screen can be loaded, rendered, navigated, and dispatched in a native terminal
host.

## Owns

- terminal application startup and event loop composition;
- app-level navigation and screen lifecycle decisions;
- integration between Mugens domain crates and Unode runtime crates;
- smoke/integration tests for repeated plugin navigation cycles.

## Does Not Own

- core AST, state, or patch semantics;
- plugin authoring APIs;
- reusable Wasmtime ABI helpers;
- reusable Ratatui node rendering primitives.

Those belong in `unode`, `unode-sdk`, `unode-tui-runtime`, and `tui-renderer`.
