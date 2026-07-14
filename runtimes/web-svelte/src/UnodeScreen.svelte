<script lang="ts">
  import { literalOf, nodeKey } from "./ir";
  import type { OnAction } from "./renderer";
  import type { ScreenStore } from "./store";
  import UnodeNode from "./UnodeNode.svelte";

  interface Props {
    store: ScreenStore;
    onAction: OnAction;
  }

  let { store, onAction }: Props = $props();
  let title = $derived(literalOf(store.screen.p.title));
</script>

<section class="unode-screen">
  {#if title != null}
    <h1 class="unode-title">{String(title)}</h1>
  {/if}

  {#each store.screen.c ?? [] as node (nodeKey(node))}
    <UnodeNode {node} {store} {onAction} />
  {/each}
</section>
