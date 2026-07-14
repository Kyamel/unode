# unode Authoring

This file describes the preferred authoring style for plugins that target the current `unode` core.

The goal is that plugin authors can discover the contract by reading code and autocomplete, without having to memorize hidden runtime conventions.

## Preferred shape

The preferred plugin shape is:

```ts
import { definePlugin, msg, route, UNODE_CORE_API_VERSION } from '$lib/unode/core/runtime';

export default definePlugin({
  manifest: {
    id: 'tests.example',
    name: 'Example',
    version: '0.1.0',
    apiVersion: UNODE_CORE_API_VERSION
  },
  i18n: {
    en,
    'pt-br': ptBr
  },
  navigation: [
    {
      id: 'tests.example.nav',
      label: msg('nav_label'),
      shortLabel: msg('nav_short'),
      to: '/app/tests/example'
    }
  ],
  commands: [
    {
      id: 'tests.example.open',
      title: msg('command_title'),
      category: 'Tests',
      run: ({ host }) => host.navigation.navigate('/app/tests/example')
    }
  ],
  routes: [
    route<MyData, MyHostApi>('/app/tests/example')
      .load(async ({ api, i18n, route, state }) => {
        const t = i18n.t;
        return {
          items: await api.catalog.listWorks({ limit: 10 })
        };
      })
      .render((data, ctx) => {
        const t = ctx.i18n.t;
        return ui.screen(
          {
            id: 'tests.example:screen',
            title: t('screen_title')
          },
          []
        );
      })
  ]
});
```

## What `definePlugin(...)` is for

`definePlugin(...)` makes the plugin contract more obvious.

Instead of hiding everything inside `activate(ctx)`, the plugin can declare:

- `manifest`
- `i18n`
- `navigation`
- `commands`
- `actions`
- `routes`
- `slots`
- `providers`
- optional `setup(ctx)` for advanced cases

This is the preferred style for normal plugin authoring.

## What `route('/path').load(...).render(...)` is for

The route builder exists to make the route lifecycle visible in code:

1. declare the route pattern
2. define `load(ctx)`
3. define `render(data, ctx)`

This is easier to discover than a plain object literal because autocomplete can guide the author through the sequence.

## `load(ctx)`

`load(ctx)` should:

- fetch serializable data
- read route params and query
- read and write screen-scoped state when needed
- avoid UI concerns

The `ctx` passed to `load()` is a `PluginRenderContext`.

Useful fields:

- `api`
  app bridge/domain API
- `i18n`
  core plugin translator
- `route`
  params and query for the current route
- `state`
  per-screen state store
- `storage`
  namespaced persistent/session storage adapter
- `events`
  shared host event bus

## `render(data, ctx)`

`render(data, ctx)` should:

- stay pure and synchronous
- return an immutable `screen` node
- declare UI intent, not presentation

The renderer decides:

- styling
- layout details
- focus behavior
- platform-specific interaction

## `msg('key')`

Use `msg(...)` in setup-time registries such as:

- `navigation`
- `commands`
- `actions`

Example:

```ts
commands: [
  {
    id: 'tests.example.open',
    title: msg('command_title'),
    category: msg('command_category'),
    run: ({ host }) => host.navigation.navigate('/app/tests/example')
  }
]
```

This keeps translations lazy. The string is resolved when the registry is queried, not when the plugin is activated.

That means catalog registration order is no longer fragile.

## `t('key')` as the preferred call shape

Inside `load()` and `render()`, prefer aliasing the translator:

```ts
const t = ctx.i18n.t;
```

Then call:

```ts
t('screen_title');
```

Instead of:

```ts
ctx.i18n.t('screen_title');
```

This keeps the runtime API unchanged while making editor tooling such as
inlang Sherlock easier to apply to `unode` plugins.

## Local Sherlock setup per plugin

To avoid key collisions between independent plugins, each plugin can keep its
own local `project.inlang/settings.json` next to its `messages/` folder:

```json
{
  "$schema": "https://inlang.com/schema/project-settings",
  "modules": [
    "https://cdn.jsdelivr.net/npm/@inlang/plugin-message-format@4/dist/index.js",
    "https://cdn.jsdelivr.net/npm/@inlang/plugin-t-function-matcher@3/dist/index.js"
  ],
  "plugin.inlang.messageFormat": {
    "pathPattern": "./messages/{locale}.json"
  },
  "baseLocale": "en",
  "locales": ["en", "pt-br"]
}
```

Sherlock supports multiple projects in one repository, which makes this local
per-plugin setup a good fit for `unode`'s plugin-owned catalogs.

## When `setup(ctx)` still makes sense

`setup(ctx)` is still useful when the plugin needs imperative setup that does not fit cleanly in the declarative arrays.

Examples:

- conditional registration
- dynamic registration from host capabilities
- migration code during transition periods
- complex slot/provider wiring

The preferred default is still declarative `definePlugin(...)`.

## Recommended authoring rules

- Prefer `definePlugin(...)` over a manual `activate(ctx)` when possible.
- Prefer `route(...).load(...).render(...)` over raw route object literals.
- Prefer `msg(...)` for setup-time labels and titles.
- Keep `load()` for data and `render()` for UI.
- Use identity helpers like `nodeScope(...)` and `scopedUi(...)` to reduce key verbosity without reintroducing implicit identity.
