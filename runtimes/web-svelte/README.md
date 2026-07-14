# unode · web runtime slice (Svelte)

Minimal end-to-end proof that the same Unode plugin used by the React slice can
render and react through Svelte.

```text
 plugin.wasm (raw C ABI)            unode_web_host.wasm (wasm-bindgen)
   render / dispatch -> JSON          normalize · track · plan_patch
        │                                        │
        └──────────────┬─────────────────────────┘
                       │ both instantiated by JS (no nesting)
        ┌──────────────┴────────────────────────────────────┐
        │  bridge.ts  ->  ScreenStore  ->  <UnodeScreen />   │
        │  (dispatch loop)  (per-key patches)  (Svelte)      │
        └────────────────────────────────────────────────────┘
```

## What This Validates

- The plugin ABI is not tied to React.
- `unode-web-host` owns normalization, state, reactivity tracking, and patch
  planning.
- A maintained web adapter can be small: IR in, patch ops applied, actions out.
- The same `plugins/web-counter` artifact works in both maintained web runtimes.

## Build & Run

From the repo root:

```sh
nix-shell --run ./runtimes/web-svelte/build.sh
cd runtimes/web-svelte
pnpm install
pnpm run dev
```

The demo mounts the bundled plugin at `/plugins/web-counter`; if opened at `/`,
the browser URL is replaced with that route before mounting.

## Verify

```sh
cargo test -p unode-web-host
cargo test --manifest-path plugins/web-counter/Cargo.toml
node runtimes/web-svelte/scripts/smoke.mjs
cd runtimes/web-svelte && pnpm run typecheck
```
