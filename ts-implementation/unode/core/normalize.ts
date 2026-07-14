import { deepFreeze } from './immutable';
import type { Immutable } from './immutable';
import type {
	ActionNode,
	ActionsNode,
	BadgeNode,
	BoolOrExpr,
	ConditionalNode,
	DisclosureNode,
	DividerNode,
	EmptyStateNode,
	FormNode,
	GridNode,
	IconNode,
	InlineNode,
	InputNode,
	ItemNode,
	ListNode,
	LoadingNode,
	MenuItem,
	MenuNode,
	MediaNode,
	NumberOrExpr,
	PressableNode,
	Primitive,
	PrimitiveOrExpr,
	RootNode,
	ScreenNode,
	ScrollNode,
	SectionNode,
	SlotNode,
	StackNode,
	StatusNode,
	StringOrExpr,
	TextNode,
	UiExpr,
	UiNode,
	ValueNode
} from './ast';
import type { ExprResolver, ResolverContext, Unsubscribe } from './runtime';

export type AuthorUiNode = RootNode;
export type NodeReactivity = 'static' | 'reactive' | 'conditional';

export type CanonicalMetadata = Readonly<{
	_key: string;
	_reactivity: NodeReactivity;
	_subtreeReactivity: NodeReactivity;
	_staticFields: Readonly<Record<string, Primitive>>;
}>;

export type CanonicalNode<T extends AuthorUiNode = AuthorUiNode> = Immutable<T & CanonicalMetadata>;
export type CanonicalUiNode = CanonicalNode<UiNode>;
export type CanonicalRootNode = CanonicalNode<RootNode>;
export type TransportUiNode = CanonicalUiNode;
export type CanonicalScreen = CanonicalNode<ScreenNode>;
export type TransportScreen = CanonicalScreen;

// ─────────────────────────────────────────────────────────────────────────────
// Reactive subscription result
//
// Returned by trackReactiveBindings(). Renderers use pathToNodes to know which
// node keys to re-evaluate when a state path changes, without walking the tree.
// Call teardown() when unmounting the screen to release all subscriptions.
// ─────────────────────────────────────────────────────────────────────────────

export type BindingSubscriptions = {
	/**
	 * path → set of node _keys that read that path.
	 *
	 * Built by walking the canonical tree once and resolving every UiExpr
	 * with tracking active. Renderers subscribe to each path and use this
	 * map to find the minimal set of nodes to re-evaluate on change.
	 *
	 * Note: a change at "work.title" also wakes nodes subscribed to "work"
	 * (ancestor prefix match). The resolver handles this via subscribersOf().
	 */
	readonly pathToNodes: ReadonlyMap<string, ReadonlySet<string>>;

	/** Unsubscribe all StateStore subscriptions created by trackReactiveBindings(). */
	teardown: Unsubscribe;
};

type NormalizeContext = {
	path: string;
	crumbs: ReadonlyArray<string>;
	seenIds: Map<string, string>;
};

// ─────────────────────────────────────────────────────────────────────────────
// Dev mode detection — portable across Vite (web), Deno, and Node.
// Warnings are emitted only outside production builds.
// ─────────────────────────────────────────────────────────────────────────────

function isDev(): boolean {
	// Vite / SvelteKit
	if (typeof import.meta !== 'undefined' && (import.meta as unknown as Record<string, unknown>).env) {
		return Boolean((import.meta as { env: { DEV?: boolean } }).env.DEV);
	}
	// Node / Deno with process
	if (typeof process !== 'undefined' && process.env) {
		return process.env['NODE_ENV'] !== 'production';
	}
	// Unknown environment — assume dev to surface warnings
	return true;
}

function freezeCanonical<T extends AuthorUiNode>(
	value: T,
	reaction: NodeReactivity,
	subtreeReaction: NodeReactivity,
	ctx: NormalizeContext
): CanonicalNode<T> {
	// Identity resolution:
	//   1. Explicit id   — semantic/a11y identity, also used for reconciliation.
	//                      Becomes the DOM element id. Must be globally unique.
	//   2. Structural fallback — ctx.path, stable within one load cycle.
	//      Safe for static structures that never change position between
	//      state updates. Dynamic collections (list items, grid children with
	//      continuation) should always supply explicit ids.
	const identity = value.id ?? ctx.path;

	if (value.id) {
		const firstLocation = ctx.seenIds.get(value.id);
		if (firstLocation) {
			throw new Error(
				`unode duplicate global id "${value.id}" at ${formatLocation(ctx)}; ids must be unique across the whole tree (first seen at ${firstLocation})`
			);
		}
		ctx.seenIds.set(value.id, formatLocation(ctx));
	}

	return deepFreeze({
		...value,
		_key: identity,
		_reactivity: reaction,
		_subtreeReactivity: subtreeReaction,
		_staticFields: collectStaticFields(value as unknown as Record<string, unknown>)
	}) as CanonicalNode<T>;
}

function rootContext(): NormalizeContext {
	return {
		path: 'screen',
		crumbs: ['screen'],
		seenIds: new Map()
	};
}

function childContext(
	parent: NormalizeContext,
	segment: string,
	index: number,
	kind: string
): NormalizeContext {
	return {
		path: `${parent.path}.${segment}${index}`,
		crumbs: [...parent.crumbs, `${kind}[${index}]`],
		seenIds: parent.seenIds
	};
}

function formatLocation(ctx: NormalizeContext): string {
	return `"${ctx.crumbs.join(' > ')}" (${ctx.path})`;
}

function isExpr(value: unknown): value is UiExpr {
	return Boolean(value && typeof value === 'object' && 'kind' in value);
}

function isReactiveExpr(value: unknown): boolean {
	return isExpr(value) && (value.kind === 'binding' || value.kind === 'param');
}

function collapseStringLiteral(value: StringOrExpr | undefined): StringOrExpr | undefined {
	return isExpr(value) && value.kind === 'literal' ? value.value : value;
}

function collapseRequiredStringLiteral(value: StringOrExpr): StringOrExpr {
	return collapseStringLiteral(value) ?? value;
}

function collapseBooleanLiteral(value: BoolOrExpr | undefined): BoolOrExpr | undefined {
	return isExpr(value) && value.kind === 'literal' ? value.value : value;
}

function collapseRequiredBooleanLiteral(value: BoolOrExpr): BoolOrExpr {
	return collapseBooleanLiteral(value) ?? value;
}

function collapseNumberLiteral(value: NumberOrExpr | undefined): NumberOrExpr | undefined {
	return isExpr(value) && value.kind === 'literal' ? value.value : value;
}

function collapsePrimitiveLiteral(value: PrimitiveOrExpr | undefined): PrimitiveOrExpr | undefined {
	return isExpr(value) && value.kind === 'literal' ? value.value : value;
}

function collapseRequiredPrimitiveLiteral(value: PrimitiveOrExpr): PrimitiveOrExpr {
	return collapsePrimitiveLiteral(value) ?? value;
}

function collectStaticFields(value: Record<string, unknown>): Readonly<Record<string, Primitive>> {
	const out: Record<string, Primitive> = {};

	for (const [key, entry] of Object.entries(value)) {
		if (
			key === 'key' ||
			key === 'id' ||
			key === 'meta' ||
			key === 'children' ||
			key === 'child' ||
			key === 'leading' ||
			key === 'primary' ||
			key === 'secondary' ||
			key === 'trailing' ||
			key === 'items' ||
			key === 'then' ||
			key === 'else' ||
			key === 'fallback' ||
			key === 'actions'
		) {
			continue;
		}

		if (
			entry === null ||
			typeof entry === 'string' ||
			typeof entry === 'number' ||
			typeof entry === 'boolean'
		) {
			out[key] = entry;
		}
	}

	return out;
}

function combineReactivity(...values: boolean[]): NodeReactivity {
	return values.some(Boolean) ? 'reactive' : 'static';
}

function mergeReactivity(...values: Array<NodeReactivity | undefined>): NodeReactivity {
	if (values.some((value) => value === 'conditional')) return 'conditional';
	if (values.some((value) => value === 'reactive')) return 'reactive';
	return 'static';
}

function subtreeReactivityOf(node: { _subtreeReactivity: NodeReactivity } | undefined): NodeReactivity | undefined {
	return node?._subtreeReactivity;
}

function subtreeReactivityOfAll(
	nodes: ReadonlyArray<{ _subtreeReactivity: NodeReactivity }> | undefined
): NodeReactivity | undefined {
	return mergeReactivity(...(nodes ?? []).map((node) => node._subtreeReactivity));
}

function assertUniqueSiblingIds(
	nodes: ReadonlyArray<{ _key: string; kind: string } | undefined> | undefined,
	ctx: NormalizeContext,
	group: string
) {
	if (!nodes) return;
	const seen = new Set<string>();

	for (const [index, node] of nodes.entries()) {
		if (!node) continue;
		if (seen.has(node._key)) {
			throw new Error(
				`unode duplicate sibling identity "${node._key}" in ${group} at "${ctx.crumbs.join(' > ')} > ${node.kind}[${index}]" (${ctx.path}); sibling ids must be unique`
			);
		}
		seen.add(node._key);
	}
}

function assertUniqueMenuItemIds(items: ReadonlyArray<MenuItem>, ctx: NormalizeContext) {
	const seen = new Set<string>();

	for (const [index, item] of items.entries()) {
		const itemId = item.id ?? `${ctx.path}.mi${index}`;
		if (seen.has(itemId)) {
			throw new Error(
				`unode duplicate menu item identity "${itemId}" at ${formatLocation(ctx)}; menu item ids must be unique`
			);
		}
		seen.add(itemId);
	}
}

function normalizeText(node: TextNode, ctx: NormalizeContext): CanonicalNode<TextNode> {
	const content = collapseRequiredStringLiteral(node.content);
	const selfReactivity = combineReactivity(isReactiveExpr(content));
	return freezeCanonical(
		{
			...node,
			content,
			role: node.role ?? 'body',
			emphasis: node.emphasis ?? 'normal'
		},
		selfReactivity,
		selfReactivity,
		ctx
	);
}

function normalizeMedia(node: MediaNode, ctx: NormalizeContext): CanonicalNode<MediaNode> {
	return freezeCanonical(
		{
			...node,
			aspectRatio: node.aspectRatio ?? 'auto'
		},
		'static',
		'static',
		ctx
	);
}

function normalizeStack(node: StackNode, ctx: NormalizeContext): CanonicalNode<StackNode> {
	const children = node.children.map((child, index) =>
		normalizeNode(child, childContext(ctx, 'c', index, child.kind))
	);
	assertUniqueSiblingIds(children, ctx, 'children');
	const selfReactivity: NodeReactivity = 'static';
	return freezeCanonical(
		{
			...node,
			gap: node.gap ?? 'md',
			children
		},
		selfReactivity,
		mergeReactivity(selfReactivity, subtreeReactivityOfAll(children)),
		ctx
	);
}

function normalizeInline(node: InlineNode, ctx: NormalizeContext): CanonicalNode<InlineNode> {
	const children = node.children.map((child, index) =>
		normalizeNode(child, childContext(ctx, 'c', index, child.kind))
	);
	assertUniqueSiblingIds(children, ctx, 'children');
	const selfReactivity: NodeReactivity = 'static';
	return freezeCanonical(
		{
			...node,
			gap: node.gap ?? 'sm',
			wrap: node.wrap ?? false,
			align: node.align ?? 'start',
			children
		},
		selfReactivity,
		mergeReactivity(selfReactivity, subtreeReactivityOfAll(children)),
		ctx
	);
}

function normalizeGrid(node: GridNode, ctx: NormalizeContext): CanonicalNode<GridNode> {
	// Dev warning: dynamic grids need stable ids on children for correct reconciliation.
	// When continuation is present the grid is treated as a dynamic collection —
	// children can be added/removed and the renderer must track identity.
	// This cannot be enforced at the type level without restricting GridNode's
	// children type, so we warn at normalization time instead.
	if (isDev() && node.continuation) {
		const missing = node.children.filter((c) => !c.id);
		if (missing.length > 0) {
			console.warn(
				`[unode] GridNode at ${formatLocation(ctx)} has continuation but ${missing.length} child(ren) without explicit id. ` +
				`Assign ids derived from your data (e.g. work.id) for stable reconciliation. ` +
				`Structural fallback ids will change if items are reordered or removed.`
			);
		}
	}

	const columns = node.columns ?? (node.maxColumns ? { base: node.maxColumns } : undefined);
	const continuation =
		node.continuation?.kind === 'incremental'
			? {
					...node.continuation,
					label: collapseStringLiteral(node.continuation.label)
				}
			: node.continuation?.kind === 'remote'
				? {
						...node.continuation,
						label: collapseStringLiteral(node.continuation.label),
						loadingLabel: collapseStringLiteral(node.continuation.loadingLabel)
					}
				: undefined;

	const children = node.children.map((child, index) =>
		normalizeNode(child, childContext(ctx, 'c', index, child.kind))
	);
	assertUniqueSiblingIds(children, ctx, 'children');
	const selfReactivity = combineReactivity(
		isReactiveExpr(node.continuation?.kind === 'incremental' ? node.continuation.label : undefined),
		isReactiveExpr(node.continuation?.kind === 'remote' ? node.continuation.label : undefined),
		isReactiveExpr(node.continuation?.kind === 'remote' ? node.continuation.loadingLabel : undefined)
	);

	return freezeCanonical(
		{
			...node,
			gap: node.gap ?? 'md',
			columns: columns
				? {
						base: columns.base ?? node.maxColumns ?? 1,
						sm: columns.sm,
						md: columns.md,
						lg: columns.lg,
						xl: columns.xl
					}
				: undefined,
			continuation,
			children
		},
		selfReactivity,
		mergeReactivity(selfReactivity, subtreeReactivityOfAll(children)),
		ctx
	);
}

function normalizeScroll(node: ScrollNode, ctx: NormalizeContext): CanonicalNode<ScrollNode> {
	const children = node.children.map((child, index) =>
		normalizeNode(child, childContext(ctx, 'c', index, child.kind))
	);
	assertUniqueSiblingIds(children, ctx, 'children');
	const selfReactivity: NodeReactivity = 'static';
	return freezeCanonical(
		{
			...node,
			children
		},
		selfReactivity,
		mergeReactivity(selfReactivity, subtreeReactivityOfAll(children)),
		ctx
	);
}

function normalizeItem(node: ItemNode, ctx: NormalizeContext): CanonicalNode<ItemNode> {
	const leading = node.leading?.map((child, index) =>
		normalizeNode(child, childContext(ctx, 'l', index, child.kind))
	);
	const primary = node.primary.map((child, index) =>
		normalizeNode(child, childContext(ctx, 'p', index, child.kind))
	);
	const secondary = node.secondary?.map((child, index) =>
		normalizeNode(child, childContext(ctx, 's', index, child.kind))
	);
	const trailing = node.trailing?.map((child, index) =>
		normalizeNode(child, childContext(ctx, 't', index, child.kind))
	);
	assertUniqueSiblingIds(leading, ctx, 'leading');
	assertUniqueSiblingIds(primary, ctx, 'primary');
	assertUniqueSiblingIds(secondary, ctx, 'secondary');
	assertUniqueSiblingIds(trailing, ctx, 'trailing');
	const selfReactivity: NodeReactivity = 'static';
	return freezeCanonical(
		{
			...node,
			leading,
			primary,
			secondary,
			trailing
		},
		selfReactivity,
		mergeReactivity(
			selfReactivity,
			subtreeReactivityOfAll(leading),
			subtreeReactivityOfAll(primary),
			subtreeReactivityOfAll(secondary),
			subtreeReactivityOfAll(trailing)
		),
		ctx
	);
}

function normalizeList(node: ListNode, ctx: NormalizeContext): CanonicalNode<ListNode> {
	const continuation =
		node.continuation?.kind === 'incremental'
			? {
					...node.continuation,
					label: collapseStringLiteral(node.continuation.label)
				}
			: node.continuation?.kind === 'remote'
				? {
						...node.continuation,
						label: collapseStringLiteral(node.continuation.label),
						loadingLabel: collapseStringLiteral(node.continuation.loadingLabel)
					}
				: undefined;

	const items = node.items.map((item, index) => normalizeItem(item, childContext(ctx, 'i', index, item.kind)));
	assertUniqueSiblingIds(items, ctx, 'items');
	const selfReactivity = combineReactivity(
		isReactiveExpr(node.continuation?.kind === 'incremental' ? node.continuation.label : undefined),
		isReactiveExpr(node.continuation?.kind === 'remote' ? node.continuation.label : undefined),
		isReactiveExpr(node.continuation?.kind === 'remote' ? node.continuation.loadingLabel : undefined)
	);

	return freezeCanonical(
		{
			...node,
			density: node.density ?? 'normal',
			continuation,
			items
		},
		selfReactivity,
		mergeReactivity(selfReactivity, subtreeReactivityOfAll(items)),
		ctx
	);
}

function normalizeAction(node: ActionNode, ctx: NormalizeContext): CanonicalNode<ActionNode> {
	const label = collapseRequiredStringLiteral(node.label);
	const disabled = collapseBooleanLiteral(node.disabled);
	const selfReactivity = combineReactivity(isReactiveExpr(label), isReactiveExpr(disabled));
	return freezeCanonical(
		{
			...node,
			label,
			disabled,
			intent: node.intent ?? 'secondary',
			variant: node.variant ?? 'button'
		},
		selfReactivity,
		selfReactivity,
		ctx
	);
}

function normalizeActions(node: ActionsNode, ctx: NormalizeContext): CanonicalNode<ActionsNode> {
	const children = node.children.map((child, index) =>
		normalizeAction(child, childContext(ctx, 'a', index, child.kind))
	);
	assertUniqueSiblingIds(children, ctx, 'actions');
	const selfReactivity: NodeReactivity = 'static';
	return freezeCanonical(
		{
			...node,
			align: node.align ?? 'start',
			children
		},
		selfReactivity,
		mergeReactivity(selfReactivity, subtreeReactivityOfAll(children)),
		ctx
	);
}

function normalizeDisclosure(node: DisclosureNode, ctx: NormalizeContext): CanonicalNode<DisclosureNode> {
	const label = collapseRequiredStringLiteral(node.label);
	const labelExpanded = collapseStringLiteral(node.labelExpanded);
	const children = node.children.map((child, index) =>
		normalizeNode(child, childContext(ctx, 'c', index, child.kind))
	);
	assertUniqueSiblingIds(children, ctx, 'children');
	const selfReactivity = combineReactivity(isReactiveExpr(label), isReactiveExpr(labelExpanded));
	return freezeCanonical(
		{
			...node,
			label,
			labelExpanded,
			children
		},
		selfReactivity,
		mergeReactivity(selfReactivity, subtreeReactivityOfAll(children)),
		ctx
	);
}

function normalizeMenuItem(item: MenuItem): Immutable<MenuItem> {
	return deepFreeze({
		...item,
		label: collapseStringLiteral(item.label),
		disabled: collapseBooleanLiteral(item.disabled),
		selected: item.selected ?? false
	}) as Immutable<MenuItem>;
}

function normalizeMenu(node: MenuNode, ctx: NormalizeContext): CanonicalNode<MenuNode> {
	const label = collapseRequiredStringLiteral(node.label);
	assertUniqueMenuItemIds(node.items, ctx);
	const items = node.items.map((item) => normalizeMenuItem(item));
	const selfReactivity = combineReactivity(
		isReactiveExpr(label),
		...items.map((item) => isReactiveExpr(item.label) || isReactiveExpr(item.disabled))
	);
	return freezeCanonical(
		{
			...node,
			label,
			intent: node.intent ?? 'secondary',
			align: node.align ?? 'start',
			items
		},
		selfReactivity,
		selfReactivity,
		ctx
	);
}

function normalizeSection(node: SectionNode, ctx: NormalizeContext): CanonicalNode<SectionNode> {
	const title = collapseStringLiteral(node.title);
	const description = collapseStringLiteral(node.description);
	const children = node.children.map((child, index) =>
		normalizeNode(child, childContext(ctx, 'c', index, child.kind))
	);
	assertUniqueSiblingIds(children, ctx, 'children');
	const selfReactivity = combineReactivity(isReactiveExpr(title), isReactiveExpr(description));
	return freezeCanonical(
		{
			...node,
			title,
			description,
			role: node.role ?? 'section',
			children
		},
		selfReactivity,
		mergeReactivity(selfReactivity, subtreeReactivityOfAll(children)),
		ctx
	);
}

function normalizeScreenNode(node: ScreenNode, ctx: NormalizeContext): CanonicalScreen {
	const title = collapseStringLiteral(node.title);
	const subtitle = collapseStringLiteral(node.subtitle);
	const children = node.children.map((child, index) =>
		normalizeNode(child, childContext(ctx, 'c', index, child.kind))
	);
	assertUniqueSiblingIds(children, ctx, 'children');
	const selfReactivity = combineReactivity(isReactiveExpr(title), isReactiveExpr(subtitle));
	return freezeCanonical(
		{
			...node,
			title,
			subtitle,
			children
		},
		selfReactivity,
		mergeReactivity(selfReactivity, subtreeReactivityOfAll(children)),
		ctx
	) as CanonicalScreen;
}

function normalizeStatus(node: StatusNode, ctx: NormalizeContext): CanonicalNode<StatusNode> {
	const title = collapseStringLiteral(node.title);
	const message = collapseRequiredStringLiteral(node.message);
	const actions = node.actions?.map((action, index) =>
		normalizeAction(action, childContext(ctx, 'a', index, action.kind))
	);
	assertUniqueSiblingIds(actions, ctx, 'actions');
	const selfReactivity = combineReactivity(isReactiveExpr(title), isReactiveExpr(message));
	return freezeCanonical(
		{
			...node,
			title,
			message,
			actions
		},
		selfReactivity,
		mergeReactivity(selfReactivity, subtreeReactivityOfAll(actions)),
		ctx
	);
}

function normalizeEmpty(node: EmptyStateNode, ctx: NormalizeContext): CanonicalNode<EmptyStateNode> {
	const title = collapseRequiredStringLiteral(node.title);
	const message = collapseStringLiteral(node.message);
	const actions = node.actions?.map((action, index) =>
		normalizeAction(action, childContext(ctx, 'a', index, action.kind))
	);
	assertUniqueSiblingIds(actions, ctx, 'actions');
	const selfReactivity = combineReactivity(isReactiveExpr(title), isReactiveExpr(message));
	return freezeCanonical(
		{
			...node,
			title,
			message,
			actions
		},
		selfReactivity,
		mergeReactivity(selfReactivity, subtreeReactivityOfAll(actions)),
		ctx
	);
}

function normalizeForm(node: FormNode, ctx: NormalizeContext): CanonicalNode<FormNode> {
	const children = node.children.map((child, index) =>
		normalizeNode(child, childContext(ctx, 'c', index, child.kind))
	);
	assertUniqueSiblingIds(children, ctx, 'children');
	const selfReactivity: NodeReactivity = 'static';
	return freezeCanonical(
		{
			...node,
			children
		},
		selfReactivity,
		mergeReactivity(selfReactivity, subtreeReactivityOfAll(children)),
		ctx
	);
}

function normalizeConditional(node: ConditionalNode, ctx: NormalizeContext): CanonicalNode<ConditionalNode> {
	const condition = collapseRequiredBooleanLiteral(node.condition);
	const thenNode = normalizeNode(node.then, childContext(ctx, 'then', 0, node.then.kind));
	const elseNode = node.else ? normalizeNode(node.else, childContext(ctx, 'else', 0, node.else.kind)) : undefined;
	const selfReactivity = isReactiveExpr(condition) ? 'conditional' : 'static';
	return freezeCanonical(
		{
			...node,
			condition,
			then: thenNode,
			else: elseNode
		},
		selfReactivity,
		mergeReactivity(selfReactivity, subtreeReactivityOf(thenNode), subtreeReactivityOf(elseNode)),
		ctx
	);
}

function normalizeSlot(node: SlotNode, ctx: NormalizeContext): CanonicalNode<SlotNode> {
	const fallback = node.fallback
		? normalizeNode(node.fallback, childContext(ctx, 'fb', 0, node.fallback.kind))
		: undefined;
	const selfReactivity: NodeReactivity = 'static';
	return freezeCanonical(
		{
			...node,
			fallback
		},
		selfReactivity,
		mergeReactivity(selfReactivity, subtreeReactivityOf(fallback)),
		ctx
	);
}

function normalizeValue(node: ValueNode, ctx: NormalizeContext): CanonicalNode<ValueNode> {
	const value = collapseRequiredPrimitiveLiteral(node.value);
	const selfReactivity = combineReactivity(isReactiveExpr(value));
	return freezeCanonical(
		{
			...node,
			value
		},
		selfReactivity,
		selfReactivity,
		ctx
	);
}

function normalizeBadge(node: BadgeNode, ctx: NormalizeContext): CanonicalNode<BadgeNode> {
	const label = collapseRequiredStringLiteral(node.label);
	const selfReactivity = combineReactivity(isReactiveExpr(label));
	return freezeCanonical(
		{
			...node,
			label
		},
		selfReactivity,
		selfReactivity,
		ctx
	);
}

function normalizePressable(node: PressableNode, ctx: NormalizeContext): CanonicalNode<PressableNode> {
	const label = collapseStringLiteral(node.label);
	const child = normalizeNode(node.child, childContext(ctx, 'child', 0, node.child.kind));
	const selfReactivity = combineReactivity(isReactiveExpr(label));
	return freezeCanonical(
		{
			...node,
			label,
			child
		},
		selfReactivity,
		mergeReactivity(selfReactivity, subtreeReactivityOf(child)),
		ctx
	);
}

function normalizeInput(node: InputNode, ctx: NormalizeContext): CanonicalNode<InputNode> {
	const label = collapseRequiredStringLiteral(node.label);
	const value = collapsePrimitiveLiteral(node.value);
	const placeholder = collapseStringLiteral(node.placeholder);
	const helpText = collapseStringLiteral(node.helpText);
	const disabled = collapseBooleanLiteral(node.disabled);
	const selfReactivity = combineReactivity(
		isReactiveExpr(label),
		isReactiveExpr(value),
		isReactiveExpr(placeholder),
		isReactiveExpr(helpText),
		isReactiveExpr(disabled)
	);

	return freezeCanonical(
		{
			...node,
			label,
			value,
			placeholder,
			helpText,
			disabled
		},
		selfReactivity,
		selfReactivity,
		ctx
	);
}

function normalizeLoading(node: LoadingNode, ctx: NormalizeContext): CanonicalNode<LoadingNode> {
	const label = collapseStringLiteral(node.label);
	const progress = collapseNumberLiteral(node.progress);
	const selfReactivity = combineReactivity(isReactiveExpr(label), isReactiveExpr(progress));
	return freezeCanonical(
		{
			...node,
			label,
			progress
		},
		selfReactivity,
		selfReactivity,
		ctx
	);
}

function normalizeIcon(node: IconNode, ctx: NormalizeContext): CanonicalNode<IconNode> {
	return freezeCanonical(node, 'static', 'static', ctx);
}

function normalizeDivider(node: DividerNode, ctx: NormalizeContext): CanonicalNode<DividerNode> {
	const label = collapseStringLiteral(node.label);
	const selfReactivity = combineReactivity(isReactiveExpr(label));
	return freezeCanonical(
		{
			...node,
			label
		},
		selfReactivity,
		selfReactivity,
		ctx
	);
}

export function normalizeNode(
	node: UiNode,
	ctx: NormalizeContext = rootContext()
): CanonicalUiNode {
	switch (node.kind) {
		case 'section':
			return normalizeSection(node, ctx);
		case 'stack':
			return normalizeStack(node, ctx);
		case 'inline':
			return normalizeInline(node, ctx);
		case 'grid':
			return normalizeGrid(node, ctx);
		case 'scroll':
			return normalizeScroll(node, ctx);
		case 'text':
			return normalizeText(node, ctx);
		case 'value':
			return normalizeValue(node, ctx);
		case 'icon':
			return normalizeIcon(node, ctx);
		case 'badge':
			return normalizeBadge(node, ctx);
		case 'divider':
			return normalizeDivider(node, ctx);
		case 'media':
			return normalizeMedia(node, ctx);
		case 'pressable':
			return normalizePressable(node, ctx);
		case 'item':
			return normalizeItem(node, ctx);
		case 'list':
			return normalizeList(node, ctx);
		case 'action':
			return normalizeAction(node, ctx);
		case 'actions':
			return normalizeActions(node, ctx);
		case 'disclosure':
			return normalizeDisclosure(node, ctx);
		case 'menu':
			return normalizeMenu(node, ctx);
		case 'input':
			return normalizeInput(node, ctx);
		case 'form':
			return normalizeForm(node, ctx);
		case 'status':
			return normalizeStatus(node, ctx);
		case 'empty':
			return normalizeEmpty(node, ctx);
		case 'loading':
			return normalizeLoading(node, ctx);
		case 'conditional':
			return normalizeConditional(node, ctx);
		case 'slot':
			return normalizeSlot(node, ctx);
		default: {
			const exhaustive: never = node;
			throw new Error(`Unknown node kind: ${String(exhaustive)}`);
		}
	}
}

export function normalizeScreen(screen: ScreenNode): CanonicalScreen {
	return normalizeScreenNode(screen, rootContext());
}

export function toTransportNode(node: UiNode): TransportUiNode {
	return normalizeNode(node);
}

export function toTransportScreen(screen: ScreenNode): TransportScreen {
	return normalizeScreen(screen);
}

// ─────────────────────────────────────────────────────────────────────────────
// Reactive binding tracker
//
// Walks the canonical tree once, resolves every UiExpr field with tracking
// active, and wires up StateStore subscriptions. Returns a BindingSubscriptions
// object the renderer uses to know which nodes to patch on state changes.
//
// Call this after normalizeScreen(), before mounting.
// Call teardown() when unmounting the screen.
//
// The resolver is not stored inside normalize — it is passed in so renderers
// can use their own ExprResolver instance (with full track/subscribersOf API)
// and share it across the screen lifecycle.
// ─────────────────────────────────────────────────────────────────────────────

/**
 * Walks `screen` once, calling resolver.track() for every binding or param
 * expression found, then subscribes to the corresponding paths in `state`.
 *
 * `onPatch(nodeKeys)` is called by the subscription handler with the set of
 * node keys the renderer needs to re-evaluate. The renderer is responsible for
 * actually re-resolving and patching its output — this function only drives
 * the notification.
 *
 * Static subtrees (node._subtreeReactivity === 'static') are skipped entirely.
 * The walk does not descend into them.
 */
export function trackReactiveBindings(
	screen: CanonicalScreen,
	resolver: ExprResolver,
	ctx: ResolverContext,
	onPatch: (nodeKeys: ReadonlySet<string>) => void
): BindingSubscriptions {
	// Phase 1: walk the tree, resolve all expressions with tracking active.
	// After this, resolver.dependenciesOf(nodeKey) returns the paths each node reads.
	walkCanonical(screen, resolver, ctx);

	// Phase 2: invert the dependency map (path → nodeKeys) and subscribe.
	// We collect all unique paths first, then create one subscription per path.
	const allPaths = new Set<string>();
	collectAllTrackedPaths(screen, resolver, allPaths);

	// Build the pathToNodes map for renderer consumption
	const pathToNodes = new Map<string, Set<string>>();
	for (const path of allPaths) {
		const nodeKeys = new Set(resolver.subscribersOf(path));
		if (nodeKeys.size > 0) {
			pathToNodes.set(path, nodeKeys);
		}
	}

	// Phase 3: subscribe to each path in the StateStore
	const unsubs: Unsubscribe[] = [];
	for (const [path] of pathToNodes) {
		const unsub = ctx.state.subscribe(path, () => {
			// On state change: find all nodes that depend on this path
			// (including ancestors via subscribersOf prefix matching)
			const affected = new Set(resolver.subscribersOf(path));
			if (affected.size > 0) {
				onPatch(affected);
			}
		});
		unsubs.push(unsub);
	}

	return {
		pathToNodes,
		teardown: () => {
			for (const unsub of unsubs) unsub();
		}
	};
}

// ─────────────────────────────────────────────────────────────────────────────
// Internal tree walker for binding tracking
// ─────────────────────────────────────────────────────────────────────────────

function walkCanonical(
	node: CanonicalRootNode,
	resolver: ExprResolver,
	ctx: ResolverContext
): void {
	// Skip subtrees with no reactive bindings at all — the _subtreeReactivity
	// flag is the fast path that avoids descending into fully static branches.
	if (node._subtreeReactivity === 'static') return;

	// Resolve this node's own expressions with tracking active.
	// resolver.track(nodeKey, path) is called inside resolvePrimitive for bindings.
	if (node._reactivity !== 'static') {
		resolveNodeExpressions(node, resolver, ctx);
	}

	// Recurse into children
	switch (node.kind) {
		case 'screen':
		case 'section':
		case 'stack':
		case 'inline':
		case 'grid':
		case 'scroll':
		case 'form':
			for (const child of node.children) {
				walkCanonical(child as CanonicalUiNode, resolver, ctx);
			}
			break;

		case 'list':
			for (const item of node.items) {
				walkCanonical(item as CanonicalUiNode, resolver, ctx);
			}
			break;

		case 'item': {
			const itemNode = node as CanonicalNode<import('./ast').ItemNode>;
			for (const c of itemNode.leading ?? []) walkCanonical(c as CanonicalUiNode, resolver, ctx);
			for (const c of itemNode.primary) walkCanonical(c as CanonicalUiNode, resolver, ctx);
			for (const c of itemNode.secondary ?? []) walkCanonical(c as CanonicalUiNode, resolver, ctx);
			for (const c of itemNode.trailing ?? []) walkCanonical(c as CanonicalUiNode, resolver, ctx);
			break;
		}

		case 'pressable':
			walkCanonical(
				(node as CanonicalNode<import('./ast').PressableNode>).child as CanonicalUiNode,
				resolver,
				ctx
			);
			break;

		case 'actions':
		case 'status':
		case 'empty':
			for (const child of (node as CanonicalNode<ActionsNode>).children ?? []) {
				walkCanonical(child as CanonicalUiNode, resolver, ctx);
			}
			break;

		case 'disclosure':
			for (const child of (node as CanonicalNode<DisclosureNode>).children) {
				walkCanonical(child as CanonicalUiNode, resolver, ctx);
			}
			break;

		case 'conditional': {
			const condNode = node as CanonicalNode<ConditionalNode>;
			// Walk both branches — the resolver tracks both, and the renderer
			// will only evaluate the active branch but we need subscriptions for both
			// so that switching branches triggers re-evaluation.
			walkCanonical(condNode.then as CanonicalUiNode, resolver, ctx);
			if (condNode.else) {
				walkCanonical(condNode.else as CanonicalUiNode, resolver, ctx);
			}
			break;
		}

		case 'slot': {
			const slotNode = node as CanonicalNode<SlotNode>;
			if (slotNode.fallback) {
				walkCanonical(slotNode.fallback as CanonicalUiNode, resolver, ctx);
			}
			break;
		}

		// Leaf nodes: text, value, badge, icon, divider, media, action, input,
		// loading, menu — no children to recurse into. Expressions on these
		// nodes are handled by resolveNodeExpressions() above.
	}
}

/**
 * Resolves all UiExpr fields on a single node with tracking active.
 * The ExprResolver calls track(nodeKey, path) for each binding encountered.
 * We pass `node._key` as the nodeKey so the resolver builds the correct map.
 */
function resolveNodeExpressions(
	node: CanonicalRootNode,
	resolver: ExprResolver,
	ctx: ResolverContext
): void {
	const key = node._key;

	switch (node.kind) {
		case 'screen':
			if (node.title && isReactiveExpr(node.title)) resolver.resolvePrimitive(node.title, ctx, key);
			if (node.subtitle && isReactiveExpr(node.subtitle)) resolver.resolvePrimitive(node.subtitle, ctx, key);
			break;
		case 'section':
			if (node.title && isReactiveExpr(node.title)) resolver.resolvePrimitive(node.title, ctx, key);
			if (node.description && isReactiveExpr(node.description)) resolver.resolvePrimitive(node.description, ctx, key);
			break;
		case 'text':
			if (isReactiveExpr(node.content)) resolver.resolvePrimitive(node.content, ctx, key);
			break;
		case 'value':
			if (isReactiveExpr(node.value)) resolver.resolvePrimitive(node.value as UiExpr, ctx, key);
			break;
		case 'badge':
			if (isReactiveExpr(node.label)) resolver.resolvePrimitive(node.label, ctx, key);
			break;
		case 'divider':
			if (node.label && isReactiveExpr(node.label)) resolver.resolvePrimitive(node.label, ctx, key);
			break;
		case 'action':
			if (isReactiveExpr(node.label)) resolver.resolvePrimitive(node.label, ctx, key);
			if (node.disabled && isReactiveExpr(node.disabled)) resolver.resolvePrimitive(node.disabled, ctx, key);
			break;
		case 'input':
			if (isReactiveExpr(node.label)) resolver.resolvePrimitive(node.label, ctx, key);
			if (node.value && isReactiveExpr(node.value)) resolver.resolvePrimitive(node.value as UiExpr, ctx, key);
			if (node.placeholder && isReactiveExpr(node.placeholder)) resolver.resolvePrimitive(node.placeholder, ctx, key);
			if (node.helpText && isReactiveExpr(node.helpText)) resolver.resolvePrimitive(node.helpText, ctx, key);
			if (node.disabled && isReactiveExpr(node.disabled)) resolver.resolvePrimitive(node.disabled, ctx, key);
			break;
		case 'disclosure':
			if (isReactiveExpr(node.label)) resolver.resolvePrimitive(node.label, ctx, key);
			if (node.labelExpanded && isReactiveExpr(node.labelExpanded)) resolver.resolvePrimitive(node.labelExpanded, ctx, key);
			// binding is always reactive — track it explicitly
			resolver.track(key, node.binding);
			break;
		case 'conditional':
			if (isReactiveExpr(node.condition)) resolver.resolvePrimitive(node.condition, ctx, key);
			break;
		case 'pressable':
			if (node.label && isReactiveExpr(node.label)) resolver.resolvePrimitive(node.label, ctx, key);
			break;
		case 'menu':
			if (isReactiveExpr(node.label)) resolver.resolvePrimitive(node.label, ctx, key);
			for (const item of node.items) {
				if (item.label && isReactiveExpr(item.label)) resolver.resolvePrimitive(item.label, ctx, key);
				if (item.disabled && isReactiveExpr(item.disabled)) resolver.resolvePrimitive(item.disabled, ctx, key);
			}
			break;
		case 'status':
			if (node.title && isReactiveExpr(node.title)) resolver.resolvePrimitive(node.title, ctx, key);
			if (isReactiveExpr(node.message)) resolver.resolvePrimitive(node.message, ctx, key);
			break;
		case 'empty':
			if (isReactiveExpr(node.title)) resolver.resolvePrimitive(node.title, ctx, key);
			if (node.message && isReactiveExpr(node.message)) resolver.resolvePrimitive(node.message, ctx, key);
			break;
		case 'loading':
			if (node.label && isReactiveExpr(node.label)) resolver.resolvePrimitive(node.label, ctx, key);
			if (node.progress && isReactiveExpr(node.progress)) resolver.resolvePrimitive(node.progress as UiExpr, ctx, key);
			break;
		// grid/list continuation labels
		case 'grid':
		case 'list':
			if (node.continuation?.kind === 'incremental' && isReactiveExpr(node.continuation.label)) {
				resolver.resolvePrimitive(node.continuation.label!, ctx, key);
			}
			if (node.continuation?.kind === 'remote') {
				if (isReactiveExpr(node.continuation.label)) resolver.resolvePrimitive(node.continuation.label!, ctx, key);
				if (isReactiveExpr(node.continuation.loadingLabel)) resolver.resolvePrimitive(node.continuation.loadingLabel!, ctx, key);
			}
			break;
	}
}

function collectAllTrackedPaths(
	node: CanonicalRootNode,
	resolver: ExprResolver,
	out: Set<string>
): void {
	if (node._subtreeReactivity === 'static') return;

	for (const path of resolver.dependenciesOf(node._key)) {
		out.add(path);
	}

	// Recurse — mirrors walkCanonical structure
	switch (node.kind) {
		case 'screen': case 'section': case 'stack': case 'inline':
		case 'grid': case 'scroll': case 'form': case 'actions': case 'disclosure':
			for (const child of (node as CanonicalNode<StackNode>).children) {
				collectAllTrackedPaths(child as CanonicalUiNode, resolver, out);
			}
			break;
		case 'list':
			for (const item of node.items) collectAllTrackedPaths(item as CanonicalUiNode, resolver, out);
			break;
		case 'item': {
			const n = node as CanonicalNode<import('./ast').ItemNode>;
			for (const c of [...(n.leading ?? []), ...n.primary, ...(n.secondary ?? []), ...(n.trailing ?? [])]) {
				collectAllTrackedPaths(c as CanonicalUiNode, resolver, out);
			}
			break;
		}
		case 'pressable':
			collectAllTrackedPaths(
				(node as CanonicalNode<import('./ast').PressableNode>).child as CanonicalUiNode,
				resolver, out
			);
			break;
		case 'conditional': {
			const n = node as CanonicalNode<ConditionalNode>;
			collectAllTrackedPaths(n.then as CanonicalUiNode, resolver, out);
			if (n.else) collectAllTrackedPaths(n.else as CanonicalUiNode, resolver, out);
			break;
		}
		case 'slot': {
			const n = node as CanonicalNode<SlotNode>;
			if (n.fallback) collectAllTrackedPaths(n.fallback as CanonicalUiNode, resolver, out);
			break;
		}
		case 'status': case 'empty':
			for (const child of (node as CanonicalNode<StatusNode>).actions ?? []) {
				collectAllTrackedPaths(child as CanonicalUiNode, resolver, out);
			}
			break;
	}
}