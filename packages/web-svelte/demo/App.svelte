<script lang="ts">
  import { onMount } from "svelte";
  import * as webHostModule from "../pkg/unode_web_host.js";
  import webHostWasmUrl from "../pkg/unode_web_host_bg.wasm?url";
  import {
    defineRenderer,
    h,
    HostSession,
    hostSlot,
    ScreenStore,
    StateWriteSink,
    UnodeScreen,
    WebPluginRegistry,
    WebRuntime,
  } from "../src";
  import Button from "./Button.svelte";
  import pluginWasmUrl from "./web_counter_plugin.wasm?url";

  // Recipes written once in the universal TS language: `action` nodes render as
  // the host's native <Button> via a host slot; the rest use built-in recipes.
  const renderer = defineRenderer()
    .recipe("action", ({ label, prop, action }) =>
      hostSlot("Button", { children: label, intent: prop("intent"), action }),
    )
    .recipe("section", ({ title, children }) =>
      h("section", { class: "ds-card" }, title ? h("h2", {}, title) : null, children),
    )
    .build();

  const PLUGIN_ROUTE_PATTERN = "/plugins/web-counter";
  const pluginRegistry = new WebPluginRegistry().register({
    id: "dev.unode.web-counter",
    routePattern: PLUGIN_ROUTE_PATTERN,
    loadWasm: () => fetch(pluginWasmUrl),
  });

  let store: ScreenStore | null = $state(null);
  let runtime: WebRuntime | null = $state(null);
  let error: string | null = $state(null);

  function routeTargetForCurrentLocation() {
    const url = new URL(window.location.href);

    if (url.pathname === "/") {
      window.history.replaceState(null, "", `${PLUGIN_ROUTE_PATTERN}${url.search}${url.hash}`);
      url.pathname = PLUGIN_ROUTE_PATTERN;
    }

    return {
      pathname: url.pathname,
      query: Object.fromEntries(url.searchParams.entries()),
    };
  }

  onMount(() => {
    let cancelled = false;

    (async () => {
      try {
        const session = await HostSession.create(
          webHostModule as never,
          webHostWasmUrl,
          "en",
        );
        const sink = new StateWriteSink();
        const pluginTarget = routeTargetForCurrentLocation();
        const { plugin, route } = await pluginRegistry.instantiateForPath(
          pluginTarget.pathname,
          pluginTarget.query,
          sink.handler,
        );
        const nextRuntime = new WebRuntime({
          plugin,
          session,
          sink,
          route,
          locale: "en",
        });
        const nextStore = nextRuntime.mount();

        if (!cancelled) {
          runtime = nextRuntime;
          store = nextStore;
        }
      } catch (e) {
        if (!cancelled) error = String(e);
      }
    })();

    return () => {
      cancelled = true;
    };
  });
</script>

{#if error}
  <pre class="unode-error">{error}</pre>
{:else if store && runtime}
  <UnodeScreen {store} onAction={runtime.onAction} {renderer} components={{ Button }} />
{:else}
  <p>Loading unode runtime...</p>
{/if}
