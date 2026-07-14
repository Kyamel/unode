# unode Runtime

## Runtime goal

The runtime should let plugins behave like small applications without coupling them to a specific platform UI stack.

The intended split is:

- `load()`
  Fetches serializable data.
- `render()`
  Produces a semantic AST for the current load cycle.
- local state
  Drives fine-grained SPA-like interactions after mount.
- route changes
  Trigger fresh load/render cycles when the addressable state changes.

## Plugin contract

At minimum, the core plugin contract should include:

- manifest
- routes
- action handlers
- commands
- navigation items
- providers
- optional slot contributions

These registries are part of the `unode` core because they are general plugin-platform concerns, even though each host may style or surface them differently.

## Preferred authoring API

The preferred current authoring model is:

- `definePlugin({...})`
- `route('/path').load(...).render(...)`
- `msg('translation.key')` for setup-time labels

This is preferred over a large imperative `activate(ctx)` body because it makes the plugin contract easier to discover through code completion alone.

Example:

```ts
export default definePlugin({
  manifest: { ... },
  i18n: { en, 'pt-br': ptBr },
  navigation: [
    { id: 'demo.nav', label: msg('nav_label'), to: '/demo' }
  ],
  routes: [
    route<MyData, MyHostApi>('/demo')
      .load(async ({ api }) => {
        return { items: await api.catalog.listWorks({ limit: 10 }) };
      })
      .render((data, ctx) => {
        const t = ctx.i18n.t;
        return ui.screen({ id: 'demo:screen', title: t('screen_title') }, []);
      })
  ]
});
```

## Route lifecycle

Recommended route lifecycle:

1. Match route.
2. Build plugin context.
3. Create fresh screen state store.
4. Seed initial state if the AST or route declares defaults.
5. Run `load(ctx)`.
6. Merge returned serializable data into state.
7. Run `render(data, ctx)` exactly once for that load cycle.
8. Mount and subscribe.
9. Handle local state updates without re-running `render()`.
10. On route change or explicit refresh, tear down and start a new cycle.

## State model

The runtime needs two distinct state dimensions.

### 1. Local screen state

Properties:

- ephemeral
- screen-scoped
- good for SPA-like interactions
- used by `binding` expressions
- destroyed on unmount unless intentionally persisted

Typical examples:

- disclosure state
- selected tab within a current view
- optimistic flags
- draft search text

### 2. Route state

Properties:

- addressable
- shareable
- deep-linkable
- platform-specific in storage, but platform-agnostic in concept

Typical examples:

- route params
- query params
- stack location
- history entries

## Reactivity contract

The runtime should support granular local reactivity.

The target contract is:

- `render()` runs once per load cycle
- the renderer walks the AST and resolves expressions
- bindings are tracked per node
- state writes re-evaluate only affected nodes or subtrees

This is the SPA-like mode.

At the same time, the runtime should also support explicit full-cycle invalidation:

- navigation to a new route
- query change
- explicit refresh
- mutation-driven invalidation

This is the route-driven mode.

## Actions

Actions should remain symbolic and serializable.

Recommended built-ins:

- `unode.navigate`
- `unode.setState`
- `unode.refresh`

Everything else should be dispatched to plugin or bridge-level action handlers.

The important rule is not the exact action names. The important rule is that:

- plugins declare intent symbolically
- renderers and runtimes interpret built-ins consistently
- action payloads stay serializable

## Events

The runtime may expose a generic event bus, but the semantics must be clear.

Good uses:

- plugin-to-plugin notifications
- host invalidation signals
- lifecycle or locale changes

Bad uses:

- replacing typed host APIs with stringly-typed events
- leaking platform concerns into plugins

If an event bus exists, it should be genuinely shared at the runtime or host level, not silently isolated per plugin context.

## Core i18n

`unode` should include core i18n helpers.

Recommended model:

- plugins ship JSON message catalogs
- plugins register those catalogs with the core runtime
- the core exposes locale-aware translation helpers in plugin context
- the emitted AST still remains serializable and renderer-agnostic

This gives plugin authors a simpler authoring experience while avoiding host-specific i18n coupling.

## Storage and persistence

The core may expose generic storage capabilities like:

- session-scoped storage
- persistent storage

Rules:

- storage must be namespaced per plugin
- the renderer or host owns the real backing store
- permissions should gate access

Caching policy is useful, but it should remain outside the canonical AST itself.

## Recommended authoring rules

Plugins should:

- keep `load()` side-effect free except for data access
- keep `render()` pure and synchronous
- prefer local state for transient UI
- prefer route state for shareable or reload-worthy state
- use stable keys on dynamic collections
- treat the AST as immutable output
- register their translation catalogs through the core i18n API rather than depending on host-only i18n services

Plugins should not:

- import renderer internals
- depend on DOM APIs
- embed styling assumptions into AST values
- rely on host-only objects directly

## Practical consequence for the current codebase

The current runtime already has useful route and registry concepts, but it still leans on full screen recomposition more than the target architecture. The next major step is introducing a true local state and expression contract without losing the route-driven flow that already works.
