<script lang="ts">
	import type { PressableNode } from '$lib/unode/core/ast';
	import type { CanonicalNode, CanonicalUiNode } from '$lib/unode/core/normalize';
	import { getActionRunner, getRendererStateStore } from '$lib/widgets/app-plugin-renderer/context';
	import CoreUiRenderer from '../CoreUiRenderer.svelte';
	import { resolveStringValue } from '../resolve';

	let { node }: { node: CanonicalNode<PressableNode> } = $props();

	const runAction = getActionRunner();
	const uiState = getRendererStateStore();
	const label = $derived(node.label ? resolveStringValue(node.label, uiState) : undefined);

	function handleClick() {
		void runAction(node.action);
	}
</script>

<button
	id={node.id}
	type="button"
	class="group block w-full rounded-[var(--radius-none)] text-left transition-transform hover:-translate-y-[1px] focus-visible:outline focus-visible:outline-[length:var(--border-w-strong)] focus-visible:outline-[var(--color-info)]"
	data-nav-activate="true"
	aria-label={label}
	onclick={handleClick}
>
	<div class="relative">
		<CoreUiRenderer node={node.child as CanonicalUiNode} />
		<span
			aria-hidden="true"
			class="pointer-events-none absolute inset-0 rounded-[var(--radius-none)] border-[length:var(--border-w-strong)] border-[var(--color-info)] opacity-0 shadow-[0_0_0_1px_var(--color-info)] transition-opacity duration-150 group-hover:opacity-100 group-focus-visible:opacity-100"
		></span>
	</div>
</button>
