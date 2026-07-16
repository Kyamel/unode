# web-svelte example

Private Svelte demo app for the Unode web runtime.

This example wires:

- `unode-core` for plugin loading, host sessions, state writes, and dispatch.
- `unode-svelte` for `<UnodeScreen />` and Svelte host-slot portals.
- `plugins/web-counter` as the bundled plugin WASM.

## Build & run

```sh
nix-shell --run ./examples/web-svelte/build.sh
cd examples/web-svelte
pnpm install
pnpm run dev
```

The demo mounts the bundled plugin at `/plugins/web-counter`; if opened at `/`,
the browser URL is replaced with that route before mounting.

## Verify

```sh
node examples/web-svelte/scripts/smoke.mjs
cd examples/web-svelte && pnpm run typecheck
```
