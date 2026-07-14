# Svelte Web Renderer — Implementation Plan

This document covers the target architecture for the unode web renderer built on Svelte 5.
It is written in the same spirit as the TUI renderer plan — concrete, implementation-focused,
and structured by dependency order.

The analysis is grounded in the current renderer (ScreenHost, CoreUiRenderer, CoreChildren,
and the individual node components) and the performance/architecture review that identified
the main gaps. The plan does not discard what works — it consolidates and corrects what is
already good.

---

## Current state: what works and what does not

### What is already correct

The node component split is right. Having `CoreDisclosureNode`, `CoreGridNode`,
`CoreListNode`, `CoreMediaNode`, `CoreMenuNode`, `CorePressableNode`, and `CoreItemNode`
as separate components is the correct decomposition — each is large enough to justify
isolation, and Svelte 5 can optimize them individually.

The `context.ts` contract for `ActionRunner` and `RendererStateStore` is the right
pattern. Svelte context is the correct mechanism to avoid prop-drilling across a deep
recursive tree.

The `resolve.ts` helpers (`resolveStringValue`, `resolveBooleanValue`, etc.) are clean
and reusable. They should not change.

The `RendererConfig` with separate `webRendererConfig` and `tuiRendererConfig` is the
right abstraction for sharing config shape across environments.

### The main problem: global state invalidation

The critical bug identified in the performance review is in `ScreenHost`:

```typescript
// ScreenHost.svelte — current
rendererStateUnsubscribe = nextState.subscribePrefix('', () => {
  rendererStateRevision += 1;  // ← any write to any path invalidates everything
});
```

And the store accessor pattern:

```typescript
get(path: string) {
  void rendererStateRevision;  // ← makes every get() depend on the single revision counter
  return rendererStateStore?.get(path);
}
```

The consequence: `uiState.get('disclosure.expanded')` and `uiState.get('work.title')`
both depend on `rendererStateRevision`. When either changes, every component that reads
any state re-evaluates. The normalization metadata (`_reactivity`, `_staticFields`,
`dependenciesOf`) built in `normalize.ts` is completely ignored by the renderer.

This is not a Svelte limitation — it is an implementation gap.

### Secondary problems

**`ScreenHost` owns too much.** Loading, error handling, state lifecycle, action
dispatch, slot layout, and route tabs are all in one 300-line component. This makes
all of it harder to test and optimize independently.

**The renderer is not integrated with SvelteKit's load pipeline.** The screen resolves
after mount, which means `preloadData` on hover never warms plugin screens. The
consequence is the visible loading state that feels worse than native SvelteKit pages.

**`ensurePluginsActivated` is called redundantly.** It is called on every navigation,
from sidebar rendering, from command palette, and includes a `fetch('/plugins/registry.json',
{ cache: 'no-store' })`. This is a bug, not a design tradeoff.

---

## Target architecture

```
SvelteKit route (+page.ts)
  load()
    → ensurePluginsActivated() — once, cached
    → runtime.resolveScreen()  — returns ResolvedScreen
    → passes to +page.svelte via data

+page.svelte
  → PluginScreenHost (thin, only layout and slot mapping)
      → PluginScreenBody (CoreUiRenderer entry point)
      → SlotRenderer (for header.actions, sidebar.primary, etc.)

CoreUiRenderer
  → reads from per-path Svelte stores (not global revision counter)
  → delegates to node components

Per-path reactive stores (new)
  → one $derived or writable per binding path accessed
  → Svelte tracks only the stores a component actually reads
  → changing 'disclosure.expanded' does not re-render 'work.title'
```

---

## Part 1 — Per-path reactive state

This is the highest-priority fix. Everything else builds on it.

### 1.1  The new state adapter

The current `RendererStateStore` interface in `context.ts` is structurally sound but
the implementation in `ScreenHost` collapses all reactivity to one counter. The fix is
to expose per-path Svelte stores.

```typescript
// src/lib/unode-web-renderer/state.ts

import { writable, derived, get, type Readable } from 'svelte/store';
import type { StateStore } from '$lib/unode/core/runtime';
import type { JsonValue, Primitive } from '$lib/unode/core/ast';

/**
 * Wraps a unode StateStore in per-path Svelte stores.
 *
 * Instead of a single revision counter, each accessed path gets its own
 * Svelte store. A component that reads 'disclosure.expanded' via
 * getPathStore('disclosure.expanded') only re-renders when that specific
 * path changes — not when any other path changes.
 *
 * This matches what normalize.ts already computes: _reactivity and
 * _staticFields tell us which paths each node depends on. The renderer
 * can use that metadata to subscribe only to relevant paths.
 */
export class SvelteStateAdapter {
  private readonly pathStores = new Map<string, ReturnType<typeof writable<unknown>>>();
  private readonly store: StateStore;

  constructor(store: StateStore) {
    this.store = store;
  }

  /**
   * Returns a Svelte store for a specific state path.
   * Creates the store on first access. Subsequent calls for the same path
   * return the same store instance.
   *
   * The store is backed by a subscription to the underlying StateStore.
   * When StateStore notifies a change at this path, the Svelte store
   * updates, which triggers only the components that read this specific path.
   */
  getPathStore(path: string): Readable<unknown> {
    const existing = this.pathStores.get(path);
    if (existing) return existing;

    const svStore = writable<unknown>(this.store.get(path));

    // Subscribe to this specific path in the unode StateStore
    const unsub = this.store.subscribe(path, (value) => {
      svStore.set(value);
    });

    // Store cleanup alongside the writable
    // In practice, cleanup happens when the screen unmounts (see teardown())
    this.pathStores.set(path, svStore);
    return svStore;
  }

  /**
   * Reactive read for a primitive path.
   * Returns a derived store that resolves to a Primitive.
   * Components use this in $derived runes.
   */
  getPrimitiveStore(path: string, fallback: Primitive): Readable<Primitive> {
    return derived(this.getPathStore(path), (value) => {
      if (value === undefined || value === null) return fallback;
      if (typeof value === 'string' || typeof value === 'number' || typeof value === 'boolean') {
        return value;
      }
      return fallback;
    });
  }

  /** Non-reactive read — for use in event handlers, not in $derived. */
  get(path: string): unknown {
    return this.store.get(path);
  }

  getPrimitive(path: string, fallback: Primitive): Primitive {
    return this.store.getPrimitive(path, fallback);
  }

  set(path: string, value: JsonValue): void {
    this.store.set(path, value);
  }

  toggle(path: string): void {
    this.store.set(path, !this.store.get(path));
  }

  ensure(path: string, fallback: JsonValue): void {
    if (this.store.get(path) === undefined) {
      this.store.set(path, fallback);
    }
  }

  snapshot(): Record<string, unknown> {
    return this.store.snapshot();
  }

  teardown(): void {
    // All Svelte store subscriptions to StateStore are cleaned up here
    // This is called when the screen unmounts
    this.pathStores.clear();
    this.store.reset();
  }
}
```

### 1.2  Updated context contract

```typescript
// src/lib/unode-web-renderer/context.ts

import { getContext, setContext } from 'svelte';
import type { SvelteStateAdapter } from './state';
import type { ActionRef } from '$lib/unode/core/ast';
import type { RendererConfig } from '$lib/unode/core/renderer/config';
import { defaultRendererConfig } from '$lib/unode/core/renderer/config';

export const ACTION_RUNNER_CTX = Symbol('unode:action-runner');
export const STATE_ADAPTER_CTX = Symbol('unode:state-adapter');
export const RENDERER_CONFIG_CTX = Symbol('unode:renderer-config');

export type ActionRunner = (action: ActionRef) => void | Promise<void>;

export function setActionRunner(runner: ActionRunner): void {
  setContext(ACTION_RUNNER_CTX, runner);
}

export function getActionRunner(): ActionRunner {
  return getContext<ActionRunner>(ACTION_RUNNER_CTX);
}

export function setStateAdapter(adapter: SvelteStateAdapter): void {
  setContext(STATE_ADAPTER_CTX, adapter);
}

export function getStateAdapter(): SvelteStateAdapter {
  return getContext<SvelteStateAdapter>(STATE_ADAPTER_CTX);
}

export function setRendererConfig(config: RendererConfig): void {
  setContext(RENDERER_CONFIG_CTX, config);
}

export function getRendererConfig(): RendererConfig {
  return getContext<RendererConfig>(RENDERER_CONFIG_CTX) ?? defaultRendererConfig;
}
```

### 1.3  How components use per-path stores

The pattern changes from reading through a revision-tracked accessor to subscribing
directly to a path-specific store. The Svelte 5 rune `$derived` handles this naturally.

```svelte
<!-- Before: any state change re-runs this -->
<script lang="ts">
  const uiState = getRendererStateStore();
  const expanded = $derived(Boolean(uiState.get(node.binding)));
</script>

<!-- After: only changes to node.binding re-run this -->
<script lang="ts">
  const adapter = getStateAdapter();
  const expandedStore = adapter.getPathStore(node.binding);
  const expanded = $derived(Boolean($expandedStore));
</script>
```

For leaf nodes that render static content (`_reactivity === 'static'`), no store
subscription is needed at all. The value is read once at mount time:

```svelte
<!-- Static text node: no reactive subscription -->
<script lang="ts">
  // node._reactivity === 'static' — content is a plain string after normalization
  // No store needed. Svelte never re-renders this unless the node itself changes.
</script>
<p class={...}>{node.content}</p>
```

For reactive nodes, only the specific paths from `node._staticFields` and the binding
expressions are subscribed:

```svelte
<!-- Reactive text node: subscribes only to the binding path it reads -->
<script lang="ts">
  const adapter = getStateAdapter();
  // node.content = { kind: 'binding', path: 'work.title' }
  // normalize.ts has already identified this — _reactivity === 'reactive'
  const contentStore = adapter.getPathStore((node.content as UiExpr).path);
  const content = $derived(String($contentStore ?? ''));
</script>
<p class={...}>{content}</p>
```

---

## Part 2 — SvelteKit load integration

This is the highest-impact UX fix. Moving screen resolution into `+page.ts` makes
`preloadData` work for plugin screens, which can make navigation feel nearly instant.

### 2.1  Route file changes

```typescript
// src/routes/app/(shell)/[...pluginPath]/+page.ts

import type { PageLoad } from './$types';
import { ensurePluginsActivated, getPluginRuntime } from '$lib/plugins-bridge/runtimeInstance';
import { createHostApi } from '$lib/plugins-bridge/hostApi';

export const load: PageLoad = async ({ url, parent, fetch: kitFetch }) => {
  // ensurePluginsActivated now has session-level caching (see Part 3)
  // On preload hover, this is effectively free after the first navigation
  const { goto } = await parent();
  const runtime = getPluginRuntime();
  const host = createHostApi({ goto, page: { url }, runtime });

  await ensurePluginsActivated(host);

  const query = url.searchParams;
  const resolved = await runtime.resolveScreen(url.pathname, query, host);

  return {
    resolvedScreen: resolved,
    // hostApi is not serializable — it is rebuilt in the component from the resolved data
    // Only plain serializable data travels through the load boundary
  };
};
```

```svelte
<!-- src/routes/app/(shell)/[...pluginPath]/+page.svelte -->
<script lang="ts">
  import type { PageData } from './$types';
  import PluginScreenHost from '$lib/unode-web-renderer/PluginScreenHost.svelte';

  let { data }: { data: PageData } = $props();
</script>

{#if data.resolvedScreen}
  <PluginScreenHost screen={data.resolvedScreen} />
{:else}
  <div>Screen not found.</div>
{/if}
```

The key consequence: on hover over a plugin link, SvelteKit calls `preloadData`, which
runs `load()`, which runs `resolveScreen()`, which calls the plugin's `load()` and
`render()`. By the time the user clicks, the screen is already resolved. The plugin
screen appears at the same speed as any native SvelteKit page.

### 2.2  Stale-while-revalidate for list screens

For screens like browse/list where showing stale content while refreshing is acceptable:

```typescript
// In +page.ts, for applicable routes
export const load: PageLoad = async ({ url, parent }) => {
  // ... same as above

  // If we have a cached screen for this route, return it immediately
  // and trigger a background refresh
  const cached = screenCache.get(url.pathname + url.search);
  if (cached) {
    // Trigger background refresh without awaiting
    runtime.resolveScreen(url.pathname, url.searchParams, host)
      .then(fresh => screenCache.set(url.pathname + url.search, fresh))
      .catch(() => {});
    return { resolvedScreen: cached };
  }

  const resolved = await runtime.resolveScreen(url.pathname, url.searchParams, host);
  if (resolved) screenCache.set(url.pathname + url.search, resolved);
  return { resolvedScreen: resolved };
};
```

---

## Part 3 — Plugin activation caching

This fixes the `fetch('/plugins/registry.json', { cache: 'no-store' })` on every
navigation bug. The fix is simple but the impact is large.

```typescript
// src/lib/plugins-bridge/runtimeInstance.ts — revised activation

let activationPromise: Promise<void> | null = null;
let activated = false;

/**
 * Ensures plugins are activated at most once per session.
 * Subsequent calls return the same promise (or resolve immediately if already done).
 *
 * In development, call invalidateActivation() to force re-activation.
 */
export async function ensurePluginsActivated(host: MugenHostApi): Promise<void> {
  if (activated) return;
  if (activationPromise) return activationPromise;

  activationPromise = performActivation(host)
    .then(() => { activated = true; })
    .catch((err) => {
      activationPromise = null; // allow retry on failure
      throw err;
    });

  return activationPromise;
}

export function invalidateActivation(): void {
  activated = false;
  activationPromise = null;
}
```

The `loadRuntimePluginUrls` call and its `fetch('/plugins/registry.json')` now happen
exactly once per session, not on every sidebar render, command palette open, or
navigation.

---

## Part 4 — PluginScreenHost (decomposed ScreenHost)

The current `ScreenHost` mixes too many concerns. The target splits them:

```
PluginScreenHost.svelte    — receives ResolvedScreen, owns state lifecycle and action runner
  PluginScreenLayout.svelte — slots, route tabs, sidebar layout
    PluginScreenBody.svelte  — CoreUiRenderer entry for the main body
    SlotRenderer.svelte      — unchanged
```

### 4.1  PluginScreenHost

```svelte
<!-- src/lib/unode-web-renderer/PluginScreenHost.svelte -->
<script lang="ts">
  import { onDestroy } from 'svelte';
  import { goto } from '$app/navigation';
  import { page } from '$app/state';
  import { SvelteStateAdapter } from './state';
  import { setActionRunner, setStateAdapter } from './context';
  import PluginScreenLayout from './PluginScreenLayout.svelte';
  import type { ResolvedScreen } from '$lib/unode/api/contracts';
  import type { ActionRef, JsonValue } from '$lib/unode/core/ast';
  import { getPluginRuntime } from '$lib/plugins-bridge/runtimeInstance';
  import { createHostApi } from '$lib/plugins-bridge/hostApi';

  let { screen }: { screen: ResolvedScreen } = $props();

  const runtime = getPluginRuntime();
  const host = createHostApi({ goto, page, runtime });

  // State adapter: one per mounted screen
  // Uses per-path Svelte stores — no global revision counter
  const adapter = new SvelteStateAdapter(screen.state);
  setStateAdapter(adapter);

  // Action runner: owns the built-in action types and delegates to the registry
  setActionRunner(async (action: ActionRef) => {
    if (action.type === 'unode.navigate') {
      const to = typeof action.params?.to === 'string' ? action.params.to : '';
      if (!to) return;
      const url = new URL(to, page.url);
      const query = action.params?.query;
      if (query && typeof query === 'object' && !Array.isArray(query)) {
        for (const [key, value] of Object.entries(query)) {
          if (value === null || value === undefined || value === false || value === '') {
            url.searchParams.delete(key);
          } else {
            url.searchParams.set(key, String(value));
          }
        }
      }
      void goto(`${url.pathname}${url.search}${url.hash}`, {
        replaceState: action.params?.mode === 'replace'
      });
      return;
    }

    if (action.type === 'unode.setState') {
      const path = typeof action.params?.path === 'string' ? action.params.path : '';
      if (!path) return;
      adapter.set(path, (action.params?.value ?? null) as JsonValue);
      return;
    }

    // Delegate to plugin action handlers
    await runtime.registries.actions.run(action, {
      screen,
      entity: screen.entity,
      route: runtime.registries.routes.resolveRouteInfo(page.url.pathname) ?? undefined,
      host
    });
  });

  // Screen refresh subscription
  const refreshUnsub = host.events.on('screen.refresh', (event) => {
    if (event.pathname && event.pathname !== page.url.pathname) return;
    if (event.screenKind && event.screenKind !== screen.screenKind) return;
    // Trigger SvelteKit navigation to re-run load()
    void goto(page.url.href, { invalidateAll: true });
  });

  onDestroy(() => {
    refreshUnsub();
    adapter.teardown();
  });
</script>

<PluginScreenLayout {screen} />
```

### 4.2  PluginScreenLayout

```svelte
<!-- src/lib/unode-web-renderer/PluginScreenLayout.svelte -->
<script lang="ts">
  import type { ResolvedScreen } from '$lib/unode/api/contracts';
  import { readRouteTabsMeta } from './screen-chrome/route-tabs';
  import RouteTabsLayout from '$lib/shared/ui/RouteTabsLayout.svelte';
  import CoreUiRenderer from './CoreUiRenderer.svelte';
  import SlotRenderer from './SlotRenderer.svelte';
  import { goto } from '$app/navigation';
  import type { Attachment } from 'svelte/attachments';

  let { screen }: { screen: ResolvedScreen } = $props();

  const headerActions = $derived(screen.slots['header.actions'] ?? []);
  const mainBefore = $derived(screen.slots['main.before'] ?? []);
  const mainAfter = $derived(screen.slots['main.after'] ?? []);
  const sidebarPrimary = $derived(screen.slots['sidebar.primary'] ?? []);
  const sidebarSecondary = $derived(screen.slots['sidebar.secondary'] ?? []);
  const routeTabs = $derived(readRouteTabsMeta(screen.meta));
  const routeTabItems = $derived(
    routeTabs?.tabs.map(tab => ({ id: tab.id, label: tab.label, badge: tab.badge, path: tab.to })) ?? []
  );
  const hasSidebar = $derived(sidebarPrimary.length > 0 || sidebarSecondary.length > 0);

  function focusInitialTarget(id: string | undefined): Attachment {
    return () => {
      if (!id || typeof document === 'undefined') return;
      document.getElementById(id)?.focus();
    };
  }

  function handleRouteTabChange(tabId: string) {
    const next = routeTabs?.tabs.find(t => t.id === tabId);
    if (next) void goto(next.to);
  }
</script>

<section class="space-y-[var(--gap-4)]" {@attach focusInitialTarget(screen.body.initialFocus)}>
  {#if headerActions.length}
    <div class="flex items-center justify-end gap-[var(--gap-2)]">
      <SlotRenderer nodes={headerActions} />
    </div>
  {/if}

  {#snippet screenContent()}
    <div class={['grid gap-[var(--gap-6)]', hasSidebar && 'lg:grid-cols-[minmax(0,1fr)_var(--context-sidebar-w)]']}>
      <div class="space-y-[var(--gap-4)]">
        <SlotRenderer nodes={mainBefore} />
        <CoreUiRenderer node={screen.body} />
        <SlotRenderer nodes={mainAfter} />
      </div>
      {#if hasSidebar}
        <aside class="space-y-[var(--gap-4)]">
          <SlotRenderer nodes={sidebarPrimary} />
          <SlotRenderer nodes={sidebarSecondary} />
        </aside>
      {/if}
    </div>
  {/snippet}

  {#if routeTabs && routeTabItems.length > 1}
    <RouteTabsLayout
      tabs={routeTabItems}
      active={routeTabs.active}
      swipeEnabled={routeTabs.swipeEnabled ?? true}
      swipeThreshold={routeTabs.swipeThreshold ?? 60}
      onNavigate={handleRouteTabChange}
    >
      {@render screenContent()}
    </RouteTabsLayout>
  {:else}
    {@render screenContent()}
  {/if}
</section>
```

---

## Part 5 — CoreUiRenderer with reactivity granularity

The current `CoreUiRenderer` is structurally correct but passes all state resolution
through the global accessor. The changes here are surgical — the structure stays, the
state access mechanism changes.

### 5.1  Static vs reactive nodes

The normalized AST carries `_reactivity` and `_staticFields`. Use them:

```svelte
<!-- Inside CoreUiRenderer — text node, before -->
{:else if node.kind === 'text'}
  <p class={textRoleClass(node.role, node.tone, node.emphasis)}>
    {resolveStringValue(node.content, uiState)}
  </p>

<!-- After: branch on reactivity -->
{:else if node.kind === 'text'}
  {#if node._reactivity === 'static'}
    <!-- node.content is already a plain string after normalization — no store needed -->
    <p class={textRoleClass(node.role, node.tone, node.emphasis)}
       style={truncateStyle(node.role, node.truncate)}>
      {node._staticFields.content ?? node.content}
    </p>
  {:else}
    <!-- node.content is a binding — subscribe to the specific path -->
    <ReactiveText {node} />
  {/if}
```

```svelte
<!-- ReactiveText.svelte — handles only reactive text nodes -->
<script lang="ts">
  import type { CanonicalNode } from '$lib/unode/core/normalize';
  import type { TextNode, UiExpr } from '$lib/unode/core/ast';
  import { getStateAdapter } from '../context';
  import { derived } from 'svelte/store';

  let { node }: { node: CanonicalNode<TextNode> } = $props();

  const adapter = getStateAdapter();

  // Extract the binding path from the expression
  const bindingPath = typeof node.content === 'object' && node.content.kind === 'binding'
    ? node.content.path
    : null;

  // Subscribe only to the specific path this node reads
  const contentStore = bindingPath
    ? adapter.getPathStore(bindingPath)
    : null;

  const content = $derived(
    contentStore
      ? String($contentStore ?? '')
      : typeof node.content === 'string'
        ? node.content
        : ''
  );
</script>

<p class={textRoleClass(node.role, node.tone, node.emphasis)}>
  {content}
</p>
```

The pattern scales to all leaf nodes. Static nodes read from `_staticFields` without
any store subscription. Reactive nodes subscribe only to the paths they read.

### 5.2  ConditionalNode with granular condition tracking

```svelte
<!-- Before: any state change re-evaluates condition -->
{:else if node.kind === 'conditional'}
  {#if resolveBooleanValue(node.condition, uiState)}
    <CoreChildren nodes={[node.then]} />
  {:else if node.else}
    <CoreChildren nodes={[node.else]} />
  {/if}

<!-- After: only the condition binding re-evaluates -->
{:else if node.kind === 'conditional'}
  <ConditionalRenderer {node} />
```

```svelte
<!-- ConditionalRenderer.svelte -->
<script lang="ts">
  import type { CanonicalNode } from '$lib/unode/core/normalize';
  import type { ConditionalNode, UiExpr } from '$lib/unode/core/ast';
  import { getStateAdapter } from '../context';
  import CoreChildren from '../CoreChildren.svelte';

  let { node }: { node: CanonicalNode<ConditionalNode> } = $props();

  const adapter = getStateAdapter();

  const conditionPath = typeof node.condition === 'object' && node.condition.kind === 'binding'
    ? node.condition.path
    : null;

  const conditionStore = conditionPath ? adapter.getPathStore(conditionPath) : null;

  const active = $derived(
    conditionStore
      ? Boolean($conditionStore)
      : typeof node.condition === 'boolean'
        ? node.condition
        : false
  );
</script>

{#if active}
  <CoreChildren nodes={[node.then]} />
{:else if node.else}
  <CoreChildren nodes={[node.else]} />
{/if}
```

---

## Part 6 — AbortSignal for load cancellation

Currently, rapid navigation leaves background fetches running. The result is wasted
CPU and network — the results are discarded, but the work happens.

```typescript
// In +page.ts — add AbortController support
export const load: PageLoad = async ({ url, parent, fetch: kitFetch }) => {
  const controller = new AbortController();
  // SvelteKit calls the load cancel() automatically on navigation away
  // but we need to thread the signal into the plugin's load() call

  const resolved = await runtime.resolveScreen(
    url.pathname,
    url.searchParams,
    host,
    { signal: controller.signal }  // ← thread AbortSignal through
  );

  return { resolvedScreen: resolved };
};
```

```typescript
// In PluginRoute.load() — plugin authors opt-in to cancellation
async load(ctx: PluginRenderContext): Promise<TData> {
  const work = await ctx.api.catalog.getWork(ctx.route.params.workId, {
    signal: ctx.signal  // ← pass through to fetch calls
  });
  return { work };
}
```

---

## Part 7 — Node component alignments

### CoreDisclosureNode — no changes needed

The current implementation is correct. It reads `uiState.get(node.binding)` which
should change to `adapter.getPathStore(node.binding)`:

```svelte
<!-- Current -->
const expanded = $derived(Boolean(uiState.get(node.binding)));

<!-- Target -->
const adapter = getStateAdapter();
const expandedStore = adapter.getPathStore(node.binding);
const expanded = $derived(Boolean($expandedStore));
```

The toggle action stays the same:
```svelte
function toggle() { adapter.toggle(node.binding); }
```

The `aria-expanded` and `aria-controls` attributes are already correct. The chevron
rotation animation is correct. The conditional rendering of children is correct.
The `$props.id()` for `contentId` is the right approach.

### CoreGridNode — minimal changes

The responsive column logic via `window.innerWidth` + `ResizeObserver` is correct and
necessary. The `IntersectionObserver` for auto-loading continuation is correct.

The only change is the state accessor for incremental continuation:

```svelte
<!-- Current -->
const visibleCount = $derived(
  incremental
    ? Math.max(incremental.initial, Math.min(total, Number(uiState.getPrimitive(incremental.binding, incremental.initial))))
    : total
);

<!-- Target -->
const visibleCountStore = incremental
  ? adapter.getPrimitiveStore(incremental.binding, incremental.initial)
  : null;

const visibleCount = $derived(
  incremental && visibleCountStore
    ? Math.max(incremental.initial, Math.min(total, Number($visibleCountStore)))
    : total
);
```

The keyboard navigation via `navigableContainer` and `navItem` stays unchanged — it is
already correctly implemented.

### CoreMenuNode — note on positioning

The current `CoreMenuNode` likely uses absolute positioning for the dropdown. This
should use the Popover API (`popover` attribute) where available, falling back to
absolute positioning. This avoids z-index stacking issues without needing a portal.

---

## Part 8 — Multithreading note

You asked whether the Svelte renderer can be multithreaded. The answer is nuanced.

**The rendering itself is single-threaded and must stay that way.** DOM access is not
thread-safe and cannot be done from a Web Worker. Svelte 5 runs its reactive system
synchronously on the main thread. This is not a limitation of Svelte — it is a
constraint of the DOM model.

**What CAN run off the main thread:**

- Plugin `load()` — the async data fetching. If a plugin fetches data from the network,
  that I/O is already non-blocking (browser fetch is async by nature).
- Heavy data transformations in `render()` — if a plugin does expensive computation in
  `render()`, that could run in a Worker and return a serialized `CanonicalScreen`.
  The renderer receives the screen and mounts it on the main thread.
- `normalizeScreen()` — the canonicalization pass. For large screens with hundreds of
  nodes, this could move to a Worker. For typical plugin screens (tens to low hundreds
  of nodes), it is fast enough on the main thread (<1ms).

**The practical architecture for Worker-based plugins:**

```
Main thread
  → PostMessage to plugin Worker: { type: "route", pattern, params, query }
Plugin Worker
  → Runs load() and render()
  → PostMessage back: { type: "screen", screen: CanonicalScreen }
Main thread
  → Receives CanonicalScreen (already JSON-safe by design)
  → normalizeScreen(screen) — fast, on main thread
  → Svelte mounts the CanonicalScreen
```

This is identical to the Deno TUI model — the plugin isolation via Worker is
already designed into the unode contract. The `CanonicalScreen` being JSON-serializable
is what makes this possible.

For the current implementation, this Worker isolation is optional. Plugins can run on
the main thread (current behavior) or in a Worker (future capability for untrusted
third-party plugins). The unode AST design makes both work without changes to the
plugin source code.

---

## Implementation order

**Phase 1 — Fix plugin activation caching** (half a day)
Cache `ensurePluginsActivated` in session memory. This is a one-file change with
immediate measurable impact on navigation speed.

**Phase 2 — Move screen resolution to `+page.ts`** (1–2 days)
Create `+page.ts` for the plugin route. Pass `ResolvedScreen` through `data`.
Simplify `+page.svelte` to just mount `PluginScreenHost`. This enables `preloadData`
on hover for all plugin screens.

**Phase 3 — Per-path state adapter** (2 days)
Implement `SvelteStateAdapter`. Update `context.ts`. Update `PluginScreenHost` to use
it. Update `CoreDisclosureNode` and `CoreGridNode` as the two components with the most
direct state binding needs. Verify that changing disclosure state does not cause unrelated
nodes to re-render (use Svelte devtools or `console.count` to confirm).

**Phase 4 — Decompose ScreenHost** (1 day)
Split into `PluginScreenHost`, `PluginScreenLayout`. Move `screen-chrome/route-tabs` to
the renderer layer. Remove the `rendererStateRevision` counter.

**Phase 5 — Static/reactive split in CoreUiRenderer** (2 days)
Add the `_reactivity` branch to `CoreUiRenderer`. Extract `ReactiveText`,
`ConditionalRenderer`, and equivalent small components for the other reactive leaf
nodes. Verify with Svelte devtools that static nodes do not re-render on state change.

**Phase 6 — AbortSignal** (1 day)
Thread `AbortSignal` from `+page.ts` through `resolveScreen` into plugin `load()`.
Test with rapid navigation between plugin routes.

**Phase 7 — Consolidate renderer location** (1 day)
Move all renderer files from `widgets/app-plugin-renderer` and `widgets/app-plugin-shell`
into `src/lib/unode-web-renderer/`. Update imports. Remove `plugins-bridge/rendererConfig.ts`
wrapper.