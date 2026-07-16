<script lang="ts">
  import { onMount } from "svelte";
  import {
    defineRenderer,
    h,
    hostSlot,
    ScreenStore,
    UnodeScreen,
  } from "unode-svelte";
  import type { WebRuntime } from "unode-web-core";
  import Button from "./Button.svelte";
  import { bootRuntime } from "./runtime";

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

  let store: ScreenStore | null = $state(null);
  let runtime: WebRuntime | null = $state(null);
  let error: string | null = $state(null);

  onMount(() => {
    let cancelled = false;

    (async () => {
      try {
        const nextRuntime = await bootRuntime();
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
