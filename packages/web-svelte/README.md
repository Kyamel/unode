# unode · Svelte package

Svelte adapter package for rendering Unode IR. The shared browser runtime lives
in `unode-core`; shared IR/store/patch primitives live in `unode-renderer`.

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
- The browser plugin loader, host session, registry, state-write sink, and
  dispatch loop are shared through `packages/unode-core`.
- Svelte consumes the shared `ScreenStore`, IR types, patch application, and prop
  normalization from `packages/unode-renderer`.
- `unode-web-host` owns normalization, state, reactivity tracking, and patch
  planning.
- A maintained web adapter can be small: IR in, patch ops applied, actions out.
- The same `plugins/web-counter` artifact works in both maintained web packages.

## Build & Run

From the repo root:

```sh
nix-shell --run ./packages/web-svelte/build.sh
cd packages/web-svelte
pnpm install
pnpm run dev
```

The demo mounts the bundled plugin at `/plugins/web-counter`; if opened at `/`,
the browser URL is replaced with that route before mounting.

## Verify

```sh
cargo test -p unode-web-host
cargo test --manifest-path plugins/web-counter/Cargo.toml
node packages/web-svelte/scripts/smoke.mjs
cd packages/web-svelte && pnpm run typecheck
```
