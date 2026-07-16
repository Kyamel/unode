# tui-playground

`tui-playground` is the example terminal application that wires the Unode pieces together
for the Mugens domain.

It is an application crate, not the reusable TUI runtime. It lives under
`crates/` because it is a Rust workspace package/binary integrated with the
other Rust crates. A future cleanup could move application binaries to `apps/`,
but that would be a repo-organization change, not an architecture change.

It depends on the domain bridge, the TUI renderer, and `unode-tui-runtime` to
prove that a plugin screen can be loaded, rendered, navigated, and dispatched in
a native terminal host.

The app registers built-in shell routes plus runtime-loaded WASM plugins from
`plugins/`. Today it knows how to register both `sanity-check` and
`web-counter` when their `.wasm` artifacts have been built. Plugin registration,
artifact discovery, host-call dispatch, and the small host-owned state map used
for `state.set` live in `src/plugin_registry.rs`.

## Owns

- terminal application startup and event loop composition;
- app-level navigation and screen lifecycle decisions;
- integration between Mugens domain crates and Unode runtime crates;
- local plugin registration for the demo shell;
- smoke/integration tests for repeated plugin navigation cycles.

## Does Not Own

- core AST, state, or patch semantics;
- plugin authoring APIs;
- reusable Wasmtime ABI helpers;
- reusable Ratatui node rendering primitives.

Those belong in `unode`, `unode-plugin-sdk`, `unode-tui-runtime`, and `unode-ratatui-renderer`.
