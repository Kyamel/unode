<script lang="ts">
	import type { CanonicalUiNode } from '$lib/unode/core/normalize';
	import CoreUiRenderer from './CoreUiRenderer.svelte';

	let { nodes = [] }: { nodes?: readonly CanonicalUiNode[] } = $props();
</script>

{#each nodes as child (child._key)}
	<CoreUiRenderer node={child} />
{/each}
