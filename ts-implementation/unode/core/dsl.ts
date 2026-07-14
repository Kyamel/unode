import { deepFreeze } from './immutable';
import type { Immutable } from './immutable';
import type {
	ActionNode,
	ActionRef,
	ActionVariant,
	ActionsNode,
	Align,
	AspectRatio,
	BadgeNode,
	BoolOrExpr,
	CollectionContinuation,
	ConditionalNode,
	ContainerRole,
	DisclosureNode,
	DividerNode,
	EmptyStateNode,
	FormNode,
	Gap,
	GridNode,
	IconNode,
	InputKind,
	InputNode,
	InlineNode,
	ItemNode,
	ListNode,
	LoadingNode,
	MenuItem,
	MenuNode,
	MediaNode,
	PressableNode,
	MediaRef,
	NodeBase,
	NumberOrExpr,
	Primitive,
	ResponsiveGridColumns,
	ScreenNode,
	ScrollNode,
	SectionNode,
	SelectOption,
	SlotNode,
	StackNode,
	StateValue,
	StatusNode,
	StringOrExpr,
	TextNode,
	TextRole,
	Tone,
	UiExpr,
	UiNode,
	ValueFormat,
	ValueNode
} from './ast';

function clean<T extends object>(obj: T): T {
	const out: Record<string, unknown> = {};
	for (const [key, value] of Object.entries(obj)) {
		if (value !== undefined) out[key] = value;
	}
	return out as T;
}

function freeze<T extends object>(obj: T): Immutable<T> {
	return deepFreeze(clean(obj));
}

/** Creates a literal expression node for immutable authored values. */
function literal<T extends Primitive>(value: T): UiExpr<T> {
	return deepFreeze({ kind: 'literal' as const, value }) as UiExpr<T>;
}

/** Creates a binding expression that resolves against the local screen state store. */
function binding<T extends Primitive = Primitive>(path: string): UiExpr<T> {
	return freeze({ kind: 'binding' as const, path });
}

/** Creates a param expression that resolves against route params or query values. */
function param<T extends Primitive = string>(name: string): UiExpr<T> {
	return freeze({ kind: 'param' as const, name });
}

/** Creates the root screen node returned from `render()`. */
function screen(
	opts: {
		title?: StringOrExpr;
		subtitle?: StringOrExpr;
		initialFocus?: string;
		initialState?: Record<string, StateValue>;
	} & NodeBase,
	children: UiNode[]
): Immutable<ScreenNode> {
	return freeze({ kind: 'screen' as const, ...opts, children });
}

/** Creates a semantic section container. */
function section(
	opts: {
		title?: StringOrExpr;
		description?: StringOrExpr;
		role?: ContainerRole;
	} & NodeBase,
	children: UiNode[]
): Immutable<SectionNode> {
	return freeze({ kind: 'section' as const, ...opts, children });
}

/** Creates a vertical stack layout container. */
function stack(children: UiNode[]): Immutable<StackNode>;
function stack(opts: { gap?: Gap } & NodeBase, children: UiNode[]): Immutable<StackNode>;
function stack(
	optsOrChildren: ({ gap?: Gap } & NodeBase) | UiNode[],
	children?: UiNode[]
): Immutable<StackNode> {
	if (Array.isArray(optsOrChildren)) {
		return freeze({ kind: 'stack' as const, children: optsOrChildren });
	}
	return freeze({ kind: 'stack' as const, ...optsOrChildren, children: children ?? [] });
}

/** Creates an inline layout container. */
function inline(children: UiNode[]): Immutable<InlineNode>;
function inline(
	opts: { gap?: Gap; wrap?: boolean; align?: Align } & NodeBase,
	children: UiNode[]
): Immutable<InlineNode>;
function inline(
	optsOrChildren: ({ gap?: Gap; wrap?: boolean; align?: Align } & NodeBase) | UiNode[],
	children?: UiNode[]
): Immutable<InlineNode> {
	if (Array.isArray(optsOrChildren)) {
		return freeze({ kind: 'inline' as const, children: optsOrChildren });
	}
	return freeze({ kind: 'inline' as const, ...optsOrChildren, children: children ?? [] });
}

/** Creates a responsive grid container. */
function grid(
	opts: {
		maxColumns?: 1 | 2 | 3 | 4 | 5 | 6;
		columns?: ResponsiveGridColumns;
		gap?: Gap;
		continuation?: CollectionContinuation;
	} & NodeBase,
	children: UiNode[]
): Immutable<GridNode> {
	return freeze({ kind: 'grid' as const, ...opts, children });
}

/** Creates a scrollable container. */
function scroll(children: UiNode[], base?: NodeBase): Immutable<ScrollNode> {
	return freeze({ kind: 'scroll' as const, ...base, children });
}

/** Creates a semantic text node. */
function text(
	content: StringOrExpr,
	opts?: { role?: TextRole; tone?: Tone; emphasis?: 'normal' | 'strong'; truncate?: boolean } & NodeBase
): Immutable<TextNode> {
	return freeze({ kind: 'text' as const, content, ...opts });
}

/** Creates a formatted scalar value node. */
function value(
	current: Primitive | UiExpr,
	format: ValueFormat,
	opts?: { currencyCode?: string; role?: TextRole; tone?: Tone } & NodeBase
): Immutable<ValueNode> {
	return freeze({ kind: 'value' as const, value: current, format, ...opts });
}

/** Creates a semantic icon node. */
function icon(name: string, label: string, opts?: { tone?: Tone } & NodeBase): Immutable<IconNode> {
	return freeze({ kind: 'icon' as const, name, label, ...opts });
}

/** Creates a compact badge/chip node. */
function badge(label: StringOrExpr, tone?: Tone, base?: NodeBase): Immutable<BadgeNode> {
	return freeze({ kind: 'badge' as const, label, tone, ...base });
}

/** Creates a semantic divider. */
function divider(label?: StringOrExpr, base?: NodeBase): Immutable<DividerNode> {
	return freeze({ kind: 'divider' as const, label, ...base });
}

/** Creates a media node that delegates rendering details to the platform renderer. */
function media(
	opts: {
		ref: MediaRef;
		mediaKind: MediaNode['mediaKind'];
		alt: string;
		aspectRatio?: AspectRatio;
		expandable?: boolean;
	} & NodeBase
): Immutable<MediaNode> {
	return freeze({ kind: 'media' as const, ...opts });
}

/** Makes a single child node behave as one semantic pressable region. */
function pressable(
	child: UiNode,
	actionRef: ActionRef,
	opts?: { label?: StringOrExpr } & NodeBase
): Immutable<PressableNode> {
	return freeze({ kind: 'pressable' as const, child, action: actionRef, ...opts });
}

/**
 * Creates a reusable item row suitable for lists and other collections.
 *
 * `id` is required — it must be the natural identifier of the underlying
 * data record (e.g. work.id, user.id). The renderer uses it to reconcile
 * items when the list changes without remounting stable rows.
 */
function item(
	id: string,
	primary: UiNode[] | string,
	opts?: Omit<ItemNode, 'kind' | 'id' | 'primary'>
): Immutable<ItemNode> {
	const primaryNodes =
		typeof primary === 'string'
			? [text(primary, { id: `${id}:primary`, role: 'body' })]
			: primary;
	return freeze({ kind: 'item' as const, id, primary: primaryNodes, ...opts });
}

/** Creates a list of semantic collection items. */
function list(
	items: ItemNode[],
	opts?: { density?: 'compact' | 'normal' | 'comfortable'; continuation?: CollectionContinuation } & NodeBase
): Immutable<ListNode> {
	return freeze({ kind: 'list' as const, items, ...opts });
}

/** Creates a user-invokable action node. */
function action(
	label: StringOrExpr,
	actionRef: ActionRef,
	opts?: {
		intent?: ActionNode['intent'];
		variant?: ActionVariant;
		leadingIcon?: string;
		disabled?: BoolOrExpr;
	} & NodeBase
): Immutable<ActionNode> {
	return freeze({ kind: 'action' as const, label, action: actionRef, ...opts });
}

/** Creates a container for grouped actions. */
function actions(children: ActionNode[], opts?: { align?: Align } & NodeBase): Immutable<ActionsNode> {
	return freeze({ kind: 'actions' as const, children, ...opts });
}

/** Creates an inline disclosure region controlled by local state. */
function disclosure(
	opts: {
		binding: string;
		label: StringOrExpr;
		labelExpanded?: StringOrExpr;
	} & NodeBase,
	children: UiNode[]
): Immutable<DisclosureNode> {
	return freeze({ kind: 'disclosure' as const, ...opts, children });
}

/** Creates a menu item for use inside `ui.menu(...)`. */
function menuItem(
	label: StringOrExpr,
	actionRef: ActionRef,
	opts?: Pick<MenuItem, 'selected' | 'disabled' | 'id'>
): Immutable<MenuItem> {
	return freeze({ label, action: actionRef, ...opts });
}

/** Creates a semantic popup menu trigger and option list. */
function menu(
	opts: {
		label: StringOrExpr;
		items: readonly MenuItem[];
		intent?: MenuNode['intent'];
		align?: MenuNode['align'];
	} & NodeBase
): Immutable<MenuNode> {
	return freeze({ kind: 'menu' as const, ...opts });
}

/** Creates an input field description. */
function input(
	opts: {
		name: string;
		inputKind: InputKind;
		label: StringOrExpr;
		value?: Primitive | UiExpr;
		placeholder?: StringOrExpr;
		helpText?: StringOrExpr;
		required?: boolean;
		disabled?: BoolOrExpr;
		options?: readonly SelectOption[];
		constraints?: InputNode['constraints'];
	} & NodeBase
): Immutable<InputNode> {
	return freeze({ kind: 'input' as const, ...opts });
}

/** Creates a semantic form container. */
function form(
	name: string,
	children: UiNode[],
	opts?: { submit?: ActionRef } & NodeBase
): Immutable<FormNode> {
	return freeze({ kind: 'form' as const, name, children, ...opts });
}

/** Creates a feedback/status block. */
function status(
	severity: StatusNode['severity'],
	message: StringOrExpr,
	opts?: { title?: StringOrExpr; actions?: readonly ActionNode[] } & NodeBase
): Immutable<StatusNode> {
	return freeze({ kind: 'status' as const, severity, message, ...opts });
}

/** Creates an empty-state block. */
function empty(
	title: StringOrExpr,
	opts?: { icon?: string; message?: StringOrExpr; actions?: readonly ActionNode[] } & NodeBase
): Immutable<EmptyStateNode> {
	return freeze({ kind: 'empty' as const, title, ...opts });
}

/** Creates a loading indicator block. */
function loading(opts?: { label?: StringOrExpr; progress?: NumberOrExpr } & NodeBase): Immutable<LoadingNode> {
	return freeze({ kind: 'loading' as const, ...opts });
}

/** Creates a conditional subtree. */
function when(
	condition: BoolOrExpr,
	thenNode: UiNode,
	elseNode?: UiNode,
	base?: NodeBase
): Immutable<ConditionalNode> {
	return freeze({ kind: 'conditional' as const, condition, then: thenNode, else: elseNode, ...base });
}

/** Creates an inline slot placeholder with optional fallback content. */
function slot(name: string, fallback?: UiNode, base?: NodeBase): Immutable<SlotNode> {
	return freeze({ kind: 'slot' as const, name, fallback, ...base });
}

/** Expression authoring helpers used inside semantic node fields. */
export const expr = {
	literal,
	binding,
	param
};

/** Main semantic UI DSL used by plugin `render()` functions. */
export const ui = {
	screen,
	section,
	stack,
	inline,
	grid,
	scroll,
	text,
	value,
	icon,
	badge,
	divider,
	media,
	pressable,
	item,
	list,
	action,
	actions,
	disclosure,
	menuItem,
	menu,
	input,
	form,
	status,
	empty,
	loading,
	when,
	slot
};