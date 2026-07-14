<script lang="ts">
	import Button from '$lib/shared/ui/Button.svelte';
	import { navigableContainer } from '$lib/shared/keyboard/actions';
	import type { ItemNode, ListNode } from '$lib/unode/core/ast';
	import type { CanonicalNode } from '$lib/unode/core/normalize';
	import {
		getActionRunner,
		getRendererConfig,
		getRendererStateStore
	} from '$lib/widgets/app-plugin-renderer/context';
	import CoreItemNode from './CoreItemNode.svelte';
	import { resolveStringValue } from '../resolve';

	let { node }: { node: CanonicalNode<ListNode> } = $props();

	const containerId = $props.id();
	const navDefaults = getRendererConfig().navigation.list;
	const autoLoadContinuation = getRendererConfig().collections.autoLoadContinuation;
	const runAction = getActionRunner();
	const uiState = getRendererStateStore();
	const rootMargin = 160;

	let loading = $state(false);
	let userHasScrolled = $state(false);

	const incremental = $derived(node.continuation?.kind === 'incremental' ? node.continuation : undefined);
	const remote = $derived(node.continuation?.kind === 'remote' ? node.continuation : undefined);
	const total = $derived(node.items.length);
	const visibleCount = $derived(
		incremental
			? Math.max(
					incremental.initial,
					Math.min(total, Number(uiState.getPrimitive(incremental.binding, incremental.initial)))
				)
			: total
	);
	const visibleItems = $derived(incremental ? node.items.slice(0, visibleCount) : node.items);
	const hasMore = $derived(
		incremental ? visibleCount < total : Boolean(remote?.hasMore && remote.loadMore)
	);
	const loadMoreLabel = $derived(
		resolveStringValue(node.continuation?.label, uiState, 'Load more')
	);
	const loadingLabel = $derived(
		resolveStringValue(remote?.loadingLabel, uiState, 'Loading...')
	);

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

	function isNearViewport(element: HTMLDivElement, root: HTMLElement | null) {
		const elementRect = element.getBoundingClientRect();
		const rootRect = root?.getBoundingClientRect();
		const topBoundary = (rootRect?.top ?? 0) - rootMargin;
		const bottomBoundary = (rootRect?.bottom ?? window.innerHeight) + rootMargin;

		return elementRect.top <= bottomBoundary && elementRect.bottom >= topBoundary;
	}

	function maybeAutoLoad(element: HTMLDivElement, root: HTMLElement | null, allowWithoutScroll = false) {
		if (!hasMore || loading) return;
		if (!userHasScrolled && !allowWithoutScroll) return;
		if (!isNearViewport(element, root)) return;
		void triggerLoad();
	}

	function observeContinuation(element: HTMLDivElement) {
		if (typeof window === 'undefined' || !autoLoadContinuation) return;

		const root = document.getElementById('main-scroll');

		const onRootScroll = () => {
			if (root && !userHasScrolled && root.scrollTop > 0) {
				userHasScrolled = true;
			}
			maybeAutoLoad(element, root);
		};

		root?.addEventListener('scroll', onRootScroll, { passive: true });

		const observer = new IntersectionObserver(
			() => {
				maybeAutoLoad(element, root);
			},
			{ root: root ?? null, rootMargin: `${rootMargin}px 0px`, threshold: 0.01 }
		);

		observer.observe(element);
		if (incremental) {
			queueMicrotask(() => {
				maybeAutoLoad(element, root, true);
			});
		}

		return () => {
			root?.removeEventListener('scroll', onRootScroll);
			observer.disconnect();
		};
	}
</script>

<div
	class="flex flex-col gap-[var(--gap-2)]"
	use:navigableContainer={{
		id: containerId,
		zone: navDefaults.zone,
		mode: 'list',
		axis: navDefaults.axis,
		wrap: navDefaults.wrap,
		pageJump: navDefaults.pageJump
	}}
>
	{#each visibleItems as item ((item as typeof node.items[number] & { _key: string })._key)}
		<CoreItemNode
			node={item as CanonicalNode<ItemNode>}
			{containerId}
			itemId={`list-${containerId}-${(item as typeof node.items[number] & { _key: string })._key}`}
		/>
	{/each}

	{#if hasMore && (remote?.loadMore || incremental)}
		<div class="flex justify-center pt-[var(--gap-2)]">
			<Button onclick={triggerLoad} disabled={loading}>
				{incremental ? loadMoreLabel : loading ? loadingLabel : loadMoreLabel}
			</Button>
		</div>
		{#if autoLoadContinuation}
			<div class="h-[var(--space-2)]" {@attach observeContinuation}></div>
		{/if}
	{/if}
</div>
