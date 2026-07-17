---
title: Installation
description: Prerequisites and commands to build the Unode workspace, plugins, and web packages locally.
---

Unode is a Rust-first monorepo. This page gets a local build working end to end:
the workspace crates, the example WASM plugins, and the web runtime bindings.

## Prerequisites

You need a Rust toolchain with the WebAssembly target, plus `wasm-bindgen` for
the web host bindings and `pnpm` for the runtime demos.

- **Rust** (stable, edition 2024) with the `wasm32-unknown-unknown` target:

  ```sh
  rustup target add wasm32-unknown-unknown
  ```

- **wasm-bindgen CLI** -- generates the browser bindings for
  `unode_web_host.wasm`.
- **pnpm** -- installs and runs the React/Svelte web runtime demos.

:::tip[Nix users]
The repo ships a `shell.nix` that provides `cargo`, `wasm-bindgen`, and the rest
of the toolchain. Prefix any command with `nix-shell --run` to run it in that
environment, e.g. `nix-shell --run ./build.sh`.
:::

## Clone and build

```sh
git clone https://github.com/Kyamel/unode.git
cd unode

# Build every crate and all targets
cargo build --workspace --all-targets
```

## Build everything (recommended)

`build.sh` builds the whole local development graph and keeps the artifacts that
the demos consume in sync: workspace crates, plugin WASM (debug + release),
`wasm-bindgen` web-host bindings, and the copied demo plugin binaries.

```sh
nix-shell --run ./build.sh
```

This produces artifacts for the TUI playground TUI app and both web examples
(`examples/web-react` and `examples/web-svelte`).

## Build pieces individually

Compile a plugin to WebAssembly -- the same artifact both the web and TUI
packages consume:

```sh
cargo build \
  --manifest-path plugins/counter/Cargo.toml \
  --target wasm32-unknown-unknown --release
```

Build the web host core (`unode_web_host.wasm`) and generate its browser
bindings:

```sh
cargo build -p unode-web-host --target wasm32-unknown-unknown --release

wasm-bindgen --target web \
  --out-dir examples/web-react/pkg \
  --out-name unode_web_host \
  target/wasm32-unknown-unknown/release/unode_web_host.wasm
```

## Run the tests

```sh
cargo test --workspace
cargo test -p unode-web-host
cargo test --manifest-path plugins/counter/Cargo.toml
```

## Run the web demos

After `build.sh` has synced the artifacts:

```sh
nix-shell --run ./examples/web-react/build.sh
nix-shell --run ./examples/web-svelte/build.sh
```

Next: build a plugin from scratch in the
[Quickstart](/getting-started/quickstart/).
