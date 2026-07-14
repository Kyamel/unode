<script lang="ts">
	import Button from '$lib/shared/ui/Button.svelte';
	import type { ActionNode } from '$lib/unode/core/ast';
	import { getActionRunner, getRendererStateStore } from '$lib/widgets/app-plugin-renderer/context';
	import { resolveBooleanValue, resolveStringValue } from '../resolve';

	let { node }: { node: ActionNode } = $props();

	const runAction = getActionRunner();
	const uiState = getRendererStateStore();

	const label = $derived(resolveStringValue(node.label, uiState));
	const disabled = $derived(resolveBooleanValue(node.disabled, uiState, false));
	const variant = $derived(
		node.variant === 'link'
			? 'link'
			: node.intent === 'primary'
				? 'solid'
				: node.intent === 'ghost'
					? 'ghost'
					: 'outline'
	);

	function handleClick() {
		if (disabled) return;
		void runAction(node.action);
	}
</script>

<Button
	id={node.id}
	variant={variant}
	disabled={disabled}
	ariaLabel={label}
	onclick={handleClick}
	className={node.intent === 'danger' ? 'text-[var(--color-danger)]' : ''}
>
	{label}
</Button>
