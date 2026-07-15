# Unode UI Flow

This diagram shows the current intent-based UI flow. Plugins emit semantic UI
data. The host owns normalization, state, reactivity, and IR lowering. Renderers
own presentation. User events return to plugins as symbolic actions, not as IR.

```mermaid
flowchart TD
    subgraph Plugin["Plugin WASM"]
        DSL["SDK DSL builders"]
        Render["plugin_render(request)"]
        Dispatch["plugin_dispatch(ActionRef, state_snapshot)"]
        HostCalls["host_call('state.set', params)"]
    end

    subgraph Host["Trusted Host Runtime"]
        Loader["Plugin loader + ABI validation"]
        Permissions["Permission guard + capability imports"]
        Normalize["normalize ScreenNode JSON"]
        Canonical["CanonicalScreen"]
        State["MemoryStateStore"]
        Reactivity["track reactive bindings"]
        Lower["lower CanonicalScreen to IrScreen"]
        PatchPlan["plan IrPatchOp after state writes"]
    end

    subgraph Renderer["App Renderer"]
        Store["ScreenStore"]
        Adapter["React / Svelte / TUI / custom adapter"]
        Components["App design-system components"]
        Event["User interaction"]
    end

    DSL --> Render
    Loader --> Render
    Permissions --> Render
    Render -- "ScreenNode / semantic JSON" --> Normalize
    Normalize --> Canonical
    Canonical --> State
    Canonical --> Reactivity
    Reactivity --> Lower
    Lower -- "IrScreen" --> Store
    Store --> Adapter
    Adapter --> Components

    Event -- "ActionRef" --> Adapter
    Adapter -- "dispatch(ActionRef)" --> Dispatch
    Dispatch --> HostCalls
    HostCalls -- "capability-gated state writes" --> State
    State --> PatchPlan
    PatchPlan -- "IrPatchOp[]" --> Store
```

## Direction of data

| Direction | Payload | Owner |
|---|---|---|
| Plugin to host | `ScreenNode` / semantic JSON | Plugin authoring SDK |
| Host internal | `CanonicalScreen` | `unode` core |
| Host to renderer | `IrScreen` and `IrPatchOp` | `unode-web-host` or native host |
| Renderer to host/plugin | `ActionRef` | Renderer adapter + host runtime |
| Plugin to host capability | `host_call` envelopes | Permission-guarded host runtime |

The plugin does not choose DOM, CSS, terminal cells, React components, or Svelte
components. It describes intent. The trusted host turns that intent into a
canonical tree and renderer IR. The app renderer decides how that IR looks and
maps user interaction back to symbolic actions.
