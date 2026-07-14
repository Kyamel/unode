<script lang="ts">
	import type {
		ActionNode,
		ActionRef,
		EmptyStateNode,
		InputNode,
		JsonValue,
		Primitive,
		StatusNode,
		ValueNode
	} from '$lib/unode/core/ast';
	import type { CanonicalRootNode, CanonicalUiNode } from '$lib/unode/core/normalize';
	import { getActionRunner, getRendererStateStore } from '$lib/widgets/app-plugin-renderer/context';
	import CoreChildren from './CoreChildren.svelte';
	import CoreActionNode from './nodes/CoreActionNode.svelte';
	import CoreDisclosureNode from './nodes/CoreDisclosureNode.svelte';
	import CoreGridNode from './nodes/CoreGridNode.svelte';
	import CoreItemNode from './nodes/CoreItemNode.svelte';
	import CoreListNode from './nodes/CoreListNode.svelte';
	import CoreMediaNode from './nodes/CoreMediaNode.svelte';
	import CoreMenuNode from './nodes/CoreMenuNode.svelte';
	import CorePressableNode from './nodes/CorePressableNode.svelte';
	import { resolveBooleanValue, resolveNumberValue, resolvePrimitiveValue, resolveStringValue } from './resolve';

	let { node }: { node: CanonicalRootNode } = $props();

	const runAction = getActionRunner();
	const uiState = getRendererStateStore();
	const locale =
		typeof document === 'undefined' ? 'en' : (document.documentElement.lang || navigator.language || 'en');

	function gapValue(gap: 'none' | 'xs' | 'sm' | 'md' | 'lg' | undefined) {
		const gapMap: Record<string, string> = {
			none: '0',
			xs: 'var(--gap-1)',
			sm: 'var(--gap-2)',
			md: 'var(--gap-3)',
			lg: 'var(--gap-5)'
		};

		return gapMap[gap ?? 'md'] ?? gapMap.md;
	}

	function textRoleClass(
		role: 'heading' | 'title' | 'subtitle' | 'body' | 'label' | 'caption' | 'code' | 'hint' | undefined,
		tone: 'default' | 'muted' | 'info' | 'success' | 'warning' | 'danger' | undefined,
		emphasis: 'normal' | 'strong' | undefined
	) {
		const base =
			role === 'heading'
				? 'text-[length:var(--fs-3xl)] font-black leading-[var(--lh-tight)]'
				: role === 'title'
					? 'text-[length:var(--fs-xl)] font-bold leading-[var(--lh-snug)]'
					: role === 'subtitle'
						? 'text-[length:var(--fs-md)] font-semibold leading-[var(--lh-normal)]'
						: role === 'label'
							? 'text-[length:var(--fs-xs)] font-black uppercase tracking-[0.2em]'
							: role === 'caption' || role === 'code'
								? 'text-[length:var(--fs-xs)] leading-[var(--lh-normal)]'
								: role === 'hint'
									? 'text-[length:var(--fs-sm)] leading-[var(--lh-relaxed)]'
									: 'text-[length:var(--fs-sm)] leading-[var(--lh-relaxed)]';

		const toneClass =
			tone === 'info'
				? 'text-[var(--color-info)]'
				: tone === 'success'
					? 'text-[var(--color-success)]'
					: tone === 'warning'
						? 'text-[var(--color-warning)]'
						: tone === 'danger'
							? 'text-[var(--color-danger)]'
							: tone === 'muted' || role === 'hint' || role === 'label' || role === 'caption' || role === 'code'
								? 'text-[var(--color-text-muted)]'
								: role === 'subtitle'
									? 'text-[var(--color-text-subtle)]'
									: 'text-[var(--color-text)]';

		return `${base} ${toneClass} ${emphasis === 'strong' ? 'font-black' : ''}`.trim();
	}

	function truncateStyle(
		role: 'heading' | 'title' | 'subtitle' | 'body' | 'label' | 'caption' | 'code' | 'hint' | undefined,
		truncate: boolean | undefined
	) {
		if (!truncate) return undefined;
		const lineClamp = role === 'heading' || role === 'title' || role === 'subtitle' ? 2 : 1;
		if (lineClamp <= 1) return 'white-space:nowrap;';
		return `display:-webkit-box;-webkit-box-orient:vertical;-webkit-line-clamp:${lineClamp};`;
	}

	function badgeToneClass(tone: 'default' | 'muted' | 'info' | 'success' | 'warning' | 'danger' | undefined) {
		return tone === 'info'
			? 'bg-[var(--color-info)] text-[var(--color-info-contrast)]'
			: tone === 'success'
				? 'bg-[var(--color-success)] text-[var(--color-success-contrast)]'
				: tone === 'warning'
					? 'bg-[var(--color-warning)] text-[var(--color-warning-contrast)]'
					: tone === 'danger'
						? 'bg-[var(--color-danger)] text-[var(--color-danger-contrast)]'
						: 'bg-[var(--color-surface-2)] text-[var(--color-text)]';
	}

	function actionsAlignClass(align: 'start' | 'center' | 'end' | undefined) {
		return align === 'end' ? 'justify-end' : align === 'center' ? 'justify-center' : 'justify-start';
	}

	function formatValue(node: ValueNode) {
		const resolved = resolvePrimitiveValue(node.value, uiState, '' as Primitive);

		if (resolved === null || resolved === undefined || resolved === '') return '';

		if (node.format === 'currency') {
			try {
				return new Intl.NumberFormat(locale, {
					style: 'currency',
					currency: node.currencyCode ?? 'USD'
				}).format(Number(resolved));
			} catch {
				return String(resolved);
			}
		}

		if (node.format === 'number' || node.format === 'percent') {
			const value = Number(resolved);
			if (Number.isFinite(value)) {
				return new Intl.NumberFormat(locale, {
					style: node.format === 'percent' ? 'percent' : 'decimal'
				}).format(node.format === 'percent' ? value : value);
			}
		}

		if (node.format === 'date' || node.format === 'datetime') {
			const date = new Date(String(resolved));
			if (!Number.isNaN(date.getTime())) {
				return new Intl.DateTimeFormat(locale, {
					dateStyle: 'medium',
					timeStyle: node.format === 'datetime' ? 'short' : undefined
				}).format(date);
			}
		}

		return String(resolved);
	}

	function statusClasses(severity: StatusNode['severity']) {
		return severity === 'danger'
			? 'border-[var(--color-danger)] text-[var(--color-danger)]'
			: severity === 'warning'
				? 'border-[var(--color-warning)] text-[var(--color-warning)]'
				: severity === 'success'
					? 'border-[var(--color-success)] text-[var(--color-success)]'
					: 'border-[var(--color-info)] text-[var(--color-info)]';
	}

	function emptyToneClasses(node: EmptyStateNode) {
		return node.actions?.length ? 'text-[var(--color-text)]' : 'text-[var(--color-text-muted)]';
	}

	function asCanonicalChildren(nodes: readonly unknown[] | undefined): readonly CanonicalUiNode[] {
		return (nodes ?? []) as readonly CanonicalUiNode[];
	}

	function actionKey(action: ActionNode, index: number): string | number {
		return (action as ActionNode & { _key: string })._key;
	}

	function inputValue(node: InputNode) {
		return resolvePrimitiveValue(node.value, uiState, '') ?? '';
	}

	function inputType(kind: InputNode['inputKind']) {
		return kind === 'number' ||
			kind === 'password' ||
			kind === 'email' ||
			kind === 'url' ||
			kind === 'date'
			? kind
			: 'text';
	}

	function updateInputValue(name: string, value: unknown) {
		uiState.set(name, value as JsonValue);
	}

	async function handleSubmit(event: SubmitEvent, submitAction?: ActionRef) {
		event.preventDefault();
		if (!submitAction) return;
		await runAction(submitAction);
	}
</script>

{#if node.kind === 'screen'}
	<section class="space-y-[var(--gap-4)]">
		<CoreChildren nodes={asCanonicalChildren(node.children)} />
	</section>
{:else if node.kind === 'section'}
	<section class="space-y-[var(--gap-3)]">
		{#if node.title}
			<h2 class="text-[length:var(--fs-xs)] font-black uppercase tracking-[0.2em] text-[var(--color-text-muted)]">
				{resolveStringValue(node.title, uiState)}
			</h2>
		{/if}
		{#if node.description}
			<p class="text-[length:var(--fs-xs)] text-[var(--color-text-muted)]">
				{resolveStringValue(node.description, uiState)}
			</p>
		{/if}
		<div class="space-y-[var(--gap-3)]">
			<CoreChildren nodes={asCanonicalChildren(node.children)} />
		</div>
	</section>
{:else if node.kind === 'stack'}
	<div class="flex flex-col" style={`gap:${gapValue(node.gap)};`}>
		<CoreChildren nodes={asCanonicalChildren(node.children)} />
	</div>
{:else if node.kind === 'inline'}
	<div
		class={`flex ${node.wrap ? 'flex-wrap' : 'flex-nowrap'} ${node.align === 'center' ? 'items-center' : node.align === 'end' ? 'items-end' : 'items-start'}`}
		style={`gap:${gapValue(node.gap)};`}
	>
		<CoreChildren nodes={asCanonicalChildren(node.children)} />
	</div>
{:else if node.kind === 'grid'}
	<CoreGridNode {node} />
{:else if node.kind === 'scroll'}
	<div class="overflow-auto">
		<CoreChildren nodes={asCanonicalChildren(node.children)} />
	</div>
{:else if node.kind === 'text'}
	<p
		class={[
			textRoleClass(node.role, node.tone, node.emphasis),
			node.truncate ? 'overflow-hidden text-ellipsis' : ''
		]}
		style={truncateStyle(node.role, node.truncate)}
	>
		{resolveStringValue(node.content, uiState)}
	</p>
{:else if node.kind === 'value'}
	<p class={textRoleClass(node.role, node.tone, undefined)}>{formatValue(node)}</p>
{:else if node.kind === 'icon'}
	<span
		class="inline-flex items-center text-[length:var(--fs-xs)] text-[var(--color-text-muted)]"
		aria-label={node.label}
		title={node.label}
	>
		{node.name}
	</span>
{:else if node.kind === 'badge'}
	<span
		class={`inline-flex items-center rounded-[var(--radius-lg)] px-[var(--space-2)] py-[var(--space-1)] text-[length:var(--fs-2xs)] font-black uppercase tracking-[0.18em] ${badgeToneClass(node.tone)}`}
	>
		{resolveStringValue(node.label, uiState)}
	</span>
{:else if node.kind === 'divider'}
	<hr class="border-[length:var(--border-w)] border-[var(--color-border)]" />
{:else if node.kind === 'media'}
	<CoreMediaNode {node} />
{:else if node.kind === 'pressable'}
	<CorePressableNode {node} />
{:else if node.kind === 'item'}
	<CoreItemNode {node} />
{:else if node.kind === 'list'}
	<CoreListNode {node} />
{:else if node.kind === 'action'}
	<CoreActionNode {node} />
{:else if node.kind === 'actions'}
	<div class={`flex flex-wrap gap-[var(--gap-2)] ${actionsAlignClass(node.align)}`}>
		{#each node.children as action, index (actionKey(action, index))}
			<CoreActionNode node={action} />
		{/each}
	</div>
{:else if node.kind === 'disclosure'}
	<CoreDisclosureNode {node} />
{:else if node.kind === 'menu'}
	<CoreMenuNode {node} />
{:else if node.kind === 'input'}
	<div class="space-y-[var(--gap-2)]">
		<label class="block text-[length:var(--fs-xs)] font-black uppercase tracking-[0.18em] text-[var(--color-text-muted)]">
			<span>{resolveStringValue(node.label, uiState)}</span>
			{#if node.inputKind === 'textarea'}
				<textarea
					id={node.id}
					class="mt-[var(--space-2)] min-h-[8rem] w-full rounded-[var(--radius-none)] border-[length:var(--border-w-heavy)] border-[var(--color-border-strong)] bg-[var(--color-surface-1)] px-[var(--space-3)] py-[var(--space-2)] text-[length:var(--fs-sm)] text-[var(--color-text)]"
					name={node.name}
					placeholder={node.placeholder ? resolveStringValue(node.placeholder, uiState) : undefined}
					required={node.required}
					disabled={node.disabled ? resolveBooleanValue(node.disabled, uiState) : false}
					oninput={(event) => updateInputValue(node.name, (event.currentTarget as HTMLTextAreaElement).value)}
				>{String(inputValue(node))}</textarea>
			{:else if node.inputKind === 'boolean'}
				<input
					id={node.id}
					class="mt-[var(--space-2)] h-[var(--space-4)] w-[var(--space-4)] accent-[var(--color-info)]"
					type="checkbox"
					name={node.name}
					checked={Boolean(inputValue(node))}
					disabled={node.disabled ? resolveBooleanValue(node.disabled, uiState) : false}
					onchange={(event) => updateInputValue(node.name, (event.currentTarget as HTMLInputElement).checked)}
				/>
			{:else if node.inputKind === 'select'}
				<select
					id={node.id}
					class="mt-[var(--space-2)] w-full rounded-[var(--radius-none)] border-[length:var(--border-w-heavy)] border-[var(--color-border-strong)] bg-[var(--color-surface-1)] px-[var(--space-3)] py-[var(--space-2)] text-[length:var(--fs-sm)] text-[var(--color-text)]"
					name={node.name}
					disabled={node.disabled ? resolveBooleanValue(node.disabled, uiState) : false}
					onchange={(event) => updateInputValue(node.name, (event.currentTarget as HTMLSelectElement).value)}
				>
					{#each node.options ?? [] as option (String(option.value))}
						<option value={String(option.value)} selected={String(inputValue(node)) === String(option.value)} disabled={option.disabled}>
							{option.label}
						</option>
					{/each}
				</select>
			{:else}
				<input
					id={node.id}
					class="mt-[var(--space-2)] w-full rounded-[var(--radius-none)] border-[length:var(--border-w-heavy)] border-[var(--color-border-strong)] bg-[var(--color-surface-1)] px-[var(--space-3)] py-[var(--space-2)] text-[length:var(--fs-sm)] text-[var(--color-text)]"
					type={inputType(node.inputKind)}
					name={node.name}
					value={String(inputValue(node))}
					placeholder={node.placeholder ? resolveStringValue(node.placeholder, uiState) : undefined}
					required={node.required}
					disabled={node.disabled ? resolveBooleanValue(node.disabled, uiState) : false}
					oninput={(event) => updateInputValue(node.name, (event.currentTarget as HTMLInputElement).value)}
				/>
			{/if}
		</label>
		{#if node.helpText}
			<p class="text-[length:var(--fs-xs)] text-[var(--color-text-muted)]">
				{resolveStringValue(node.helpText, uiState)}
			</p>
		{/if}
	</div>
{:else if node.kind === 'form'}
	<form class="space-y-[var(--gap-3)]" onsubmit={(event) => handleSubmit(event, node.submit)}>
		<CoreChildren nodes={asCanonicalChildren(node.children)} />
	</form>
{:else if node.kind === 'status'}
	<div class={`rounded-[var(--radius-xl)] border-[length:var(--border-w-heavy)] bg-[var(--color-bg)] p-[var(--space-4)] ${statusClasses(node.severity)}`}>
		{#if node.title}
			<div class="text-[length:var(--fs-xs)] font-black uppercase tracking-[0.18em]">
				{resolveStringValue(node.title, uiState)}
			</div>
		{/if}
		<div class="mt-[var(--space-2)] text-[length:var(--fs-sm)] font-semibold text-[var(--color-text)]">
			{resolveStringValue(node.message, uiState)}
		</div>
		{#if node.actions?.length}
			<div class="mt-[var(--space-3)] flex flex-wrap gap-[var(--gap-2)]">
				{#each node.actions as action, index (actionKey(action, index))}
					<CoreActionNode node={action} />
				{/each}
			</div>
		{/if}
	</div>
{:else if node.kind === 'empty'}
	<div class="rounded-[var(--radius-xl)] border-[length:var(--border-w-emphasis)] border-dashed border-[var(--color-border-strong)] bg-[color-mix(in_srgb,var(--color-bg)_70%,transparent)] p-[var(--space-6)] text-center">
		<div class={`text-[length:var(--fs-sm)] font-black uppercase tracking-[0.2em] ${emptyToneClasses(node)}`}>
			{resolveStringValue(node.title, uiState)}
		</div>
		{#if node.message}
			<div class="mt-[var(--space-2)] text-[length:var(--fs-xs)] text-[var(--color-text-muted)]">
				{resolveStringValue(node.message, uiState)}
			</div>
		{/if}
		{#if node.actions?.length}
			<div class="mt-[var(--space-4)] flex flex-wrap justify-center gap-[var(--gap-2)]">
				{#each node.actions as action, index (actionKey(action, index))}
					<CoreActionNode node={action} />
				{/each}
			</div>
		{/if}
	</div>
{:else if node.kind === 'loading'}
	<div class="rounded-[var(--radius-none)] border-[length:var(--border-w-heavy)] border-dashed border-[var(--color-border-strong)] bg-[var(--color-surface-1)] px-[var(--space-4)] py-[var(--space-3)] text-[length:var(--fs-xs)] font-black uppercase tracking-[0.18em] text-[var(--color-text-muted)]">
		{node.label ? resolveStringValue(node.label, uiState) : 'Loading...'}
		{#if node.progress !== undefined}
			<span class="ml-[var(--space-2)]">{Math.round(resolveNumberValue(node.progress, uiState, 0))}%</span>
		{/if}
	</div>
{:else if node.kind === 'conditional'}
	{#if resolveBooleanValue(node.condition, uiState)}
		<CoreChildren nodes={[node.then as CanonicalUiNode]} />
	{:else if node.else}
		<CoreChildren nodes={[node.else as CanonicalUiNode]} />
	{/if}
{:else if node.kind === 'slot'}
	{#if node.fallback}
		<CoreChildren nodes={[node.fallback as CanonicalUiNode]} />
	{/if}
{:else}
	<div class="text-[length:var(--fs-xs)] font-black uppercase tracking-[0.18em] text-[var(--color-danger)]">
		Unsupported node
	</div>
{/if}
