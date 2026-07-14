<script lang="ts">
	import { tick } from 'svelte';
	import { fade } from 'svelte/transition';
	import type { MenuNode } from '$lib/unode/core/ast';
	import { getActionRunner, getRendererStateStore } from '$lib/widgets/app-plugin-renderer/context';
	import { resolveBooleanValue, resolveStringValue } from '../resolve';

	let { node }: { node: MenuNode } = $props();

	const runAction = getActionRunner();
	const uiState = getRendererStateStore();
	const menuUid = $props.id();
	const rootId = `${menuUid}-root`;
	const triggerId = $derived(node.id ?? `${menuUid}-trigger`);
	const menuId = `${menuUid}-menu`;

	let open = $state(false);

	const hasSelection = $derived(node.items.some((item) => item.selected));
	const buttonVariant = $derived(
		node.intent === 'primary'
			? 'bg-[var(--color-primary)] text-[var(--color-primary-contrast)]'
			: node.intent === 'danger'
				? 'bg-[var(--color-danger)] text-[var(--color-danger-contrast)]'
				: node.intent === 'ghost'
					? 'bg-transparent text-[var(--color-text)]'
					: 'bg-[var(--color-surface-1)] text-[var(--color-text)]'
	);
	const menuAlignClass = $derived(node.align === 'end' ? 'right-0' : 'left-0');
	const label = $derived(resolveStringValue(node.label, uiState));

	function itemLabel(index: number) {
		return resolveStringValue(node.items[index]?.label, uiState);
	}

	function itemDisabled(index: number) {
		return resolveBooleanValue(node.items[index]?.disabled, uiState, false);
	}

	function getRootEl(): HTMLElement | null {
		return document.getElementById(rootId);
	}

	function getTriggerEl(): HTMLButtonElement | null {
		const element = document.getElementById(triggerId);
		return element instanceof HTMLButtonElement ? element : null;
	}

	function getMenuEl(): HTMLDivElement | null {
		const element = document.getElementById(menuId);
		return element instanceof HTMLDivElement ? element : null;
	}

	function enabledIndexes(): number[] {
		return node.items
			.map((_, index) => index)
			.filter((index) => !itemDisabled(index));
	}

	function currentIndex(): number {
		const enabled = enabledIndexes();
		if (!enabled.length) return -1;
		const selected = node.items.findIndex((item, index) => item.selected && !itemDisabled(index));
		return selected >= 0 ? selected : enabled[0];
	}

	async function focusItem(index: number) {
		await tick();
		const menuEl = getMenuEl();
		const items = menuEl
			? Array.from(menuEl.querySelectorAll<HTMLButtonElement>('[role^="menuitem"]'))
			: [];
		items[index]?.focus();
	}

	async function openMenu(index = currentIndex()) {
		open = true;
		await focusItem(index);
	}

	function closeMenu(returnFocus = false) {
		open = false;
		if (returnFocus) {
			getTriggerEl()?.focus();
		}
	}

	async function toggleMenu() {
		if (open) {
			closeMenu(true);
			return;
		}
		await openMenu();
	}

	async function activateItem(index: number) {
		const item = node.items[index];
		if (!item || itemDisabled(index)) return;
		closeMenu(false);
		await runAction(item.action);
	}

	function moveFocus(direction: 1 | -1) {
		const enabled = enabledIndexes();
		if (!enabled.length) return;

		const menuEl = getMenuEl();
		const items = menuEl
			? Array.from(menuEl.querySelectorAll<HTMLButtonElement>('[role^="menuitem"]'))
			: [];
		const activeElement = document.activeElement as HTMLButtonElement | null;
		const current = items.findIndex((item) => item === activeElement);
		const next = current < 0 ? 0 : (current + direction + enabled.length) % enabled.length;
		items[next]?.focus();
	}

	function handleTriggerKeydown(event: KeyboardEvent) {
		if (event.key === 'ArrowDown') {
			event.preventDefault();
			void openMenu(0);
			return;
		}

		if (event.key === 'ArrowUp') {
			event.preventDefault();
			const enabled = enabledIndexes();
			void openMenu(Math.max(0, enabled.length - 1));
		}
	}

	function handleMenuKeydown(event: KeyboardEvent) {
		if (!open) return;

		if (event.key === 'Escape') {
			event.preventDefault();
			closeMenu(true);
			return;
		}

		if (event.key === 'ArrowDown') {
			event.preventDefault();
			moveFocus(1);
			return;
		}

		if (event.key === 'ArrowUp') {
			event.preventDefault();
			moveFocus(-1);
			return;
		}

		if (event.key === 'Home') {
			event.preventDefault();
			void focusItem(0);
			return;
		}

		if (event.key === 'End') {
			event.preventDefault();
			const enabled = enabledIndexes();
			void focusItem(Math.max(0, enabled.length - 1));
		}
	}

	function handleDocumentClick(event: MouseEvent) {
		if (!open) return;
		const target = event.target;
		if (target instanceof Node && getRootEl()?.contains(target)) return;
		closeMenu(false);
	}

	function handleDocumentKeydown(event: KeyboardEvent) {
		if (event.key === 'Escape' && open) {
			event.preventDefault();
			closeMenu(true);
		}
	}
</script>

<svelte:document onclick={handleDocumentClick} onkeydown={handleDocumentKeydown} />

<div class="relative inline-flex" id={rootId}>
	<button
		id={triggerId}
		type="button"
		class={`inline-flex items-center gap-[var(--gap-2)] border-[length:var(--border-w-heavy)] border-[var(--color-border-strong)] px-[var(--space-4)] py-[var(--space-2)] text-[length:var(--fs-xs)] font-black uppercase tracking-widest shadow-[var(--shadow-md)] transition-colors hover:bg-[var(--color-text)] hover:text-[var(--color-bg)] ${buttonVariant}`}
		aria-haspopup="menu"
		aria-expanded={open}
		aria-controls={menuId}
		onclick={toggleMenu}
		onkeydown={handleTriggerKeydown}
	>
		<span>{label}</span>
		<span
			class="text-[length:var(--fs-2xs)] text-[var(--color-text-muted)] transition-transform"
			style={`transform: rotate(${open ? 180 : 0}deg);`}
			aria-hidden="true"
		>
			▼
		</span>
	</button>

	{#if open}
		<div
			id={menuId}
			class={`absolute top-[calc(100%+var(--space-2))] z-[var(--z-toast)] max-w-[calc(100vw-var(--space-8))] border-[length:var(--border-w-heavy)] border-[var(--color-border-strong)] bg-[var(--color-bg)] p-[var(--space-2)] shadow-[var(--shadow-xl)] ${menuAlignClass}`}
			style="width: max-content;"
			role="menu"
			tabindex="-1"
			aria-label={label}
			onkeydown={handleMenuKeydown}
			transition:fade={{ duration: 120 }}
		>
			<div class="flex flex-col gap-[var(--gap-1)]" style="width: max-content;">
				{#each node.items as item, index (item.key)}
					<button
						type="button"
						class={`flex w-full items-center justify-between gap-[var(--gap-3)] border-[length:var(--border-w-strong)] border-transparent px-[var(--space-3)] py-[var(--space-2)] text-left text-[length:var(--fs-xs)] font-bold uppercase tracking-[0.12em] transition-colors ${item.selected ? 'bg-[var(--color-primary-alpha-10)] text-[var(--color-text)]' : 'bg-transparent text-[var(--color-text)] hover:bg-[var(--color-surface-1)]'} ${itemDisabled(index) ? 'opacity-50' : ''}`}
						role={hasSelection ? 'menuitemradio' : 'menuitem'}
						aria-checked={hasSelection ? item.selected === true : undefined}
						aria-disabled={itemDisabled(index)}
						disabled={itemDisabled(index)}
						onclick={() => void activateItem(index)}
					>
						<span class="flex-1 whitespace-nowrap">{itemLabel(index)}</span>
						{#if item.selected}
							<span class="text-[var(--color-primary)]" aria-hidden="true">●</span>
						{/if}
					</button>
				{/each}
			</div>
		</div>
	{/if}
</div>
