# unode-tui-runtime

`unode-tui-runtime` owns the terminal host's plugin WASM boundary.

There is no separate `unode-tui-host` crate because the TUI application already
runs as native Rust. It can call `unode` core functions directly. This crate
therefore focuses on what is unique to the terminal runtime: Wasmtime guest
sessions, plugin loading, host calls, and memory access.

## Owns

- Wasmtime-oriented guest/session types;
- plugin descriptor, loader, and cache helpers;
- raw ABI calls for manifest/load/render/dispatch;
- linear-memory JSON read/write utilities;
- terminal-host `host_call` dispatch abstractions.

## Does Not Own

- core normalization, state, resolver, or patch semantics;
- Ratatui widget rendering;
- app-level terminal event loops;
- domain-specific APIs.

The TUI stack should combine this crate with `unode`, `tui-renderer`, and the
application crate.
