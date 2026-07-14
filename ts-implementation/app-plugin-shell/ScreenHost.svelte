<script lang="ts">
  import { page } from '$app/state';
  import { afterNavigate, goto } from '$app/navigation';
  import { onMount } from 'svelte';
  import type { Attachment } from 'svelte/attachments';
  import RouteTabsLayout from '$lib/shared/ui/RouteTabsLayout.svelte';
  import { readRouteTabsMeta } from '$lib/plugins-bridge/screen-chrome/route-tabs';
  import { setActionRunner, setRendererStateStore } from '$lib/widgets/app-plugin-renderer/context';
  import CoreUiRenderer from '$lib/widgets/app-plugin-renderer/CoreUiRenderer.svelte';
  import SlotRenderer from './SlotRenderer.svelte';
  import { createHostApi } from '$lib/plugins-bridge/hostApi';
  import { ensurePluginsActivated, getPluginRuntime } from '$lib/plugins-bridge/runtimeInstance';
  import type { ResolvedScreen, ResolvedRouteInfo } from '$lib/unode/api/contracts';
  import type { ActionRef as CoreActionRef, JsonValue, Primitive } from '$lib/unode/core/ast';
  import type { StateStore } from '$lib/unode/core/runtime';
  import type { MugenHostApi } from '$lib/plugins-bridge/host';

  let screen = $state<ResolvedScreen | null>(null);
  let routeInfo = $state<ResolvedRouteInfo | null>(null);
  let error = $state<string | null>(null);
  let loading = $state<boolean>(true);

  let runtime = $state(getPluginRuntime());
  let hostApi = $state<MugenHostApi | null>(null);
  let requestKey = 0;
  let uiStateKey = $state('');
  let refreshUnsubscribe: (() => void) | null = null;
  let rendererStateRevision = $state(0);
  let rendererStateStore: StateStore | null = null;
  let rendererStateUnsubscribe: (() => void) | null = null;

  const pathname = $derived(page.url.pathname);
  const search = $derived(page.url.search);

  function isRecord(value: unknown): value is Record<string, unknown> {
    return value !== null && typeof value === 'object' && !Array.isArray(value);
  }

  function mergeMissingState(
    nextState: StateStore,
    snapshot: Record<string, unknown>,
    pathPrefix = ''
  ) {
    for (const [key, value] of Object.entries(snapshot)) {
      const path = pathPrefix ? `${pathPrefix}.${key}` : key;
      const current = nextState.get(path);

      if (current === undefined) {
        nextState.set(path, value as JsonValue);
        continue;
      }

      if (isRecord(value) && isRecord(current)) {
        mergeMissingState(nextState, value, path);
      }
    }
  }

  function attachRendererState(nextState: StateStore, preserveState: boolean) {
    const previousSnapshot = preserveState ? rendererStateStore?.snapshot() : undefined;

    rendererStateUnsubscribe?.();
    rendererStateStore = nextState;

    if (previousSnapshot) {
      mergeMissingState(nextState, previousSnapshot);
    }

    rendererStateUnsubscribe = nextState.subscribePrefix('', () => {
      rendererStateRevision += 1;
    });
    rendererStateRevision += 1;
  }

  function clearRendererState() {
    rendererStateUnsubscribe?.();
    rendererStateUnsubscribe = null;
    rendererStateStore = null;
    rendererStateRevision += 1;
  }

  const rendererState = {
    current: () => rendererStateStore,
    get(path: string) {
      void rendererStateRevision;
      return rendererStateStore?.get(path);
    },
    getPrimitive(path: string, fallback: Primitive) {
      void rendererStateRevision;
      return rendererStateStore?.getPrimitive(path, fallback) ?? fallback;
    },
    set(path: string, value: JsonValue) {
      rendererStateStore?.set(path, value);
    },
    toggle(path: string) {
      if (!rendererStateStore) return;
      rendererStateStore.set(path, !rendererStateStore.get(path));
    },
    ensure(path: string, fallback: JsonValue) {
      if (!rendererStateStore || rendererStateStore.get(path) !== undefined) return;
      rendererStateStore.set(path, fallback);
    }
  };

  setRendererStateStore(rendererState);

  setActionRunner(async (action: CoreActionRef) => {
    if (action.type === 'unode.navigate') {
      const to = typeof action.params?.to === 'string' ? action.params.to : '';
      if (!to) return;
      const url = new URL(to, page.url);
      const query =
        action.params?.query && typeof action.params.query === 'object' && !Array.isArray(action.params.query)
          ? action.params.query
          : undefined;

      if (query) {
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
      rendererStateStore?.set(path, (action.params?.value ?? null) as JsonValue);
      return;
    }

    if (!runtime || !hostApi || !screen) return;
    await runtime.registries.actions.run(action, {
      screen,
      entity: screen.entity,
      route: routeInfo ?? undefined,
      host: hostApi
    });
  });

  async function loadScreen() {
    requestKey += 1;
    const current = requestKey;
    loading = true;
    error = null;

    const query = new URLSearchParams(search);
    const host = createHostApi({ goto, page, runtime });
    hostApi = host;
    if (!refreshUnsubscribe) {
      refreshUnsubscribe = host.events.on('screen.refresh', (event) => {
        if (event.pathname && event.pathname !== page.url.pathname) return;
        if (event.screenKind && event.screenKind !== screen?.screenKind) return;
        void loadScreen();
      });
    }

    try {
      await ensurePluginsActivated(host);
      const resolved = await runtime.resolveScreen(pathname, query, host);
      if (current !== requestKey) return;
      if (!resolved) {
        screen = null;
        clearRendererState();
        error = 'Screen not resolved.';
        return;
      }
      const nextUiKey = `${resolved.screenKind}:${pathname}`;
      const preserveState = uiStateKey === nextUiKey;
      uiStateKey = nextUiKey;
      attachRendererState(resolved.state, preserveState);
      screen = resolved;
      routeInfo = runtime.registries.routes.resolveRouteInfo(pathname);
    } catch (err: unknown) {
      if (current !== requestKey) return;
      error = err instanceof Error ? err.message : 'Failed to resolve screen.';
      screen = null;
      clearRendererState();
    } finally {
      if (current === requestKey) {
        loading = false;
      }
    }
  }

  onMount(() => {
    void loadScreen();
    return () => {
      refreshUnsubscribe?.();
      refreshUnsubscribe = null;
      clearRendererState();
    };
  });

  afterNavigate(() => {
    void loadScreen();
  });

  function focusInitialTarget(initialFocusId: string | undefined): Attachment {
    return () => {
      if (!initialFocusId || typeof document === 'undefined') return;

      const target = document.getElementById(initialFocusId);
      if (!(target instanceof HTMLElement)) return;
      target.focus();
    };
  }

  const headerActions = $derived(screen?.slots['header.actions'] ?? []);
  const mainBefore = $derived(screen?.slots['main.before'] ?? []);
  const mainAfter = $derived(screen?.slots['main.after'] ?? []);
  const sidebarPrimary = $derived(screen?.slots['sidebar.primary'] ?? []);
  const sidebarSecondary = $derived(screen?.slots['sidebar.secondary'] ?? []);
  const routeTabs = $derived(readRouteTabsMeta(screen?.meta));
  const routeTabItems = $derived(
    routeTabs
      ? routeTabs.tabs.map((tab) => ({
          id: tab.id,
          label: tab.label,
          badge: tab.badge,
          path: tab.to
        }))
      : []
  );
  const hasSidebar = $derived(sidebarPrimary.length > 0 || sidebarSecondary.length > 0);
  const shellLayoutClass = $derived([
    'grid gap-[var(--gap-6)]',
    hasSidebar && 'lg:grid-cols-[minmax(0,1fr)_var(--context-sidebar-w)]'
  ]);

  function handleRouteTabChange(tabId: string) {
    if (!routeTabs) return;
    const next = routeTabs.tabs.find((tab) => tab.id === tabId);
    if (!next) return;
    void goto(next.to);
  }
</script>

<section class="space-y-[var(--gap-4)]" {@attach focusInitialTarget(screen?.body.initialFocus)}>
  {#if loading}
    <div class="border-[length:var(--border-w-emphasis)] border-dashed border-[var(--color-border-strong)] bg-[color-mix(in_srgb,var(--color-bg)_70%,transparent)] p-[var(--space-8)] text-center text-[length:var(--fs-xs)] font-black uppercase tracking-widest text-[var(--color-text-muted)]">
      Loading screen...
    </div>
  {/if}

  {#if error}
    <div class="border-[length:var(--border-w-strong)] border-[var(--color-border-strong)] bg-[var(--color-bg)] px-[var(--space-4)] py-[var(--space-3)] text-[length:var(--fs-sm)] font-bold text-[var(--color-danger)]">
      {error}
    </div>
  {/if}

  {#if screen}
    {@const resolvedScreen = screen}

    {#if headerActions.length}
      <div class="flex items-center justify-end gap-[var(--gap-2)]">
        <SlotRenderer nodes={headerActions} />
      </div>
    {/if}

    {#snippet screenContent()}
      <div class={shellLayoutClass}>
        <div class="space-y-[var(--gap-4)]">
          <SlotRenderer nodes={mainBefore} />
          <CoreUiRenderer node={resolvedScreen.body} />
          <SlotRenderer nodes={mainAfter} />
        </div>

        {#if sidebarPrimary.length || sidebarSecondary.length}
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
  {/if}
</section>
