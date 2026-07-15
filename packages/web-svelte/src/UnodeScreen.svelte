<script lang="ts">
  import { nodeKey, rendererPropsOf } from "unode-renderer";
  import type { OnAction } from "./renderer";
  import type { ScreenStore } from "unode-renderer";
  import UnodeNode from "./UnodeNode.svelte";

  interface Props {
    store: ScreenStore;
    onAction: OnAction;
  }

  let { store, onAction }: Props = $props();
  let screenProps = $derived(rendererPropsOf(store.screen.p));
  let title = $derived(screenProps.title);
</script>

<section class="unode-screen">
  {#if title != null}
    <h1 class="unode-title">{String(title)}</h1>
  {/if}

  {#each store.screen.c ?? [] as node (nodeKey(node))}
    <UnodeNode {node} {store} {onAction} />
  {/each}
</section>
