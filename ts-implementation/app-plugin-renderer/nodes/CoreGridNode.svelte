<script lang="ts">
	import { onMount } from 'svelte';
	import Button from '$lib/shared/ui/Button.svelte';
	import { navigableContainer, navItem } from '$lib/shared/keyboard/actions';
	import type { GridNode } from '$lib/unode/core/ast';
	import type { CanonicalNode, CanonicalUiNode } from '$lib/unode/core/normalize';
	import {
		getActionRunner,
		getRendererConfig,
		getRendererStateStore
	} from '$lib/widgets/app-plugin-renderer/context';
	import CoreUiRenderer from '../CoreUiRenderer.svelte';
	import { resolveStringValue } from '../resolve';

	let { node }: { node: CanonicalNode<GridNode> } = $props();

	const gapMap: Record<string, string> = {
		none: '0',
		xs: 'var(--gap-1)',
		sm: 'var(--gap-2)',
		md: 'var(--gap-3)',
		lg: 'var(--gap-5)'
	};

	const containerId = $props.id();
	let cols = $state(1);
	let loading = $state(false);
	let userHasScrolled = $state(false);
	const uiState = getRendererStateStore();
	const gap = $derived(gapMap[node.gap ?? 'md'] ?? gapMap.md);
	const rendererConfig = getRendererConfig();
	const runAction = getActionRunner();
	const breakpoints = rendererConfig.breakpoints;
	const navDefaults = rendererConfig.navigation.grid;
	const autoLoadContinuation = rendererConfig.collections.autoLoadContinuation;
	const incremental = $derived(node.continuation?.kind === 'incremental' ? node.continuation : undefined);
	const remote = $derived(node.continuation?.kind === 'remote' ? node.continuation : undefined);
	const total = $derived(node.children.length);
	const visibleCount = $derived(
		incremental
			? Math.max(
					incremental.initial,
					Math.min(total, Number(uiState.getPrimitive(incremental.binding, incremental.initial)))
				)
			: total
	);
	const visibleChildren = $derived(incremental ? node.children.slice(0, visibleCount) : node.children);
	const visibleCanonicalChildren = $derived(visibleChildren as readonly CanonicalUiNode[]);
	const hasMore = $derived(
		incremental ? visibleCount < total : Boolean(remote?.hasMore && remote.loadMore)
	);
	const loadMoreLabel = $derived(
		resolveStringValue(node.continuation?.label, uiState, 'Load more')
	);
	const loadingLabel = $derived(
		resolveStringValue(remote?.loadingLabel, uiState, 'Loading...')
	);

	function resolveCols(width: number) {
		const columns = node.columns ?? {};
		if (width >= breakpoints.xl && columns.xl) return columns.xl;
		if (width >= breakpoints.lg && columns.lg) return columns.lg;
		if (width >= breakpoints.md && columns.md) return columns.md;
		if (width >= breakpoints.sm && columns.sm) return columns.sm;
		return columns.base ?? 1;
	}

	function updateCols() {
		if (typeof window === 'undefined') return;
		cols = resolveCols(window.innerWidth);
	}

	function isEditableTarget(target: EventTarget | null): boolean {
		if (!(target instanceof HTMLElement)) return false;
		const tag = target.tagName.toLowerCase();
		if (tag === 'input' || tag === 'textarea' || tag === 'select') return true;
		return target.isContentEditable;
	}

	function isItemActionable(item: CanonicalUiNode) {
		return item.kind === 'pressable' || item.kind === 'action' || item.kind === 'menu';
	}

	async function triggerLoad() {
		if (incremental) {
			uiState.set(incremental.binding, Math.min(total, visibleCount + incremental.step));
			return;
		}

		if (!remote?.loadMore || loading || !hasMore) return;
		loading = true;
		try {
			await runAction(remote.loadMore);
		} finally {
			loading = false;
		}
	}

	function handleItemKeydown(event: KeyboardEvent) {
		if (event.key !== 'Enter' && event.key !== ' ') return;
		if (isEditableTarget(event.target)) return;
		const root = event.currentTarget as HTMLElement | null;
		if (!root) return;
		const activator =
			root.querySelector<HTMLElement>('[data-nav-activate]') ??
			root.querySelector<HTMLElement>('button,a[href],[role="button"]');
		if (!activator) return;
		event.preventDefault();
		activator.click();
	}

	onMount(() => {
		updateCols();
		const onResize = () => updateCols();
		window.addEventListener('resize', onResize, { passive: true });
		return () => window.removeEventListener('resize', onResize);
	});

	function observeContinuation(element: HTMLDivElement) {
		if (typeof window === 'undefined' || !autoLoadContinuation) return;

		const root = document.getElementById('main-scroll');
		const rootMargin = 160;

		const maybeAutoLoad = (allowWithoutScroll = false) => {
			if (!hasMore || loading) return;
			if (!userHasScrolled && !allowWithoutScroll) return;

			const elementRect = element.getBoundingClientRect();
			const rootRect = root?.getBoundingClientRect();
			const topBoundary = (rootRect?.top ?? 0) - rootMargin;
			const bottomBoundary = (rootRect?.bottom ?? window.innerHeight) + rootMargin;
			const nearViewport = elementRect.top <= bottomBoundary && elementRect.bottom >= topBoundary;

			if (!nearViewport) return;
			void triggerLoad();
		};

		const onRootScroll = () => {
			if (root && !userHasScrolled && root.scrollTop > 0) {
				userHasScrolled = true;
			}
			maybeAutoLoad();
		};

		root?.addEventListener('scroll', onRootScroll, { passive: true });

		const observer = new IntersectionObserver(
			() => {
				maybeAutoLoad();
			},
			{ root: root ?? null, rootMargin: `${rootMargin}px 0px`, threshold: 0.01 }
		);

		observer.observe(element);
		if (incremental) {
			queueMicrotask(() => {
				maybeAutoLoad(true);
			});
		}

		return () => {
			root?.removeEventListener('scroll', onRootScroll);
			observer.disconnect();
		};
	}
</script>

<div
	class="grid"
	style={`grid-template-columns:repeat(${cols}, minmax(0, 1fr));gap:${gap};`}
	use:navigableContainer={{
		id: containerId,
		zone: navDefaults.zone,
		mode: 'grid',
		strategy: navDefaults.strategy,
		axis: navDefaults.axis,
		pageRows: navDefaults.pageRows
	}}
>
	{#each visibleCanonicalChildren as child (child._key)}
		{@const actionable = isItemActionable(child)}
		<div
			class="min-w-0 focus-visible:outline focus-visible:outline-[length:var(--border-w-strong)] focus-visible:outline-[var(--color-text)]"
			role={actionable ? 'button' : undefined}
			use:navItem={{ containerId, id: `grid-${containerId}-${child._key}` }}
			onkeydown={actionable ? handleItemKeydown : undefined}
		>
			<CoreUiRenderer node={child} />
		</div>
	{/each}
</div>

{#if hasMore && (remote?.loadMore || incremental)}
	<div class="mt-[var(--space-4)] flex justify-center">
		<Button onclick={triggerLoad} disabled={loading}>
			{incremental ? loadMoreLabel : loading ? loadingLabel : loadMoreLabel}
		</Button>
	</div>
	{#if autoLoadContinuation}
		<div class="h-[var(--space-2)]" {@attach observeContinuation}></div>
	{/if}
{/if}
