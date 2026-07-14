---
title: Reactivity
description: How Unode's granular, Solid-style reactive updates work — ExprResolver, StateStore, and targeted patches.
---

Unode uses **granular** reactivity: closer to Solid-style targeted updates than
to virtual-DOM diffing. A plugin renders once into a serializable AST. From then
on, a state write patches only the nodes that actually depend on the changed
path. `render()` is never called again for ordinary state writes.

## The flow

After the host normalizes a screen, it tracks reactive bindings:

```text
normalize_screen(raw_json)
  → CanonicalScreen with _reactivity metadata per node

track_reactive_bindings(screen, resolver, state, on_patch)
  → walk the tree once
  → resolver.track(node_key, path) for each binding found
  → subscribe to each path in the StateStore
  → return BindingSubscriptions { path_to_nodes, teardown }
```

When a state path changes:

```text
state.set("ui.favorited", true)
  → StateStore notifies subscribers of "ui.favorited"
  → resolver.subscribers_of("ui.favorited") → [node_key_A, node_key_B]
  → on_patch({node_key_A, node_key_B})
  → renderer re-evaluates ONLY those nodes
  → renderer patches only the affected output (DOM nodes / terminal cells)
```

The AST structure is fixed for the load cycle. Reactivity is purely expression
resolution against an updated StateStore.

## ExprResolver

The resolver maintains two maps in both directions:

```text
node_to_path: Map<NodeKey, Set<StatePath>>   "which paths does this node read?"
path_to_node: Map<StatePath, Set<NodeKey>>   "which nodes react to this path?"
```

Key operations:

- `track(node_key, path)` — records the dependency in both maps.
- `clear_tracking(node_key)` — removes a node before re-evaluation so stale
  subscriptions are cleaned up.
- `dependencies_of(node_key)` — used after the initial walk to set up
  subscriptions.
- `subscribers_of(path)` — called on a state change to find affected nodes. It
  includes **ancestor prefix matching**: a change at `work.title` also wakes
  nodes subscribed to `work`.

## StateStore

`MemoryStateStore` is the single source of truth for local screen state. It
lives in the host, **not** inside the plugin WASM.

- Paths are dot-separated: `work.title`, `ui.favorited`, `items.0.name`.
- Batched writes collapse into a single notification cycle.
- `reset()` restores the initial seed state on navigation.

## Static subtree optimization

Nodes whose `_subtreeReactivity == "static"` have no reactive descendants. The
renderer skips them entirely during the tracking walk and re-evaluation passes:

```text
walk(node):
  if node._subtreeReactivity == "static":
    return  // nothing reactive below — skip

  if node._reactivity != "static":
    resolve_node_expressions(node)

  for child in children(node):
    walk(child)
```

For typical screens — mostly static text and media — this is the most impactful
optimization.

## Crossing the WASM boundary

Plugin action handlers run inside WASM. A `state.set()` in a handler crosses the
boundary through a host function:

```text
Plugin WASM:  ctx.state.set("ui.favorited", true)
                → host function state_set(...)
Host:         StateStore.set("ui.favorited", true)
                → notify subscribers → patch affected nodes
```

For typical interactions (one to three writes per action) this overhead is
imperceptible. Tight loops should batch writes via `unode.batchState` so many
paths collapse into one notification cycle.

## Known granularity limits

The model is a strong baseline, with tradeoffs worth tracking:

- **Path breadth** — a binding to a broad object like `work` wakes on writes to
  nested paths like `work.title`. Prefer narrow bindings.
- **Node-level re-resolution** — patches target node fields, not arbitrary
  subexpressions inside a field, so composite text may re-resolve more than the
  exact changed fragment.
- **Explicit bindings only** — values computed inside plugin Rust and emitted as
  literals are opaque to the host. Host-side reactivity requires dependencies to
  stay visible in the AST as bindings.
- **Indexed list paths** — paths like `items.0.title` are fragile under
  insertion or reordering. Keyed collection identity is future work.
- **Stable node IDs** — precise patches depend on stable node keys. Interactive,
  stateful, or plugin-extension anchors should keep explicit IDs.

## Locale is not reactive state

The locale lives in the resolver context, not the StateStore. Because a locale
change typically affects most visible text, the host handles it with a full
`render()` re-run rather than tracking per-string locale dependencies.
