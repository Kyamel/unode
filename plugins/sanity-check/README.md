# Sanity Check Plugin

This plugin is a minimal Rust `cdylib` compiled to `wasm32-unknown-unknown` and
loaded at runtime by the MGN TUI shell through `unode-tui-runtime`.

## Host test

```bash
cargo test --manifest-path plugins/sanity-check/Cargo.toml --offline
```

## WASM build

The TUI shell looks for the artifact at one of these paths:

- `plugins/sanity-check/target/wasm32-unknown-unknown/debug/sanity_check_plugin.wasm`
- `plugins/sanity-check/target/wasm32-unknown-unknown/release/sanity_check_plugin.wasm`

Example build:

```bash
nix-shell
cargo build --manifest-path plugins/sanity-check/Cargo.toml --target wasm32-unknown-unknown
```

The repo ships a `shell.nix` plus `.cargo/config.toml`, so inside `nix-shell`
the wasm target automatically uses `wasm-ld`.

## TUI smoke run

```bash
cargo run -p mgn
```

Inside the shell, the `Sanity Check` navigation item opens the plugin route and
renders the `ScreenNode` returned from the WASM guest.

## Build Without `nix-shell`

```bash
wasm-ld --version
cargo build --manifest-path plugins/sanity-check/Cargo.toml --target wasm32-unknown-unknown
```

If `wasm-ld` is not on `PATH`, either enter `nix-shell` or point
`CARGO_TARGET_WASM32_UNKNOWN_UNKNOWN_LINKER` at the linker binary available on
your machine.
