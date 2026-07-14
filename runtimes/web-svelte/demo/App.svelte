<script lang="ts">
  import { onMount } from "svelte";
  import * as webHostModule from "../pkg/unode_web_host.js";
  import webHostWasmUrl from "../pkg/unode_web_host_bg.wasm?url";
  import {
    HostSession,
    PluginInstance,
    ScreenStore,
    StateWriteSink,
    UnodeScreen,
    WebRuntime,
  } from "../src";
  import pluginWasmUrl from "./web_counter_plugin.wasm?url";

  const PLUGIN_ROUTE_PATTERN = "/plugins/web-counter";

  let store: ScreenStore | null = $state(null);
  let runtime: WebRuntime | null = $state(null);
  let error: string | null = $state(null);

  function routeForCurrentLocation() {
    const url = new URL(window.location.href);

    if (url.pathname === "/") {
      window.history.replaceState(null, "", `${PLUGIN_ROUTE_PATTERN}${url.search}${url.hash}`);
      url.pathname = PLUGIN_ROUTE_PATTERN;
    }

    return {
      pattern: PLUGIN_ROUTE_PATTERN,
      params: {},
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
        const plugin = await PluginInstance.instantiate(await fetch(pluginWasmUrl), sink.handler);
        const nextRuntime = new WebRuntime({
          plugin,
          session,
          sink,
          route: routeForCurrentLocation(),
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
  <UnodeScreen {store} onAction={runtime.onAction} />
{:else}
  <p>Loading unode runtime...</p>
{/if}
