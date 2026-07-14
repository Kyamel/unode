# unode Implementation Status

This file tracks the refactor work that has actually landed in the repository, so the docs do not drift away from the code.

## Declarative plugin authoring landed

The core now exposes a more discoverable authoring API:

- `definePlugin({...})`
- `route('/path').load(...).render(...)`
- `msg('key')` for lazy setup-time i18n

The tests and the real plugin modules now use this style in production code.

This matters because plugin authors no longer need to infer the route lifecycle from a raw object type or from docs alone. The contract is visible directly in the builder chain and the declarative plugin object shape.

## Phase 1 applied

The following pieces were added as a new parallel core under `src/lib/unode/core/`:

- `immutable.ts`
  Runtime deep-freeze helper plus `Immutable<T>`.
- `ast.ts`
  Smaller canonical AST aligned with `docs/unode/AST.md`.
- `i18n.ts`
  Core JSON-catalog registry and translator helpers.
- `runtime.ts`
  Core runtime and plugin contracts in type form.
- `dsl.ts`
  Initial authoring sugar that emits frozen canonical nodes.
- `index.ts`
  Barrel export for the new core.

The package exports were also updated so the new core can be imported through the `unode` package subpath.

## What this phase does not migrate yet

This phase does not replace the current implementation.

The existing runtime and Web renderer remain the active production path:

- `src/lib/unode/api`
- `src/lib/unode/runtime`
- `src/lib/widgets/app-plugin-renderer`
- `src/lib/widgets/app-plugin-shell`

So, at this point, the repo has:

- a legacy plugin/runtime path that is still in use
- a new `core vNext` path that defines the target contracts in code

## Architectural decisions reflected in code

The new core now encodes these decisions directly:

- immutable AST output is mandatory
- the canonical AST is smaller and excludes `table`
- i18n is part of core and uses plugin-registered JSON catalogs
- `commands`, `navigation`, and `providers` are modeled as core concerns
- the app bridge remains the place for domain APIs and domain permissions

## Known gaps after Phase 1

- the current renderer does not consume `src/lib/unode/core/ast.ts` yet
- the current runtime does not use the new core i18n layer yet
- the current plugins still author against the legacy UI DSL
- the current refresh flow is still more route-driven than binding-driven
- compound nodes in the existing renderer have not been migrated to sugar yet

## Suggested next step

Phase 2 should focus on adaptation, not replacement:

1. introduce compatibility adapters between the legacy UI model and the new core AST
2. begin migrating the runtime toward the new core contracts
3. keep the current Web renderer working while the internals shift

## Phase 2 applied

The repository now also includes a compatibility layer under `src/lib/unode/core/compat/`:

- `legacy-actions.ts`
  Maps core symbolic actions to the legacy action shape and back.
- `legacy-ui.ts`
  Adapts the core AST into the current renderer node model on a best-effort basis.
- `plugin.ts`
  Adapts a core plugin definition into the current runtime plugin module contract.
- `state.ts`
  Adds a reusable in-memory state store implementation for the new runtime contracts.

This means Phase 2 has started connecting the new architecture to the real runtime instead of keeping it purely parallel.

## Current limitations of the compatibility layer

The compatibility layer is intentionally conservative.

- It keeps the current Web renderer in place.
- It degrades some core nodes into simpler legacy equivalents.
- It does not yet deliver full binding-driven local reactivity in the current renderer.
- Inline slot semantics are still limited by the legacy screen/section model.
- Some newer core semantics are adapted approximately rather than natively.
- Imperative `dispatch()` in compat mode currently supports only builtin core actions like navigation and local state writes.

That is acceptable for this phase because the goal is compatibility and migration safety, not final parity.

## Validation status

Phase 2 now passes the project's validation commands:

- `npm run check`
- `npm run test:unit -- --run`
- `npm run test:e2e`

In this environment, Playwright needed to run outside the default sandbox so the preview server could bind to port `4173`. The code itself passed once the server was allowed to start.

## Phase 3 applied

The active runtime now consumes more of the new architecture directly instead of relying only on edge adapters.

- `src/lib/unode/runtime/runtime.ts`
  The runtime can now activate both legacy plugin modules and `core vNext` plugin definitions directly. Core plugins are adapted internally during activation instead of requiring callers to do it manually.
- `src/lib/unode/runtime/loader.ts`
  The loader contract now allows imported modules to be either legacy plugins or core plugins.
- `src/lib/unode/runtime/context.ts`
  `ctx.events` now uses the host event bus instead of an isolated per-plugin bus, which means plugin-to-plugin events flow through the real runtime boundary.
- `src/lib/unode/runtime/runtime.spec.ts`
  Added coverage for direct core-plugin activation and cross-plugin events through the host bus.

## What Phase 3 still does not migrate

- The Web renderer still renders the legacy UI node model.
- The screen shell still keeps a legacy `uiState` layer separate from the new `StateStore`.
- Core bindings and local reactivity still degrade into the current renderer model instead of being rendered natively.

So after Phase 3, the runtime boundary is notably closer to the target architecture, while the renderer boundary is still the main legacy holdout.

## Experimental core-authored plugin landed

The repository now also includes an in-app migration lab for the new authoring model:

- `src/lib/plugins/tests/banner-lab/index.ts`
  A real plugin authored directly against `unode core vNext`, not the legacy plugin DSL.
- `src/lib/plugins/tests/banner-lab/ui.ts`
  A local sugar layer that composes a higher-level `workBanner()` from core primitives.
- `src/lib/plugins/tests/banner-lab/messages/*.json`
  Core i18n catalogs registered by the plugin itself.
- Route:
  `/app/tests/banner`

This plugin exists specifically to trial higher-level UI authoring on top of the canonical AST before extracting shared sugar into reusable libraries.

## What the banner lab proves

- Plugins can now be authored directly against the new core contracts.
- Core i18n registration works in a real plugin.
- Local sugar can make authoring simpler than writing the raw AST by hand.
- Higher-level UI built from primitives can already render through the current Web runtime via the compatibility layer.

## Shared banner component extracted

The work banner used in the lab is no longer local-only.

- `src/lib/plugins-bridge/components/workBanner.ts`
  Shared builder for the manga/work banner layout
- `src/lib/plugins-bridge/components/index.ts`
  Re-export surface for shared plugin-bridge components
- `src/lib/plugins-bridge/components/workBanner.spec.ts`
  Coverage for the shared banner builder

The banner lab now consumes this shared builder instead of owning its own copy, which means other plugins can reuse the same layout contract immediately.

## Metadata layout and reactive details lab landed

The banner lab page now also includes a second exploratory UI built from primitives:

- a responsive metadata layout with the cover beside a two-column metadata grid
- a reactive `show more details` / `show less details` control

This matters because it proves two additional things in the real app:

- primitives are already expressive enough to build richer detail layouts without dropping to Svelte
- local state bindings can already drive richer user interactions in the lab route

## Responsive grid support landed in core

The canonical `grid` node now supports responsive column maps, not just a single maximum column count.

- Core contract:
  `columns: { base, sm, md, lg, xl }`
- Compatibility layer:
  responsive grid columns are preserved when adapting into the legacy Web renderer model
- Banner lab usage:
  the gallery now scales as `1 / 2 / 3 / 4 / 5` columns from base to `xl`

This is important because sugar components like banners are only genuinely reusable if the layout contract can express different densities across mobile, tablet, and desktop.

## Disclosure primitive landed

The canonical AST now includes a dedicated `disclosure` node for inline expandable content.

- `src/lib/unode/core/ast.ts`
  Adds `DisclosureNode` to the core AST union
- `src/lib/unode/core/dsl.ts`
  Adds `ui.disclosure(...)` to the authoring DSL
- `src/lib/unode/api/ui-types.ts`
  Adds a matching legacy renderer node shape
- `src/lib/unode/core/compat/legacy-ui.ts`
  Adapts core disclosure nodes into the active renderer path
- `src/lib/widgets/app-plugin-renderer/nodes/DisclosureNode.svelte`
  Native Web renderer for disclosure semantics

This matters because the old `show more` pattern was implemented as a generic action plus a conditional block, which leaked implementation details into the plugin authoring layer.

With `disclosure`, the plugin now declares:

- there is a trigger
- it controls adjacent collapsible content
- the expanded state is stored in a boolean binding

That gives the renderer enough information to own the interaction and presentation more cleanly.

The banner lab metadata section now uses `ui.disclosure(...)` instead of composing the toggle manually from `action + setState + when`.

## Expandable media landed

The canonical `media` node now supports an opt-in expandable intent.

- `src/lib/unode/core/ast.ts`
  Adds `expandable?: boolean` to the core media node
- `src/lib/unode/core/dsl.ts`
  Exposes that capability in the core authoring API
- `src/lib/widgets/app-plugin-renderer-core/nodes/CoreMediaNode.svelte`
  The Web renderer opens a fullscreen detail view when expandable media is activated

This matters because clicking a cover to inspect it more closely is a renderer concern, not a plugin-specific workaround.

The metadata lab cover now opts into `expandable: true`, while the shared `workBanner` intentionally does not because it is being reserved for work-detail navigation flows.

## Explicit node identity is now enforced

The canonical normalizer and the active Web renderer no longer fall back to structural tree paths or array indexes for node identity.

What changed:

- canonical `_key` is now derived only from explicit `key` or `id`
- normalization throws if a node is missing stable identity
- menu items now require explicit `key`
- renderer keyed blocks use canonical `_key` only

This is important because it removes a class of subtle bugs where identity changed accidentally after tree reshaping, collection reordering, or environment-specific render differences.

## Identity helpers landed in the bridge

The bridge now exposes small helpers for explicit but less verbose node identity:

- `nodeScope(base)`
- `entityScope(kind, id)`
- `scopedUi(scope)`
- `withIdentity(scope, part, value)`

These helpers do not reintroduce implicit identity.

They still require the plugin author to choose semantic names such as:

- `page`
- `intro`
- `intro.title`
- `pressable:work-id`

The difference is that plugins no longer need to hand-build long string prefixes everywhere.

## Shared metadata components extracted

The banner lab no longer owns the metadata layout helpers inline.

- `src/lib/plugins-bridge/components/workMetadata.ts`
  Shared builders for:
  - the responsive work metadata layout with cover + summary grid
  - the disclosure for advanced details
  - the summary grid in isolation
- `src/lib/plugins-bridge/components/workMetadata.spec.ts`
  Coverage for the shared metadata builders
- `src/lib/plugins-bridge/components/index.ts`
  Re-export surface for the new shared metadata helpers

The banner lab now consumes these shared builders instead of defining the whole metadata composition locally, which makes it much easier to reuse the same layout contract in future detail-oriented plugins.

## Shared work components now use view models

The shared work-oriented components in `plugins-bridge` no longer receive raw domain objects plus translation helpers as their public contract.

- `src/lib/plugins-bridge/components/work-banner/`
  The banner component and its view-model factory now live together in a dedicated subfolder.
- `src/lib/plugins-bridge/components/work-metadata/`
  The metadata layout/disclosure components and their view-model factory now live together in a dedicated subfolder.

The public direction is now:

- build a view model from domain data at the plugin or bridge boundary
- pass that view model into the shared UI component builder

This makes the component layer more stable and more clearly presentation-focused, while keeping domain labeling, fallbacks, and formatting near the bridge boundary instead of inside the component contract itself.

## Incremental list continuation now autoloads on scroll in Web

The Web renderer now treats incremental list continuation the same way it already treated grid continuation:

- `src/lib/widgets/app-plugin-renderer/nodes/ListNode.svelte`
  The continuation sentinel is observed with `IntersectionObserver`
- autoload only starts after the user has actually scrolled the main container
- incremental local continuation can also prime itself on first paint when the sentinel is already in view
- the existing `Load more` button remains in place as a fallback affordance

This keeps the core contract portable:

- filters can still remain route/query-driven
- incremental density stays local and SPA-like
- TUI is not forced into a browser-style URL or infinite-scroll model

The `tests/chapters` lab now proves this behavior by autoloading more chapters when the continuation area enters the viewport after scroll, without requiring a route reload.

## Incremental grid continuation landed

The canonical `grid` node now supports the same local incremental continuation model as `list`.

- `src/lib/unode/core/ast.ts`
  Grid nodes can declare incremental continuation directly in the core AST
- `src/lib/unode/core/dsl.ts`
  `ui.grid(...)` now accepts the same continuation contract
- `src/lib/unode/core/compat/legacy-ui.ts`
  Core grid continuation is preserved into the active renderer model
- `src/lib/widgets/app-plugin-renderer/nodes/GridNode.svelte`
  The Web renderer now slices visible children locally, keeps the fallback button, and autoloads more cards on scroll

The banner lab gallery now uses this contract so larger card/banner collections can prove the same SPA-like density behavior as the chapters list.

## Menu primitive landed

The canonical AST now includes a dedicated `menu` node for anchored action menus.

- `src/lib/unode/core/ast.ts`
  Adds `MenuNode` and `MenuItem`
- `src/lib/unode/core/dsl.ts`
  Adds `ui.menu(...)` and `ui.menuItem(...)`
- `src/lib/unode/core/compat/legacy-ui.ts`
  Adapts core menus into the active renderer model
- `src/lib/widgets/app-plugin-renderer/nodes/MenuNode.svelte`
  Native Web renderer for the anchored menu interaction

This matters because some interactions are better expressed as a renderer-owned popup menu than as a generic button or a native select.

## Chapter filter lab landed

The test plugins now include a second exploratory route:

- `src/lib/plugins/tests/chapters-lab/`
  A real plugin that uses the new `menu` primitive to filter a chapter list by language
- Route:
  `/app/tests/chapters`

This lab intentionally uses URL query params to drive the filter state, proving that the current architecture can support full rerender flows for filter UIs while still authoring the visible interaction declaratively through the core DSL.

## Incremental list continuation landed

The canonical `list` node now supports a renderer-owned incremental continuation mode.

- Core contract:
  `continuation: { kind: 'incremental', binding, initial, step, label? }`
- Current Web renderer:
  slices the visible items locally and grows the list without rerendering the whole route
- Current chapters lab usage:
  language stays route/query-driven, while `load more chapters` is local SPA-like state

This matters because it proves that `unode` can combine:

- route-driven filters for shareable navigation state
- renderer-local progressive reveal for collection density

without forcing every collection interaction into full route reload semantics.

## Shared chapter components extracted

The chapters lab no longer owns its filter and list composition inline.

- `src/lib/plugins-bridge/components/chapter-language-filter/`
  Shared view-model factory and toolbar component for the route-driven language filter menu
- `src/lib/plugins-bridge/components/chapter-list/`
  Shared view-model factory and incremental chapter list component
- `src/lib/plugins/tests/chapters-lab/ui.ts`
  Now consumes those shared components instead of defining the filter/list builders locally

This brings the chapters lab into the same architectural pattern already used by the work banner and work metadata components:

- domain data is adapted into a view model at the edge
- the shared component layer stays focused on declarative UI composition

## Route tabs moved into screen chrome

Manga subroute tabs no longer depend on an ad-hoc helper under `plugins/mangas/tabs.ts`.

- `src/lib/plugins-bridge/screen-chrome/route-tabs.ts`
  Defines a bridge-level `routeTabs` screen metadata contract
- `src/lib/plugins-bridge/domains/manga/route-tabs.ts`
  Builds the manga browse/work tab groups as domain navigation metadata
- `src/lib/widgets/app-plugin-shell/ScreenHost.svelte`
  Renders route tabs as shell chrome via `RouteTabsLayout.svelte`, keeping swipe and scroll-memory in the renderer layer instead of inside each plugin body

## Remote collection continuation landed

The canonical AST now supports renderer-visible remote continuation for `list` and `grid`.

- `src/lib/unode/core/ast.ts`
  Adds `remote` continuation alongside the existing local incremental continuation
- `src/lib/unode/core/compat/legacy-ui.ts`
  Preserves remote continuation into the active renderer model with `hasMore`, `loadMore`, and loading labels
- `src/lib/widgets/app-plugin-renderer/nodes/ListNode.svelte`
  Already supported action-based continuation and now consumes the core contract through compat
- `src/lib/widgets/app-plugin-renderer/nodes/GridNode.svelte`
  Already supported action-based continuation and now consumes the core contract through compat

The `tests/banner` lab now exercises this contract with real cursor-backed fetching:

- first page is loaded normally
- additional pages are fetched through a core action
- the renderer can autoload on scroll
- the button remains available as fallback

## Phase 4 applied

All real app plugins under `src/lib/plugins/mangas/` now author directly against the `unode core` contracts.

- Browse routes:
  - `browse-hot`
  - `browse-recent`
  - `browse-recommended`
  - `browse-friends`
- Work routes:
  - `work-meta`
  - `work-chapters`
  - `work-staff`
  - `work-related`

This means the repo no longer has production plugins authored against the old `PluginModule + ctx.ui + uiBuilder` plugin API.

## What changed in the plugin layer

- Each manga plugin now exports a `PluginDefinition` and registers JSON catalogs through core i18n.
- Shared composed UI moved further into `plugins-bridge/components/`, especially:
  - `work-list/`
  - `work-banner/`
  - `work-metadata/`
  - `chapter-language-filter/`
  - `chapter-list/`
- The old per-plugin `messages.ts` translator wrappers were deleted because plugins now register their catalogs directly with the core i18n registry.
- Route-tab chrome remains a bridge-level concern through `screen.meta.routeTabs`, instead of being assembled manually inside every plugin body.

## Legacy code removed in this phase

- Deleted the old `messages.ts` wrappers for all migrated manga plugins.
- Deleted the unused `src/lib/plugins-bridge/legacy-ui.ts` helper.
- Removed the last in-repo uses of the legacy `navAction()` helper.

## Legacy code intentionally still present

Some legacy code remains because the active Web renderer still consumes the legacy renderer node model.

- `src/lib/unode/api/ui-types.ts`
- `src/lib/widgets/app-plugin-renderer/`
- `src/lib/widgets/app-plugin-shell/`
- `src/lib/unode/core/compat/`
- `src/lib/unode/api/manifest.ts`

So after Phase 4, the plugin authoring layer is migrated, but the renderer boundary is still compat-driven.

## Validation status after Phase 4

The migrated plugin layer passes the project validation commands:

- `npm run check`
- `npm test`

## Representation layers and normalization landed

The core now has an explicit normalization stage between authoring DSL and renderer lowering.

- `src/lib/unode/core/normalize.ts`
  Defines the current representation-layer vocabulary in code:
  - `AuthorUiNode`
  - `CanonicalUiNode`
  - `TransportUiNode`
  - `normalizeNode(...)`
  - `normalizeScreen(...)`
  - `toTransportNode(...)`
  - `toTransportScreen(...)`
- `src/lib/unode/core/normalize.spec.ts`
  Covers default-filling, subtree-reactivity propagation, and the current transport behavior
- `src/lib/unode/registries/routes.ts`
  Core-authored routes normalize their rendered screens before they are composed by the runtime

This does **not** mean the repo has a bytecode layer or a compact transport protocol yet.

Right now the practical pipeline is:

1. author with the core DSL
2. normalize into canonical IR
3. hand canonical IR to the active runtime/render pipeline

The canonical metadata now distinguishes between:

- `_reactivity`
  Reactivity owned by the node itself
- `_subtreeReactivity`
  Reactivity anywhere under that node, including descendants
- `_staticFields`
  Primitive fields already resolved during normalization

That split matters for the renderer because it can skip fully static subtrees without misclassifying
layout/container nodes that wrap reactive descendants.

That is important because it gives the next renderer refactor a cleaner seam:

- the renderer can consume normalized core IR directly
- transport compaction can be added later without changing authoring
- validation/default resolution no longer needs to live inside random renderer paths

## Legacy renderer path removed

The active runtime and Web shell now consume canonical core IR directly.

- `src/lib/unode/api/contracts.ts`
  `ScreenDefinition` and `ResolvedScreen` now carry canonical `body` and canonical `slots` only.
- `src/lib/unode/runtime/context.ts`
  Plugin activation now builds a core-native setup context instead of adapting core plugins into the old plugin API.
- `src/lib/unode/runtime/runtime.ts`
  The runtime activates only core plugins and composes canonical screens without `bodyCore` or legacy fallback fields.
- `src/lib/unode/registries/routes.ts`
  Core routes are registered directly and normalized before composition.
- `src/lib/unode/registries/screens.ts`
  Slot contributions now resolve to canonical core nodes.
- `src/lib/unode/registries/actions.ts`
  Action dispatch now uses core `ActionRef` objects directly.
- `src/lib/widgets/app-plugin-shell/ScreenHost.svelte`
  The shell renders canonical `resolvedScreen.body` only.
- `src/lib/widgets/app-plugin-shell/SlotRenderer.svelte`
  Slots render through the core renderer path as well.
- `src/lib/widgets/app-plugin-renderer-core/`
  The core Web renderer is now the only active renderer path.

Files removed in this step:

- `src/lib/unode/api/ui.ts`
- `src/lib/unode/api/ui-types.ts`
- `src/lib/unode/api/manifest.ts`
- `src/lib/unode/renderer/adapter.ts`
- `src/lib/unode/core/compat/legacy-actions.ts`
- `src/lib/unode/core/compat/legacy-ui.ts`
- `src/lib/unode/core/compat/plugin.ts`
- `src/lib/widgets/app-plugin-renderer/`

The practical pipeline is now:

1. author with the core DSL
2. normalize into canonical IR
3. render canonical IR directly in the Web shell

The legacy plugin API and the legacy Web renderer are no longer part of the active implementation.

## Renderer state now uses the core store directly

The active Web shell no longer keeps a second mutable UI state object beside the core store.

- `src/lib/unode/api/contracts.ts`
  `ResolvedScreen` now carries the screen `StateStore` through the runtime boundary.
- `src/lib/unode/registries/routes.ts`
  Core route resolution now returns the `MemoryStateStore` instance together with the canonical screen.
- `src/lib/widgets/app-plugin-shell/ScreenHost.svelte`
  The shell now binds renderer reactivity to the screen `StateStore` instead of maintaining a separate Svelte-owned `uiState` object.
- `src/lib/widgets/app-plugin-renderer-core/context.ts`
  Renderer context now exposes a state-store bridge rather than a Web-only record-shaped UI state helper.
- `src/lib/widgets/app-plugin-renderer-core/`
  Renderer nodes now resolve bindings from the core store directly.

This matters because there is now a single state source of truth for local SPA-like interactions:

- the core `MemoryStateStore` stays portable to TUI
- the Web renderer only adds a tiny rerender bridge, not a second data store
- local bindings, disclosures, menus, and incremental collection state no longer depend on Svelte-local mirrored data

## Structural cleanup after legacy removal

The remaining migration leftovers have now been renamed into their final homes:

- `src/lib/unode/core/state.ts`
  `MemoryStateStore` now lives directly in the core package, rather than under a stale `core/compat` path.
- `src/lib/widgets/app-plugin-renderer-core/context.ts`
  The active renderer context now lives alongside the active renderer implementation.

The old `src/lib/unode/core/compat/` barrel and the old `src/lib/widgets/app-plugin-renderer/` directory are no longer
part of the live implementation.

## Root screen and initial focus tightened

The core contracts now treat `screen` as a root object, not a regular child node.

- `src/lib/unode/core/ast.ts`
  `UiNode` no longer includes `ScreenNode`; `RootNode` now represents `screen | UiNode`.
- `src/lib/unode/core/normalize.ts`
  Normalization keeps `screen` on its own root-only path.
- `src/lib/unode/core/dsl.ts`
  `screen(...)` now also supports `initialFocus?: string`.
- `src/lib/widgets/app-plugin-shell/ScreenHost.svelte`
  The Web shell now honors `screen.initialFocus` by focusing the matching rendered node id after mount/navigation.

This matters because:

- invalid nested `screen` trees are prevented by type
- renderers can assume a simpler root contract
- TUI now has an explicit parity hook for initial focus/cursor placement

For collection continuation, the intended cross-platform rule is now explicit:

- Web may keep `grid + continuation` as a dense card grid
- TUI may degrade the same contract to a linear continuation flow while preserving behavior
