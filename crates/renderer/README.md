# tui-renderer

`tui-renderer` is the terminal renderer crate for Unode screens.

It translates Unode screen/chrome concepts into Ratatui-oriented terminal
rendering structures. It should stay focused on drawing and terminal interaction
semantics, not plugin sandboxing or domain APIs.

## Owns

- terminal rendering helpers and screen models;
- mapping Unode semantic nodes to Ratatui concepts;
- terminal-focused collection of route tabs, actions, and shell chrome data;
- renderer tests for output order and terminal behavior.

## Does Not Own

- plugin WASM instantiation;
- host-call permission enforcement;
- core AST normalization or patch planning;
- Mugens-specific domain models.

Runtime and sandbox work belongs in `unode-tui-runtime`; protocol work belongs in
`unode`.
