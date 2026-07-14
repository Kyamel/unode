# unode Host Bridge

## Why the bridge exists

`unode` should stay app-domain agnostic.

That means the core must not know about:

- works
- chapters
- posts
- users
- auth
- reader progress

Each app should define a host bridge that extends `unode` with its own domain capabilities.

## Bridge responsibilities

The app bridge is responsible for:

- typed domain APIs exposed to plugins
- domain permission strings
- permission metadata for those APIs
- optional app-level sugar helpers

Examples of bridge-level APIs:

- `catalog.getWorkById`
- `library.addFavorite`
- `reader.openChapter`
- `users.getCurrentUser`

None of those belong in `unode core`.

## Recommended bridge shape

The bridge should give plugins a typed `ctx.api` that is:

- app-specific
- permission-checked
- still abstracted away from the host implementation

Recommended pattern:

1. The app defines `THostApi`.
2. The app defines app permission strings.
3. The app defines metadata mapping methods to required permissions.
4. The renderer or host wraps the real implementation with a permissioned proxy.
5. Plugins receive only the wrapped API.

## Two permission layers

The architecture should keep two permission layers distinct.

### Core built-in permissions

These gate generic capabilities such as:

- HTTP
- storage
- events

These are enforced by the renderer or runtime because only it knows how those capabilities are backed on a given platform.

### App domain permissions

These gate app-specific methods such as:

- `catalog.read`
- `library.write`
- `reader.open`

These are enforced by the host bridge proxy using method-level metadata.

The current codebase already has this idea in spirit, but the target model should make it more granular and more explicit than today's coarse group-based guards.

## i18n stance

Core i18n belongs in `unode`, not in the bridge.

That means:

- plugins register JSON catalogs through the core
- plugins use core translation helpers
- the bridge does not need to provide i18n for normal plugin authoring

The bridge may still expose domain-specific formatting helpers if an app really needs them, but localization itself should not depend on the app bridge.

## Bridge-level extras

Some capabilities are useful but do not necessarily belong in `unode core`.

Examples:

- host shell section contributions

Command, navigation, and provider registries are considered core. Host-shell specific presentation of them may still live in the bridge or renderer.

## Practical consequence for the current codebase

`src/lib/plugins-bridge` is already the right place in the repo for this idea. The next architectural step is to make the relationship cleaner:

- `src/lib/unode`
  generic core
- `src/lib/plugins-bridge`
  Mugen-specific domain bridge
- renderer implementation
  platform-specific adapter

That split is already visible in the code. The docs here simply make it first-class.
