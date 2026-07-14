{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
  packages = with pkgs; [
    cargo
    rustc
    rustfmt
    lld              # provides wasm-ld for the wasm32-unknown-unknown target
    wasmtime

    # Web runtime slice (runtimes/web-react)
    wasm-bindgen-cli # 0.2.108, must match `wasm-bindgen` crate pin
    binaryen         # wasm-opt, for optional release size optimization
    nodejs_22
    pnpm
  ];

  shellHook = ''
    export CARGO_TARGET_WASM32_UNKNOWN_UNKNOWN_LINKER=wasm-ld
    echo "unode dev shell ready: cargo, rustc, wasm-ld, wasmtime, wasm-bindgen, node, pnpm"
  '';
}
