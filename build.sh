#!/usr/bin/env bash
# Build the whole local Unode development graph and refresh generated artifacts.
#
# Recommended:
#   nix-shell --run ./build.sh
#
# This keeps the artifacts consumed by MGN, the web demos, and the website
# playground in sync with the Rust sources: workspace crates, plugin WASM
# builds, wasm-bindgen web-host bindings, and copied browser plugin binaries.
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

plugin_package_name() {
  local manifest="$1"
  awk -F '"' '/^name =/ { print $2; exit }' "$manifest"
}

plugin_wasm_name() {
  local manifest="$1"
  local package_name
  package_name="$(plugin_package_name "$manifest")"
  printf '%s.wasm\n' "${package_name//-/_}"
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

copy_plugin_wasm_to_playground() {
  local manifest="$1"
  local plugin_dir
  local wasm_name

  plugin_dir="$(cd "$(dirname "$manifest")" && pwd)"
  wasm_name="$(plugin_wasm_name "$manifest")"

  step "Copying $wasm_name into website playground"
  cp \
    "$plugin_dir/target/wasm32-unknown-unknown/release/$wasm_name" \
    "$ROOT/website/src/playground/wasm/$wasm_name"
}

copy_web_counter_demo_wasm() {
  local runtime_dir="$1"
  local runtime_name="$2"

  step "Copying counter plugin WASM into $runtime_name demo"
  cp \
    "$ROOT/plugins/counter/target/wasm32-unknown-unknown/release/web_counter_plugin.wasm" \
    "$runtime_dir/demo/web_counter_plugin.wasm"
}

cd "$ROOT"

require_cmd cargo
require_cmd wasm-bindgen

mkdir -p "$ROOT/website/src/playground/pkg" "$ROOT/website/src/playground/wasm"

step "Building Rust workspace (all targets)"
cargo build --workspace --all-targets

mapfile -t plugin_manifests < <(find "$ROOT/plugins" -mindepth 2 -maxdepth 2 -name Cargo.toml | sort)
for manifest in "${plugin_manifests[@]}"; do
  plugin_name="$(basename "$(dirname "$manifest")")"
  build_plugin_wasm "$manifest" "$plugin_name"
done

step "Building unode-web-host WASM (release)"
cargo build -p unode-web-host --target wasm32-unknown-unknown --release

generate_web_host_bindings "$ROOT/examples/web-react" "React"
generate_web_host_bindings "$ROOT/examples/web-svelte" "Svelte"
generate_web_host_bindings "$ROOT/website/src/playground" "website playground"

for manifest in "${plugin_manifests[@]}"; do
  copy_plugin_wasm_to_playground "$manifest"
done

copy_web_counter_demo_wasm "$ROOT/examples/web-react" "React"
copy_web_counter_demo_wasm "$ROOT/examples/web-svelte" "Svelte"

step "Done"
echo "Artifacts are in sync for MGN, web-react, web-svelte, and the website playground."
