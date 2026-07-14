{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
  packages = with pkgs; [
    cargo
    rustc
    lld
    wasmtime
  ];

  shellHook = ''
    export CARGO_TARGET_WASM32_UNKNOWN_UNKNOWN_LINKER=wasm-ld
    echo "unode dev shell ready: cargo, rustc, wasm-ld, wasmtime"
  '';
}
