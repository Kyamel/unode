<script lang="ts">
	import type { MediaNode } from '$lib/unode/core/ast';
	import { m } from '$lib/shared/i18n/messages';

	let { node }: { node: MediaNode } = $props();
	let previewOpen = $state(false);
	let previewZoom = $state(1);
	let panX = $state(0);
	let panY = $state(0);
	let panning = $state(false);
	let pointerOffsetX = $state(0);
	let pointerOffsetY = $state(0);

	const aspectMap: Record<string, string> = {
		square: '1 / 1',
		poster: '2 / 3',
		video: '16 / 9',
		auto: 'auto'
	};

	const ratio = $derived(aspectMap[node.aspectRatio ?? 'auto'] ?? 'auto');
	const fit = $derived(node.mediaKind === 'cover' ? 'cover' : 'contain');
	const src = $derived(
		node.ref.type === 'url'
			? node.ref.src
			: node.ref.type === 'asset'
				? node.ref.name
				: undefined
	);
	const hasImage = $derived(Boolean(src));
	const canPreview = $derived(Boolean(node.expandable) && hasImage);
	const isInteractive = $derived(canPreview);
	const placeholderLabel = $derived(
		node.ref.type === 'placeholder' ? (node.ref.label ?? node.alt ?? 'Unavailable') : (node.alt ?? 'Unavailable')
	);
	const mediaAriaLabel = $derived(
		node.alt ??
			(node.ref.type === 'placeholder' && node.ref.kind === 'cover'
				? 'Cover placeholder'
				: node.ref.type === 'placeholder' && node.ref.kind === 'avatar'
					? 'Avatar placeholder'
					: 'Media placeholder')
	);
	const triggerAriaLabel = $derived(
		canPreview ? `${m.reader_open_fullscreen()}: ${mediaAriaLabel}` : mediaAriaLabel
	);
	const canPan = $derived(previewZoom > 1);
	const previewImageStyle = $derived(
		`transform: translate(${panX}px, ${panY}px) scale(${previewZoom}); transform-origin: center center;`
	);
	const previewCursorClass = $derived(
		canPan ? (panning ? 'cursor-grabbing' : 'cursor-grab') : 'cursor-zoom-in'
	);

	$effect(() => {
		if (!previewOpen) return;

		const previousOverflow = document.body.style.overflow;
		document.body.style.overflow = 'hidden';

		return () => {
			document.body.style.overflow = previousOverflow;
		};
	});

	function handleClick() {
		if (canPreview) {
			resetPreviewViewport();
			previewOpen = true;
		}
	}

	function closePreview() {
		previewOpen = false;
		resetPreviewViewport();
	}

	function handleDocumentKeydown(event: KeyboardEvent) {
		if (!previewOpen) return;
		if (event.key !== 'Escape') return;
		event.preventDefault();
		closePreview();
	}

	function resetPreviewViewport() {
		previewZoom = 1;
		panX = 0;
		panY = 0;
		panning = false;
	}

	function setPreviewZoom(nextZoom: number) {
		const clampedZoom = Math.min(4, Math.max(1, Math.round(nextZoom * 100) / 100));
		previewZoom = clampedZoom;

		if (clampedZoom === 1) {
			panX = 0;
			panY = 0;
			panning = false;
		}
	}

	function zoomInPreview() {
		setPreviewZoom(previewZoom + 0.25);
	}

	function zoomOutPreview() {
		setPreviewZoom(previewZoom - 0.25);
	}

	function handlePreviewWheel(event: WheelEvent) {
		if (!previewOpen) return;
		event.preventDefault();
		setPreviewZoom(previewZoom + (event.deltaY < 0 ? 0.25 : -0.25));
	}

	function handlePreviewPointerDown(event: PointerEvent) {
		if (!canPan || event.button !== 0) return;

		const target = event.currentTarget as HTMLElement | null;
		target?.setPointerCapture(event.pointerId);
		panning = true;
		pointerOffsetX = event.clientX - panX;
		pointerOffsetY = event.clientY - panY;
	}

	function handlePreviewPointerMove(event: PointerEvent) {
		if (!panning) return;
		panX = event.clientX - pointerOffsetX;
		panY = event.clientY - pointerOffsetY;
	}

	function stopPreviewPanning(event?: PointerEvent) {
		const target = event?.currentTarget as HTMLElement | null;
		if (event && target?.hasPointerCapture(event.pointerId)) {
			target.releasePointerCapture(event.pointerId);
		}
		panning = false;
	}
</script>

<svelte:document onkeydown={handleDocumentKeydown} />

{#if isInteractive}
	<button
		type="button"
		class="group relative overflow-hidden rounded-[var(--radius-none)] border-[length:var(--border-w-heavy)] border-[var(--color-border-strong)] bg-[var(--color-surface-1)] cursor-zoom-in transition-transform duration-[var(--dur-normal)] hover:-translate-y-[2px] hover:shadow-[var(--shadow-xl)] focus-visible:outline focus-visible:outline-[length:var(--border-w-strong)] focus-visible:outline-[var(--color-text)]"
		style={`aspect-ratio:${ratio};`}
		onclick={handleClick}
		aria-label={triggerAriaLabel}
	>
		{#if hasImage && src}
			<img
				src={src}
				alt={node.alt}
				class="h-full w-full"
				style={`object-fit:${fit};`}
				loading="lazy"
			/>
		{:else}
			<div class="flex h-full w-full items-center justify-center bg-[linear-gradient(135deg,var(--color-surface-2),var(--color-surface-1))] p-[var(--space-4)] text-center">
				<span class="text-[length:var(--fs-sm)] font-black leading-[var(--lh-snug)] text-[var(--color-text-muted)]">
					{placeholderLabel}
				</span>
			</div>
		{/if}

		<span
			class="pointer-events-none absolute inset-0 border-[length:var(--border-w-heavy)] border-transparent transition-colors duration-[var(--dur-normal)] group-hover:border-[var(--color-primary)] group-focus-visible:border-[var(--color-primary)]"
			aria-hidden="true"
		></span>
		<span
			class="pointer-events-none absolute inset-x-0 bottom-0 flex items-center justify-between gap-[var(--gap-2)] bg-[linear-gradient(180deg,transparent,rgba(0,0,0,0.82))] px-[var(--space-3)] py-[var(--space-3)] opacity-0 transition-all duration-[var(--dur-normal)] group-hover:opacity-100 group-focus-visible:opacity-100"
			aria-hidden="true"
		>
			<span class="text-[length:var(--fs-2xs)] font-black uppercase tracking-[0.2em] text-[var(--color-text-inverse)]">
				{m.reader_open_fullscreen()}
			</span>
			<span class="flex h-[var(--size-12)] w-[var(--size-12)] items-center justify-center border-[length:var(--border-w-strong)] border-[var(--color-text-inverse)] bg-[var(--color-primary)] text-[var(--color-primary-contrast)] shadow-[var(--shadow-md)]">
				<svg viewBox="0 0 24 24" class="h-[var(--space-5)] w-[var(--space-5)]" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
					<path d="M4 9V4h5"></path>
					<path d="M20 9V4h-5"></path>
					<path d="M4 15v5h5"></path>
					<path d="M20 15v5h-5"></path>
				</svg>
			</span>
		</span>
	</button>
{:else}
	<div
		class="overflow-hidden rounded-[var(--radius-none)] border-[length:var(--border-w-heavy)] border-[var(--color-border-strong)] bg-[var(--color-surface-1)]"
		style={`aspect-ratio:${ratio};`}
	>
		{#if hasImage && src}
			<img
				src={src}
				alt={node.alt}
				class="h-full w-full"
				style={`object-fit:${fit};`}
				loading="lazy"
			/>
		{:else}
			<div class="flex h-full w-full items-center justify-center bg-[linear-gradient(135deg,var(--color-surface-2),var(--color-surface-1))] p-[var(--space-4)] text-center">
				<span class="text-[length:var(--fs-sm)] font-black leading-[var(--lh-snug)] text-[var(--color-text-muted)]">
					{placeholderLabel}
				</span>
			</div>
		{/if}
	</div>
{/if}

{#if previewOpen && src}
	<div
		class="fixed inset-0 z-[var(--z-modal)] overflow-hidden bg-[var(--color-ink)]"
		role="dialog"
		aria-modal="true"
		aria-label={mediaAriaLabel}
	>
		<button
			type="button"
			class="absolute inset-0 bg-[var(--color-overlay-soft)]"
			aria-label={m.reader_close()}
			onclick={closePreview}
		></button>

		<div class="relative z-10 flex h-full w-full flex-col">
			<div class="pointer-events-none absolute inset-x-0 top-0 z-20 flex items-start justify-between gap-[var(--gap-3)] p-[var(--space-4)]">
				<div class="border-[length:var(--border-w-strong)] border-[var(--color-text-inverse-faint)] bg-[var(--color-overlay-strong)] px-[var(--space-3)] py-[var(--space-2)] text-[length:var(--fs-2xs)] font-black uppercase tracking-[0.2em] text-[var(--color-text-inverse-muted)]">
					Scroll to zoom
				</div>

				<div class="pointer-events-auto flex items-center gap-[var(--gap-2)]">
					<button
						type="button"
						class="border-[length:var(--border-w-strong)] border-[var(--color-text-inverse)] bg-[var(--color-overlay-strong)] px-[var(--space-3)] py-[var(--space-2)] text-[length:var(--fs-xs)] font-black uppercase tracking-[0.18em] text-[var(--color-text-inverse)] transition-colors hover:bg-[var(--color-primary)] hover:text-[var(--color-primary-contrast)]"
						aria-label="Zoom out"
						onclick={zoomOutPreview}
						disabled={previewZoom <= 1}
					>
						-
					</button>
					<button
						type="button"
						class="border-[length:var(--border-w-strong)] border-[var(--color-text-inverse)] bg-[var(--color-overlay-strong)] px-[var(--space-3)] py-[var(--space-2)] text-[length:var(--fs-xs)] font-black uppercase tracking-[0.18em] text-[var(--color-text-inverse)]"
						aria-label="Reset zoom"
						onclick={resetPreviewViewport}
					>
						{Math.round(previewZoom * 100)}%
					</button>
					<button
						type="button"
						class="border-[length:var(--border-w-strong)] border-[var(--color-text-inverse)] bg-[var(--color-overlay-strong)] px-[var(--space-3)] py-[var(--space-2)] text-[length:var(--fs-xs)] font-black uppercase tracking-[0.18em] text-[var(--color-text-inverse)] transition-colors hover:bg-[var(--color-primary)] hover:text-[var(--color-primary-contrast)]"
						aria-label="Zoom in"
						onclick={zoomInPreview}
						disabled={previewZoom >= 4}
					>
						+
					</button>
					<button
						type="button"
						class="border-[length:var(--border-w-strong)] border-[var(--color-text-inverse)] bg-[var(--color-overlay-strong)] p-[var(--space-2)] text-[var(--color-text-inverse)] transition-colors hover:bg-[var(--color-primary)] hover:text-[var(--color-primary-contrast)]"
						aria-label={m.reader_close()}
						onclick={closePreview}
					>
						<svg viewBox="0 0 24 24" class="h-[var(--space-5)] w-[var(--space-5)]" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
							<path d="M18 6L6 18"></path>
							<path d="M6 6l12 12"></path>
						</svg>
					</button>
				</div>
			</div>

			<div
				class="relative flex flex-1 items-center justify-center overflow-hidden p-[var(--space-4)] md:p-[var(--space-6)]"
				onwheel={handlePreviewWheel}
			>
				<button
					type="button"
					class={`relative flex h-full w-full items-center justify-center overflow-hidden border-[length:var(--border-w-heavy)] border-[var(--color-text-inverse-faint)] bg-[var(--color-ink)] p-[var(--space-4)] ${previewCursorClass}`}
					style={`touch-action:${canPan ? 'none' : 'pan-y'};`}
					aria-label={mediaAriaLabel}
					onpointerdown={handlePreviewPointerDown}
					onpointermove={handlePreviewPointerMove}
					onpointerup={stopPreviewPanning}
					onpointercancel={stopPreviewPanning}
				>
					<img
						src={src}
						alt={node.alt}
						class="mx-auto block h-auto max-h-full w-auto max-w-full select-none object-contain transition-transform duration-[var(--dur-fast)]"
						style={previewImageStyle}
						loading="lazy"
						draggable="false"
					/>
				</button>
			</div>
		</div>
	</div>
{/if}
