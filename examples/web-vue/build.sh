#!/usr/bin/env bash
# Builds the two wasm artifacts the demo needs.
#
# Run inside the repo's nix shell (provides wasm-ld + wasm-bindgen):
#   nix-shell --run ./examples/web-vue/build.sh
#
# Uses the wasm-bindgen CLI directly rather than wasm-pack: on nix that avoids
# wasm-pack trying to download its own wasm-bindgen / wasm-opt from the network.
set -euo pipefail

HERE="$(cd "$(dirname "$0")" && pwd)"
ROOT="$(cd "$HERE/../.." && pwd)"

echo "==> Building unode-web-host (Rust core) for wasm32"
cargo build -p unode-web-host --target wasm32-unknown-unknown --release

echo "==> Generating wasm-bindgen JS bindings (target: web)"
wasm-bindgen \
  --target web \
  --out-dir "$HERE/pkg" \
  --out-name unode_web_host \
  "$ROOT/target/wasm32-unknown-unknown/release/unode_web_host.wasm"

echo "==> Building counter plugin (raw wasm C ABI)"
cargo build \
  --manifest-path "$ROOT/plugins/counter/Cargo.toml" \
  --target wasm32-unknown-unknown \
  --release

cp "$ROOT/plugins/counter/target/wasm32-unknown-unknown/release/web_counter_plugin.wasm" \
  "$HERE/demo/web_counter_plugin.wasm"

echo "==> Done. Now: pnpm install && pnpm run dev"
