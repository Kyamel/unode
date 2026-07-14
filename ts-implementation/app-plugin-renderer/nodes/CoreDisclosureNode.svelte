<script lang="ts">
	import type { DisclosureNode } from '$lib/unode/core/ast';
	import type { CanonicalNode, CanonicalUiNode } from '$lib/unode/core/normalize';
	import { getRendererStateStore } from '$lib/widgets/app-plugin-renderer/context';
	import CoreChildren from '../CoreChildren.svelte';
	import { resolveStringValue } from '../resolve';

	let { node }: { node: CanonicalNode<DisclosureNode> } = $props();

	const uiState = getRendererStateStore();
	const contentId = $props.id();

	const expanded = $derived(Boolean(uiState.get(node.binding)));
	const label = $derived(
		resolveStringValue(expanded ? (node.labelExpanded ?? node.label) : node.label, uiState)
	);

	function toggle() {
		uiState.set(node.binding, !expanded);
	}
</script>

<div class="space-y-[var(--gap-3)]">
	<button
		id={node.id}
		type="button"
		class="inline-flex items-center gap-[var(--gap-2)] border-b-[length:var(--border-w-strong)] border-[var(--color-border-strong)] pb-[var(--space-1)] text-left text-[length:var(--fs-sm)] font-black uppercase tracking-[0.12em] text-[var(--color-text)] transition-transform"
		aria-expanded={expanded}
		aria-controls={contentId}
		onclick={toggle}
	>
		<span
			class="inline-flex text-[length:var(--fs-xs)] text-[var(--color-text-muted)] transition-transform"
			style={`transform:rotate(${expanded ? 90 : 0}deg);`}
			aria-hidden="true"
		>
			▶
		</span>
		<span>{label}</span>
	</button>

	{#if expanded}
		<div id={contentId} class="space-y-[var(--gap-3)]">
			<CoreChildren nodes={node.children as readonly CanonicalUiNode[]} />
		</div>
	{/if}
</div>
