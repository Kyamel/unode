# tui-ratatui example

The TUI counterpart of `examples/web-react` and `examples/web-svelte`: a
minimal ratatui host that loads the counter plugin WASM and renders it through
the `unode-renderer` SDK.

Structure mirrors the web demos:

- `src/main.rs` — bootstrap: find the plugin wasm, terminal setup/teardown.
- `src/app.rs` — the App: renderer declaration, plugin session, and the
  render → focus → dispatch loop.
- `src/button.rs` — the host's native Button. Where the web demos back
  `action` nodes with a framework component via `hostSlot("Button")`, here the
  `Action` recipe is overridden with a ratatui painter.

## Run

```sh
# build the plugin once (inside nix-shell if wasm32 target lives there)
cargo build --manifest-path plugins/counter/Cargo.toml --target wasm32-unknown-unknown

cargo run -p tui-ratatui-example
```

Keys: arrow keys move focus, Enter dispatches the focused action into the
sandboxed plugin (its `state.set` host calls update the counter), `q` quits.
