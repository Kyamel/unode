<script lang="ts">
	import { navItem } from '$lib/shared/keyboard/actions';
	import type { ItemNode } from '$lib/unode/core/ast';
	import type { CanonicalNode, CanonicalUiNode } from '$lib/unode/core/normalize';
	import { getActionRunner } from '$lib/widgets/app-plugin-renderer/context';
	import CoreChildren from '../CoreChildren.svelte';

	let {
		node,
		containerId,
		itemId
	}: {
		node: CanonicalNode<ItemNode>;
		containerId?: string;
		itemId?: string;
	} = $props();

	const runAction = getActionRunner();
	const clickable = $derived(Boolean(node.action));

	function handleClick() {
		if (!node.action) return;
		void runAction(node.action);
	}
</script>

{#if clickable}
	<button
		id={node.id}
		type="button"
		class="flex w-full items-start gap-[var(--gap-3)] rounded-[var(--radius-none)] border-[length:var(--border-w-heavy)] border-[var(--color-border-strong)] bg-[var(--color-surface-1)] p-[var(--space-3)] text-left transition-colors hover:bg-[var(--color-surface-2)]"
		use:navItem={containerId ? { containerId, id: itemId } : null}
		onclick={handleClick}
	>
		{#if node.leading?.length}
			<div class="shrink-0">
				<CoreChildren nodes={node.leading as readonly CanonicalUiNode[]} />
			</div>
		{/if}
		<div class="min-w-0 flex-1 space-y-[var(--gap-2)]">
			<div class="space-y-[var(--gap-1)]">
				<CoreChildren nodes={node.primary as readonly CanonicalUiNode[]} />
			</div>
			{#if node.secondary?.length}
				<div class="space-y-[var(--gap-1)]">
					<CoreChildren nodes={node.secondary as readonly CanonicalUiNode[]} />
				</div>
			{/if}
		</div>
		{#if node.trailing?.length}
			<div class="shrink-0">
				<CoreChildren nodes={node.trailing as readonly CanonicalUiNode[]} />
			</div>
		{/if}
	</button>
{:else}
	<div
		id={node.id}
		class="flex w-full items-start gap-[var(--gap-3)] rounded-[var(--radius-none)] border-[length:var(--border-w-heavy)] border-[var(--color-border-strong)] bg-[var(--color-surface-1)] p-[var(--space-3)]"
		use:navItem={containerId ? { containerId, id: itemId } : null}
	>
		{#if node.leading?.length}
			<div class="shrink-0">
				<CoreChildren nodes={node.leading as readonly CanonicalUiNode[]} />
			</div>
		{/if}
		<div class="min-w-0 flex-1 space-y-[var(--gap-2)]">
			<div class="space-y-[var(--gap-1)]">
				<CoreChildren nodes={node.primary as readonly CanonicalUiNode[]} />
			</div>
			{#if node.secondary?.length}
				<div class="space-y-[var(--gap-1)]">
					<CoreChildren nodes={node.secondary as readonly CanonicalUiNode[]} />
				</div>
			{/if}
		</div>
		{#if node.trailing?.length}
			<div class="shrink-0">
				<CoreChildren nodes={node.trailing as readonly CanonicalUiNode[]} />
			</div>
		{/if}
	</div>
{/if}
