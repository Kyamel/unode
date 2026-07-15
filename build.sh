#!/usr/bin/env bash
# Build the whole local Unode development graph and refresh generated artifacts.
#
# Recommended:
#   nix-shell --run ./build.sh
#
# This keeps the artifacts consumed by MGN and the web demos in sync with the
# Rust sources: workspace crates, plugin WASM builds, wasm-bindgen web-host
# bindings, and copied demo plugin binaries.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

require_cmd() {
  local cmd="$1"
  if ! command -v "$cmd" >/dev/null 2>&1; then
    echo "error: missing required command: $cmd" >&2
    echo "hint: run inside nix-shell: nix-shell --run ./build.sh" >&2
    exit 1
  fi
}

step() {
  echo
  echo "==> $*"
}

build_plugin_wasm() {
  local manifest="$1"
  local name="$2"

  step "Building $name plugin WASM (debug, for MGN artifact discovery)"
  cargo build \
    --manifest-path "$manifest" \
    --target wasm32-unknown-unknown

  step "Building $name plugin WASM (release, for web demos)"
  cargo build \
    --manifest-path "$manifest" \
    --target wasm32-unknown-unknown \
    --release
}

generate_web_host_bindings() {
  local runtime_dir="$1"
  local runtime_name="$2"

  step "Generating $runtime_name wasm-bindgen web-host bindings"
  wasm-bindgen \
    --target web \
    --out-dir "$runtime_dir/pkg" \
    --out-name unode_web_host \
    "$ROOT/target/wasm32-unknown-unknown/release/unode_web_host.wasm"
}

copy_web_counter_demo_wasm() {
  local runtime_dir="$1"
  local runtime_name="$2"

  step "Copying web-counter plugin WASM into $runtime_name demo"
  cp \
    "$ROOT/plugins/web-counter/target/wasm32-unknown-unknown/release/web_counter_plugin.wasm" \
    "$runtime_dir/demo/web_counter_plugin.wasm"
}

cd "$ROOT"

require_cmd cargo
require_cmd wasm-bindgen

step "Building Rust workspace (all targets)"
cargo build --workspace --all-targets

build_plugin_wasm "$ROOT/plugins/sanity-check/Cargo.toml" "sanity-check"
build_plugin_wasm "$ROOT/plugins/web-counter/Cargo.toml" "web-counter"

step "Building unode-web-host WASM (release)"
cargo build -p unode-web-host --target wasm32-unknown-unknown --release

generate_web_host_bindings "$ROOT/packages/web-react" "React"
generate_web_host_bindings "$ROOT/packages/web-svelte" "Svelte"

copy_web_counter_demo_wasm "$ROOT/packages/web-react" "React"
copy_web_counter_demo_wasm "$ROOT/packages/web-svelte" "Svelte"

step "Done"
echo "Artifacts are in sync for MGN, web-react, and web-svelte."
