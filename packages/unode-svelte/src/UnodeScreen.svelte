<script lang="ts">
  import { defineRenderer, type Renderer, type ScreenStore } from "unode-web-renderer";
  import { createSveltePortal, type HostComponents, type OnAction } from "./renderer.svelte";

  interface Props {
    store: ScreenStore;
    onAction?: OnAction;
    /** A renderer from `defineRenderer()`. Defaults to the built-in recipes. */
    renderer?: Renderer;
    /** Host components that fulfill `hostSlot(name)` holes. */
    components?: HostComponents;
  }

  let { store, onAction, renderer, components = {} }: Props = $props();

  const fallbackRenderer = defineRenderer().build();
  let host: HTMLElement;

  $effect(() => {
    const active = renderer ?? fallbackRenderer;
    const handle = active.mount(host, store, {
      onAction,
      portal: createSveltePortal(components),
    });
    return () => handle.unmount();
  });
</script>

<div class="unode-root" bind:this={host}></div>
