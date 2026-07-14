<script lang="ts">
  import type { CanonicalUiNode } from '$lib/unode/core/normalize';
  import CoreUiRenderer from '$lib/widgets/app-plugin-renderer/CoreUiRenderer.svelte';

  let { nodes = [] }: { nodes?: CanonicalUiNode[] } = $props();
</script>

{#if nodes.length}
  <div class="space-y-[var(--gap-3)]">
    {#each nodes as node (node._key)}
      <CoreUiRenderer {node} />
    {/each}
  </div>
{/if}
