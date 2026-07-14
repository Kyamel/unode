# unode · web runtime slice (React)

Minimal end-to-end proof that a unode plugin, compiled to WebAssembly, renders
and reacts in the browser through React — mirroring what `mgn` already does in
the terminal.

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
| `src/pluginHost.ts` | Native `WebAssembly.instantiate` of the plugin. Drives the ABI (`unode_alloc`, `plugin_render`, `plugin_dispatch`, `host_call`). The plugin's isolated linear memory **is** the sandbox. |
| `src/session.ts` | Typed wrapper over the wasm-bindgen `WebSession` (`mount`, `applyWrites`). |
| `src/store.ts` | `ScreenStore`: applies `IrPatchOp`s and wakes **only** the affected node key. |
| `src/renderer.tsx` | React adapter. Each keyed node subscribes to its own key via `useSyncExternalStore`; a `SetProp` patch re-renders exactly one component. |
| `src/bridge.ts` | The dispatch loop: click → plugin.dispatch → state writes → session.applyWrites → patches → store. |
| `plugins/web-counter` (Rust) | The reactive demo plugin: a line bound to `ui.countLabel` + increment / decrement / reset. |

## Build & run

The repo's `shell.nix` provides the whole toolchain (wasm-ld, wasm-bindgen 0.2.108,
node, pnpm). From the repo root:

```sh
nix-shell --run ./runtimes/web-react/build.sh
cd runtimes/web-react
nix-shell --run 'pnpm install && pnpm run dev'   # open the printed localhost URL
```

`build.sh` uses the `wasm-bindgen` CLI directly (not wasm-pack) so nothing is
fetched from the network under nix. It emits `./pkg` (core bindings) and
`./demo/web_counter_plugin.wasm` (the plugin).

Click **Increment** — only the big number re-renders. `render()` is never called
again; the update is pure patch re-application, planned in Rust.

## Verifying without a browser

Three layers are checked without a DOM:

```sh
# 1. the engine, on the host toolchain (no wasm)
cargo test -p unode-web-host
cargo test --manifest-path plugins/web-counter/Cargo.toml

# 2. the real compiled wasm artifacts, end-to-end minus React
nix-shell --run 'node runtimes/web-react/scripts/smoke.mjs'

# 3. the TS glue types against the generated bindings
nix-shell --run 'cd runtimes/web-react && pnpm run typecheck'
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
