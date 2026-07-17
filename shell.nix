{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
  RUST_SRC_PATH = "${pkgs.rustPlatform.rustLibSrc}";

  packages = with pkgs; [
    cargo
    rustc
    rustfmt
    rustPlatform.rustLibSrc
    lld              # provides wasm-ld for the wasm32-unknown-unknown target
    wasmtime

    # Web examples (examples/web-react and examples/web-svelte)
    wasm-bindgen-cli # 0.2.108, must match `wasm-bindgen` crate pin
    wasm-tools       # component-model tooling: validate WIT, lift modules into components
    binaryen         # wasm-opt, for optional release size optimization
    nodejs_22
    pnpm
  ];

  shellHook = ''
    export CARGO_TARGET_WASM32_UNKNOWN_UNKNOWN_LINKER=wasm-ld
    echo "unode dev shell ready: cargo, rustc, wasm-ld, wasmtime, wasm-bindgen, wasm-tools, node, pnpm"
  '';
}
