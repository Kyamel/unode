import type { Immutable } from './immutable';

export const UNODE_AST_VERSION = '2.0.0-alpha.1' as const;

/** JSON-safe primitive value used throughout the canonical AST. */
export type Primitive = string | number | boolean | null;
export type JsonValue = Primitive | JsonObject | JsonArray;
export type JsonObject = { readonly [key: string]: JsonValue };
export type JsonArray = readonly JsonValue[];
export type StateValue = JsonValue;

/** Expression form used for route params, local bindings, and literals. */
export type UiExpr<T extends Primitive = Primitive> =
	| { readonly kind: 'literal'; readonly value: T }
	| { readonly kind: 'binding'; readonly path: string }
	| { readonly kind: 'param'; readonly name: string };

export type StringOrExpr = string | UiExpr<string>;
export type BoolOrExpr = boolean | UiExpr<boolean>;
export type NumberOrExpr = number | UiExpr<number>;
export type PrimitiveOrExpr = Primitive | UiExpr;

export type Tone = 'default' | 'muted' | 'info' | 'success' | 'warning' | 'danger';
export type Gap = 'none' | 'xs' | 'sm' | 'md' | 'lg';
export type TextRole =
	| 'heading'
	| 'title'
	| 'subtitle'
	| 'body'
	| 'label'
	| 'caption'
	| 'code'
	| 'hint';
export type ContainerRole =
	| 'page'
	| 'panel'
	| 'section'
	| 'group'
	| 'toolbar'
	| 'sidebar'
	| 'dialog';
export type ValueFormat =
	| 'number'
	| 'currency'
	| 'date'
	| 'datetime'
	| 'duration'
	| 'bytes'
	| 'percent'
	| 'raw';
export type ActionIntent = 'primary' | 'secondary' | 'ghost' | 'danger';
export type ActionVariant = 'button' | 'link' | 'icon-button' | 'menu-item';
export type InputKind =
	| 'text'
	| 'textarea'
	| 'number'
	| 'password'
	| 'email'
	| 'url'
	| 'boolean'
	| 'select'
	| 'multiselect'
	| 'date'
	| 'file';
export type AspectRatio = 'square' | 'poster' | 'video' | 'auto';
export type Align = 'start' | 'center' | 'end';
export type GridColumnCount = 1 | 2 | 3 | 4 | 5 | 6;
export type ResponsiveGridColumns = Readonly<{
	base?: GridColumnCount;
	sm?: GridColumnCount;
	md?: GridColumnCount;
	lg?: GridColumnCount;
	xl?: GridColumnCount;
}>;

/** Symbolic action reference interpreted by the runtime or renderer. */
export interface ActionRef {
	readonly type: string;
	readonly params?: Readonly<Record<string, JsonValue>>;
	readonly confirm?: Readonly<{
		title?: StringOrExpr;
		message: StringOrExpr;
	}>;
}

/** Reference to a media resource without committing to a renderer-specific implementation. */
export type MediaRef =
	| { readonly type: 'url'; readonly src: string }
	| { readonly type: 'at-blob'; readonly did: string; readonly cid: string }
	| { readonly type: 'asset'; readonly name: string }
	| {
			readonly type: 'placeholder';
			readonly kind?: 'cover' | 'image' | 'avatar' | 'thumbnail';
			readonly label?: string;
	  };

/**
 * Shared node metadata.
 *
 * `id` serves dual purpose:
 *   - Reconciliation identity: used by renderers to track nodes across
 *     reactive updates, equivalent to React's `key`. Must be unique among
 *     siblings in dynamic collections (lists, grids with continuation).
 *   - Semantic/accessibility identity: becomes the element's DOM id,
 *     aria-labelledby target, and focus restoration anchor.
 *
 * If absent, the normalizer derives a structural fallback from the node's
 * position in the tree (e.g. "screen.c0.c1"). This is stable within a load
 * cycle and safe for static structures. Dynamic collections should always
 * provide explicit ids.
 */
export interface NodeBase {
	readonly id?: string;
	readonly meta?: Readonly<Record<string, JsonValue>>;
}

/** Root screen node returned from plugin `render()`. */
export interface ScreenNode extends NodeBase {
	readonly kind: 'screen';
	readonly title?: StringOrExpr;
	readonly subtitle?: StringOrExpr;
	readonly initialFocus?: string;
	readonly initialState?: Readonly<Record<string, StateValue>>;
	readonly children: readonly UiNode[];
}

/** Semantic section container. */
export interface SectionNode extends NodeBase {
	readonly kind: 'section';
	readonly role?: ContainerRole;
	readonly title?: StringOrExpr;
	readonly description?: StringOrExpr;
	readonly children: readonly UiNode[];
}

/** Vertical layout container. */
export interface StackNode extends NodeBase {
	readonly kind: 'stack';
	readonly gap?: Gap;
	readonly children: readonly UiNode[];
}

/** Horizontal/wrapping layout container. */
export interface InlineNode extends NodeBase {
	readonly kind: 'inline';
	readonly gap?: Gap;
	readonly wrap?: boolean;
	readonly align?: Align;
	readonly children: readonly UiNode[];
}

/** Responsive grid container. */
export interface GridNode extends NodeBase {
	readonly kind: 'grid';
	readonly maxColumns?: GridColumnCount;
	readonly columns?: ResponsiveGridColumns;
	readonly gap?: Gap;
	readonly continuation?: CollectionContinuation;
	readonly children: readonly UiNode[];
}

/** Scrollable container. */
export interface ScrollNode extends NodeBase {
	readonly kind: 'scroll';
	readonly children: readonly UiNode[];
}

/** Semantic text leaf node. */
export interface TextNode extends NodeBase {
	readonly kind: 'text';
	readonly content: StringOrExpr;
	readonly role?: TextRole;
	readonly tone?: Tone;
	readonly emphasis?: 'normal' | 'strong';
	readonly truncate?: boolean;
}

/** Formatted scalar value leaf node. */
export interface ValueNode extends NodeBase {
	readonly kind: 'value';
	readonly value: PrimitiveOrExpr;
	readonly format: ValueFormat;
	readonly currencyCode?: string;
	readonly role?: TextRole;
	readonly tone?: Tone;
}

/** Semantic icon leaf node. */
export interface IconNode extends NodeBase {
	readonly kind: 'icon';
	readonly name: string;
	readonly label: string;
	readonly tone?: Tone;
}

/** Compact badge/chip leaf node. */
export interface BadgeNode extends NodeBase {
	readonly kind: 'badge';
	readonly label: StringOrExpr;
	readonly tone?: Tone;
}

/** Visual separator with optional label. */
export interface DividerNode extends NodeBase {
	readonly kind: 'divider';
	readonly label?: StringOrExpr;
}

/** Media leaf node rendered according to platform capabilities. */
export interface MediaNode extends NodeBase {
	readonly kind: 'media';
	readonly ref: MediaRef;
	readonly mediaKind: 'cover' | 'image' | 'avatar' | 'thumbnail';
	readonly alt: string;
	readonly aspectRatio?: AspectRatio;
	readonly expandable?: boolean;
}

/** Single clickable/focusable region wrapping another node. */
export interface PressableNode extends NodeBase {
	readonly kind: 'pressable';
	readonly child: UiNode;
	readonly action: ActionRef;
	readonly label?: StringOrExpr;
}

/**
 * Reusable collection item row structure.
 *
 * `id` is required on ItemNode because items appear in dynamic collections
 * (ListNode, GridNode) where order may change or items may be inserted/removed.
 * A stable id is the only way the renderer can reconcile items correctly
 * without remounting unchanged rows. Use the underlying data record's natural
 * identifier: work.id, chapter.id, etc.
 */
export interface ItemNode {
	readonly kind: 'item';
	readonly id: string;
	readonly meta?: Readonly<Record<string, JsonValue>>;
	readonly leading?: readonly UiNode[];
	readonly primary: readonly UiNode[];
	readonly secondary?: readonly UiNode[];
	readonly trailing?: readonly UiNode[];
	readonly action?: ActionRef;
}

/** Local continuation strategy that reveals more already-loaded items. */
export interface IncrementalContinuation {
	readonly kind: 'incremental';
	readonly binding: string;
	readonly initial: number;
	readonly step: number;
	readonly label?: StringOrExpr;
}

/** Remote continuation strategy that asks the host/plugin for more items. */
export interface RemoteContinuation {
	readonly kind: 'remote';
	readonly hasMore: boolean;
	readonly loadMore: ActionRef;
	readonly label?: StringOrExpr;
	readonly loadingLabel?: StringOrExpr;
}

/** Supported continuation strategies for large collections. */
export type CollectionContinuation = IncrementalContinuation | RemoteContinuation;

/** Semantic list of collection items. */
export interface ListNode extends NodeBase {
	readonly kind: 'list';
	readonly items: readonly ItemNode[];
	readonly density?: 'compact' | 'normal' | 'comfortable';
	readonly continuation?: CollectionContinuation;
}

/** User-invokable action node. */
export interface ActionNode extends NodeBase {
	readonly kind: 'action';
	readonly label: StringOrExpr;
	readonly action: ActionRef;
	readonly intent?: ActionIntent;
	readonly variant?: ActionVariant;
	readonly leadingIcon?: string;
	readonly disabled?: BoolOrExpr;
}

/** Group of related actions. */
export interface ActionsNode extends NodeBase {
	readonly kind: 'actions';
	readonly align?: Align;
	readonly children: readonly ActionNode[];
}

/** Inline collapsible region controlled by a binding path. */
export interface DisclosureNode extends NodeBase {
	readonly kind: 'disclosure';
	readonly binding: string;
	readonly label: StringOrExpr;
	readonly labelExpanded?: StringOrExpr;
	readonly children: readonly UiNode[];
}

/** Option rendered inside a menu. */
export interface MenuItem {
	readonly id?: string;
	readonly label: StringOrExpr;
	readonly action: ActionRef;
	readonly selected?: boolean;
	readonly disabled?: BoolOrExpr;
}

/** Popup or contextual menu trigger and item list. */
export interface MenuNode extends NodeBase {
	readonly kind: 'menu';
	readonly label: StringOrExpr;
	readonly items: readonly MenuItem[];
	readonly intent?: ActionIntent;
	readonly align?: Exclude<Align, 'center'>;
}

/** Select-like option shape used by certain input kinds. */
export interface SelectOption {
	readonly label: string;
	readonly value: Primitive;
	readonly disabled?: boolean;
}

/** Semantic input field description. */
export interface InputNode extends NodeBase {
	readonly kind: 'input';
	readonly name: string;
	readonly inputKind: InputKind;
	readonly label: StringOrExpr;
	readonly value?: PrimitiveOrExpr;
	readonly placeholder?: StringOrExpr;
	readonly helpText?: StringOrExpr;
	readonly required?: boolean;
	readonly disabled?: BoolOrExpr;
	readonly options?: readonly SelectOption[];
	readonly constraints?: Readonly<{
		min?: number;
		max?: number;
		minLength?: number;
		maxLength?: number;
		pattern?: string;
	}>;
}

/** Semantic form container. */
export interface FormNode extends NodeBase {
	readonly kind: 'form';
	readonly name: string;
	readonly children: readonly UiNode[];
	readonly submit?: ActionRef;
}

/** Status or feedback block. */
export interface StatusNode extends NodeBase {
	readonly kind: 'status';
	readonly severity: Exclude<Tone, 'default' | 'muted'>;
	readonly title?: StringOrExpr;
	readonly message: StringOrExpr;
	readonly actions?: readonly ActionNode[];
}

/** Empty state block for absent content. */
export interface EmptyStateNode extends NodeBase {
	readonly kind: 'empty';
	readonly icon?: string;
	readonly title: StringOrExpr;
	readonly message?: StringOrExpr;
	readonly actions?: readonly ActionNode[];
}

/** Loading indicator block. */
export interface LoadingNode extends NodeBase {
	readonly kind: 'loading';
	readonly label?: StringOrExpr;
	readonly progress?: NumberOrExpr;
}

/** Conditional subtree. */
export interface ConditionalNode extends NodeBase {
	readonly kind: 'conditional';
	readonly condition: BoolOrExpr;
	readonly then: UiNode;
	readonly else?: UiNode;
}

/** Inline slot placeholder with optional fallback UI. */
export interface SlotNode extends NodeBase {
	readonly kind: 'slot';
	readonly name: string;
	readonly fallback?: UiNode;
}

/** All canonical child-capable UI nodes except the root `screen`. */
export type UiNode =
	| SectionNode
	| StackNode
	| InlineNode
	| GridNode
	| ScrollNode
	| TextNode
	| ValueNode
	| IconNode
	| BadgeNode
	| DividerNode
	| MediaNode
	| PressableNode
	| ItemNode
	| ListNode
	| ActionNode
	| ActionsNode
	| DisclosureNode
	| MenuNode
	| InputNode
	| FormNode
	| StatusNode
	| EmptyStateNode
	| LoadingNode
	| ConditionalNode
	| SlotNode;

export type RootNode = ScreenNode | UiNode;

/** String union of all non-root node kinds. */
export type UiNodeKind = UiNode['kind'];
export type ImmutableNode = Immutable<UiNode>;
export type ImmutableScreen = Immutable<ScreenNode>;