# TUI Runtime Sessions

## Why this exists

During the first end-to-end Wasmtime integration, we found a stability bug in the TUI shell:

- open the sanity-check plugin
- switch to the inspect tab
- go back home through plugin dispatch
- reopen the plugin
- repeat a few times

Reusing the same long-lived guest instance across these activation cycles eventually caused a WASM trap during `plugin_render`.

The exact regression is preserved in the `mgn` app test:

- `app_survives_three_full_plugin_navigation_cycles`

That test exists to ensure we do not accidentally reintroduce the trap while optimizing session reuse.

## Current architecture

The TUI runtime now separates two layers:

1. `CompiledWasmtimePlugin`
   - owns the cached Wasmtime `Engine` + compiled `Module`
   - avoids recompiling the `.wasm` binary for every render or dispatch

2. `PluginSession`
   - owns a live guest instance plus its ABI bridge
   - is created on plugin activation through `plugin_load`
   - is reused while the current activated route stays mounted
   - is dropped when the shell navigates away from that plugin route

This gives us a safer boundary:

- compilation is cached
- instance state is not kept alive across unrelated activation cycles
- render and dispatch inside the same mounted screen reuse the same guest

In practice, this is a middle ground between two worse extremes:

- worst for stability: one global instance reused forever
- worst for performance: full Wasmtime compile + instantiate on every call

## Why session instantiation is still ephemeral

We are still treating plugin activation as an isolation boundary on purpose.

Today, when the user leaves the plugin screen and comes back later, the runtime creates a fresh guest session. We are doing that because:

- the lifecycle semantics are still being stabilized
- we already saw evidence that cross-activation instance reuse was unsafe
- correctness and crash containment matter more than aggressive reuse at this stage

So the current rule is:

- cache the compiled module
- keep the guest instance only for the currently mounted activation
- reset the session on route leave, plugin switch, or future trap recovery

## What improved already

Compared to the previous workaround, this is already better:

- the ephemeral policy moved out of `mgn` and into `unode-tui-runtime`
- `Module` compilation is cached
- `plugin_load` is now session startup work, not something the shell manually repeats before every call

That means we keep the robustness of reset-on-activation without paying the full compile cost each time.

## Next step

The next architecture step should focus on explicit session ownership rather than broader implicit reuse.

Recommended direction:

1. keep `CompiledWasmtimePlugin` cached per loaded plugin
2. promote `PluginSession` to a first-class mounted-screen/session object in the runtime
3. reset the session only on clear lifecycle boundaries:
   - route leave
   - plugin change
   - trap / fatal guest error
   - explicit plugin reload
4. later, if needed, experiment with pooling `Store`/`Instance` creation paths only after the lifecycle contract is proven safe

Only after that should we consider longer-lived reuse across activations.

## Performance and robustness goal

The target architecture is:

- compiled module cache for startup speed
- session-per-mounted-screen for correctness
- deterministic reset rules for robustness
- no renderer-specific assumptions in the runtime

That keeps the TUI runtime compatible with the same plugin ABI and lifecycle
model used by the maintained web packages, while still respecting the different
host implementation details.
