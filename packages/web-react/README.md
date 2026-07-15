# unode · React package

React adapter package for rendering Unode IR with app-defined components. The
shared browser runtime lives in `unode-core`; shared IR/store/patch primitives
live in `unode-renderer`.

```
 plugin.wasm (raw C ABI)            unode_web_host.wasm (wasm-bindgen)
   render / dispatch → JSON           normalize · track · plan_patch
        │                                        │
        └──────────────┬─────────────────────────┘
                       │ both instantiated by JS (no nesting)
        ┌──────────────┴───────────────────────────────────┐
        │  bridge.ts  ──▶  ScreenStore  ──▶  <UnodeScreen/>  │
        │  (dispatch loop)   (per-key patches)  (React)      │
        └────────────────────────────────────────────────────┘
```

## What each piece is

| File | Role |
|---|---|
| `crates/unode-web-host` (Rust) | The **same** core as the TUI, wrapped by wasm-bindgen. Owns normalize, the reactive dependency graph, and patch planning. Nothing here is re-implemented in TS. |
| `unode-core` | Shared web runtime implementation: native `WebAssembly.instantiate`, plugin registry, wasm-bindgen `WebSession` wrapper, `StateWriteSink`, and dispatch loop. |
| `unode-renderer` | Shared `ScreenStore`, IR types, prop normalization, and patch application. |
| `src/renderer.tsx` | React adapter and `createReactRenderer()` factory. Each keyed node subscribes to its own key via `useSyncExternalStore`; apps can map semantic nodes to their own components. |
| `plugins/web-counter` (Rust) | The reactive demo plugin: a line bound to `ui.countLabel` + increment / decrement / reset. |

## Custom React renderers

Apps can keep Unode's runtime, store, patches, and action dispatch while
replacing the visual node mapping:

```tsx
import { createReactRenderer } from "unode-react";
import { Button, Panel, Text } from "./design-system";

export const { UnodeScreen } = createReactRenderer({
  nodes: {
    text({ props }) {
      return <Text tone={props.tone}>{props.content}</Text>;
    },
    section({ children }) {
      return <Panel>{children}</Panel>;
    },
    action({ props, dispatch }) {
      return (
        <Button disabled={Boolean(props.disabled)} onClick={() => dispatch(props.action)}>
          {props.label}
        </Button>
      );
    },
  },
});
```

The renderer spec receives normalized props, rendered children, raw child nodes,
and `dispatch(ActionRef)`. It does not receive plugin WASM internals, permission
state, or host capabilities.

## Build & run

The repo's `shell.nix` provides the whole toolchain (wasm-ld, wasm-bindgen 0.2.108,
node, pnpm). From the repo root:

```sh
nix-shell --run ./packages/web-react/build.sh
cd packages/web-react
nix-shell --run 'pnpm install && pnpm run dev'   # open the printed localhost URL
```

`build.sh` uses the `wasm-bindgen` CLI directly (not wasm-pack) so nothing is
fetched from the network under nix. It emits `./pkg` (core bindings) and
`./demo/web_counter_plugin.wasm` (the plugin).

Click **Increment** — only the big number re-renders. `render()` is never called
again; the update is pure patch re-application, planned in Rust.

The demo does not implement a full route registry yet. It mounts the one bundled
plugin at `/plugins/web-counter`; if opened at `/`, `demo/main.tsx` replaces the
browser URL with that plugin route before mounting.

## Verifying without a browser

Three layers are checked without a DOM:

```sh
# 1. the engine, on the host toolchain (no wasm)
cargo test -p unode-web-host
cargo test --manifest-path plugins/web-counter/Cargo.toml

# 2. the real compiled wasm artifacts, end-to-end minus React
nix-shell --run 'node packages/web-react/scripts/smoke.mjs'

# 3. the TS glue types against the generated bindings
nix-shell --run 'cd packages/web-react && pnpm run typecheck'
```

`tests/reactive_slice.rs` asserts that writing `ui.count` produces exactly one
`SetProp` patch on the bound node (and none for an unrelated write).
`scripts/smoke.mjs` drives the actual `plugin.wasm` + `unode_web_host.wasm`
through the ABI and asserts increment yields the single scoped patch
`{o:"sp", k:"web-counter.value", f:"ct", v:{v:"Count: 1"}}`.

## Notes / next steps

- **State writes cross the sandbox boundary.** During `dispatch` the plugin calls
  the `host_call` import with `state.set`; the `StateWriteSink` buffers them and
  the bridge drains + applies them after dispatch. The plugin never returns UI
  state. The plugin wasm literally imports `unode.host_call` — a real host could
  refuse to provide it (or gate it by permission), and the plugin simply cannot
  write.
- **Svelte/Vue adapters** are just another `renderer.*` over the same `ScreenStore`
  + IR — the wasm host and bridge stay identical.
- **`ReplaceNode` / `ReplaceChildren`** patches are implemented in the store; the
  counter only exercises `SetProp`.
