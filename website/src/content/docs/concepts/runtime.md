---
title: Runtime & Lifecycle
description: The plugin contract, route lifecycle, built-in actions, and Unode's state model.
---

The runtime drives plugins through their lifecycle: matching routes,
instantiating WASM, calling exports, and managing state. This page describes the
contract a plugin fulfills and how the host runs it.

## Plugin contract

A plugin is a WASM module that exports five lifecycle entry points:

```text
plugin_manifest()          -> manifest JSON
plugin_load(request)       -> data JSON (merged into StateStore)
plugin_render(request)     -> CanonicalScreen JSON
plugin_render_slot(request) -> slot contribution JSON
plugin_dispatch(request)   -> dispatch response (side effects via host calls)
```

Plugin authors write ordinary Rust functions and expose them with the SDK's
`export_plugin!` macro, which generates the raw ABI exports. See the
[Quickstart](/getting-started/quickstart/) for a complete example and the
[Plugin WASM ABI](/reference/plugin-abi/) for the raw symbol list.

## Route lifecycle

```text
1. Route match
   RouteRegistry.resolve(pathname) -> (plugin_id, pattern, params)

2. Plugin instantiation (if not cached)
   -> validate required permissions
   -> inject host functions gated by the PermissionProfile
   -> call plugin init if exported

3. Fresh StateStore
   MemoryStateStore::new()  // seeded from ScreenNode.initialState after render

4. load(request)  -> data JSON  -> host merges into the StateStore

5. render(request) -> raw CanonicalScreen JSON (before normalization)

6. normalize_screen(raw)
   -> fill defaults, compute _reactivity / _subtreeReactivity / _staticFields
   -> validate id uniqueness, assign structural _key fallbacks
   -> merge screen.initialState into the StateStore

7. resolve slots
   -> call plugin_render_slot() for declared contributors
   -> normalize returned nodes and preserve contributor origin

8. track_reactive_bindings(screen, resolver, state, on_patch)
   -> walk the static-skipping tree, register path subscriptions

9. lower + mount
   lower_screen(canonical) -> renderer.mount(ir_screen)

10. Reactive loop (per interaction)
   dispatch ActionRef -> plugin requests state writes -> host applies them
   -> resolver.subscribers_of(path) -> plan_patch_ops -> patch only those nodes

11. Teardown on navigation
    subscriptions.teardown(); state.reset()
    // compiled modules may be cached; guest sessions are disposable
```

## Registries

Plugins populate host registries at activation. These are generic -- the host
carries no domain knowledge:

- **Routes** -- pattern -> (load fn, render fn). Zero declared routes means the
  plugin has no screen surface of its own.
- **Actions** -- type string -> handler fn
- **Commands** -- id -> (title, category, handler)
- **Navigation** -- id -> (label, path, priority)
- **Providers** -- capability -> provider fn
- **Slots** -- target slot name -> contribution fn

The host queries these to build navigation menus, command palettes, and slot
contributions.

## Built-in actions

These action types are handled by the host before reaching a plugin's action
registry:

| Action type        | Params                    | Effect                          |
| ------------------ | ------------------------- | ------------------------------- |
| `unode.setState`   | `{ path, value }`         | Writes to the StateStore        |
| `unode.navigate`   | `{ to, mode?, query? }`   | Triggers navigation             |
| `unode.refresh`    | `{}`                      | Triggers a full load/render     |
| `unode.batchState` | `{ [path]: value }`       | Batches multiple state writes   |

All other action types are dispatched to plugin action handlers.

## State model

**Local screen state (SPA-like)** -- ephemeral and screen-scoped. Driven by
`unode.setState` actions and direct state writes, it lives in the host's
`MemoryStateStore`, is destroyed on unmount, and drives `binding` expression
evaluation. Typical uses: disclosure expanded/collapsed, filter selections,
draft input, optimistic UI flags.

**Route state (URL-like)** -- addressable and shareable. On the web it maps to
the URL path + query string; in the TUI it is a breadcrumb stack plus a params
object. It survives back/forward navigation and is read via `param`
expressions. Typical uses: selected item id, search query, active page.

**Persistence across sessions** -- via `ctx.storage`, namespaced per plugin, with
session or persistent scope. Gated by the `storage.*`
[permissions](/concepts/permissions/).

## Optimistic updates

Action handlers can write optimistically, call a domain API, and roll back on
failure:

```rust
// optimistic write
ctx.state.set("ui.favorited", json!(true));

match ctx.api::<LibraryApi>().add_favorite(work_id).await {
    Ok(()) => {}
    Err(_) => {
        // rollback
        ctx.state.set("ui.favorited", json!(false));
        ctx.state.set("ui.favoriteError", json!("Failed. Try again."));
    }
}
```

## Plugin caching

WASM modules are expensive to parse and compile, so hosts cache compiled modules
where possible. Mounted guest sessions are disposable isolation units: they are
dropped on route leave, plugin switch, trap, or explicit reload. Only future
pooling behind a proven reset contract should reuse live instances across
activations.
