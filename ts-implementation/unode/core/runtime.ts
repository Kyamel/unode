import type { ActionRef, ImmutableNode, ImmutableScreen, JsonValue, Primitive, UiExpr } from './ast';
import type { CoreTranslator, MessageCatalogs, MessageValues } from './i18n';

export const UNODE_CORE_API_VERSION = '2.0.0-alpha.1' as const;

/** Function used to tear down a subscription or listener. */
export type Unsubscribe = () => void;
/** Listener notified when a single state path changes. */
export type StateListener = (value: unknown, path: string) => void;
/** Listener notified when the active route changes. */
export type RouteListener = (route: ResolvedRoute) => void;

/** Built-in permission identifiers owned by the `unode` core runtime. */
export type CoreBuiltinPermission =
	| 'http.fetch'
	| 'storage.session.read'
	| 'storage.session.write'
	| 'storage.persistent.read'
	| 'storage.persistent.write'
	| 'events.read'
	| 'events.write';

export interface PermissionRequest<TPermission extends string = string> {
	readonly permission: TPermission;
	readonly required?: boolean;
	readonly reason?: string;
	readonly origins?: readonly string[];
}

/**
 * Mutable per-screen state store used for local SPA-like reactivity.
 *
 * Renderers should observe this store instead of maintaining their own
 * competing source of truth for screen-local state.
 */
export interface StateStore {
	get(path: string): unknown;
	getPrimitive(path: string, fallback: Primitive): Primitive;
	set(path: string, value: JsonValue): void;
	mergeData(data: Record<string, unknown>): void;
	batch(fn: () => void): void;
	subscribe(path: string, listener: StateListener): Unsubscribe;
	subscribePrefix(prefix: string, listener: StateListener): Unsubscribe;
	snapshot(): Record<string, unknown>;
	reset(): void;
}

/** Addressable route state made available to plugin `load()` and `render()`. */
export interface ResolvedRoute {
	readonly pattern: string;
	readonly params: Readonly<Record<string, string>>;
	readonly query: Readonly<Record<string, string>>;
}

/**
 * Navigation API exposed by the host.
 *
 * The concept is route-driven, even when the concrete renderer is not a browser.
 */
export interface Navigator {
	navigate(
		to: string,
		options?: {
			params?: Record<string, string>;
			query?: Record<string, string>;
			mode?: 'push' | 'replace';
		}
	): void;

	back(): void;
	forward(): void;
	current(): ResolvedRoute;
	onNavigate(listener: RouteListener): Unsubscribe;
}

/** Context used when resolving expressions against current route/state. */
export interface ResolverContext {
	readonly state: StateStore;
	readonly route: ResolvedRoute;
	readonly locale: string;
}

/**
 * Optional expression resolver abstraction for runtimes that want to track
 * binding dependencies more explicitly.
 */
export interface ExprResolver {
	track(nodeKey: string, path: string): void;
	clearTracking(nodeKey: string): void;
	dependenciesOf(nodeKey: string): readonly string[];
	subscribersOf(path: string): readonly string[];
	resolvePrimitive(expr: Primitive | UiExpr, ctx: ResolverContext, nodeKey?: string): Primitive;
}

/** Host-backed HTTP helper surface exposed to plugin code. */
export interface UNodeHttpApi {
	fetch(url: string, init?: RequestInit): Promise<Response>;
	getJson<T = unknown>(url: string, headers?: Record<string, string>): Promise<T>;
	postJson<T = unknown>(
		url: string,
		body: unknown,
		headers?: Record<string, string>
	): Promise<T>;
}

/** Host-backed storage surface namespaced by plugin and scope. */
export interface UNodeStorageApi {
	get(scope: 'session' | 'persistent', key: string): Promise<JsonValue | undefined>;
	set(scope: 'session' | 'persistent', key: string, value: JsonValue): Promise<void>;
	delete(scope: 'session' | 'persistent', key: string): Promise<void>;
	keys(scope: 'session' | 'persistent'): Promise<string[]>;
}

/** Shared runtime event bus exposed to plugins. */
export interface UNodeEventsApi {
	emit(type: string, payload?: Record<string, JsonValue>): void;
	on(
		type: string,
		handler: (payload: Record<string, JsonValue>, from: string) => void | Promise<void>
	): Unsubscribe;
}

/**
 * Locale-aware translator owned by the core i18n registry.
 *
 * For plugin authoring, prefer aliasing `const t = ctx.i18n.t` inside `load()`
 * and `render()` so editor tooling can match plain `t('key')` calls.
 */
export interface PluginI18nApi extends CoreTranslator {
	register(catalogs: MessageCatalogs): void;
	translator(): CoreTranslator;
}

/**
 * Text that may either be eagerly provided as a string or lazily translated
 * later from a catalog entry.
 *
 * Use `msg(...)` for setup-time registries like navigation, commands, and actions.
 */
export type I18nText =
	| string
	| Readonly<{
			kind: 'i18n';
			key: string;
			values?: MessageValues;
			fallback?: string;
	  }>;

/** Creates a lazily translated text descriptor for setup-time registries. */
export function msg(key: string, values?: MessageValues, fallback?: string): I18nText {
	return {
		kind: 'i18n',
		key,
		values,
		fallback
	};
}

/** Minimal logger surface exposed to plugins. */
export interface Logger {
	info(message: string, meta?: Record<string, unknown>): void;
	warn(message: string, meta?: Record<string, unknown>): void;
	error(message: string, meta?: Record<string, unknown>): void;
}

/** Required plugin metadata understood by the runtime. */
export interface PluginManifest<TPermission extends string = CoreBuiltinPermission | string> {
	readonly id: string;
	readonly name: string;
	readonly version: string;
	readonly apiVersion: string;
	readonly description?: string;
	readonly author?: string;
	readonly permissions?: readonly PermissionRequest<TPermission>[];
	readonly requires?: readonly string[];
}

/**
 * Context available inside a route's `load()` and `render()` functions.
 *
 * This is the main authoring context for route-driven plugin UI.
 */
export interface PluginRenderContext<THostApi = unknown> {
	readonly pluginId: string;
	readonly route: ResolvedRoute;
	readonly state: StateStore;
	readonly navigate: Navigator['navigate'];
	readonly locale: string;
	readonly i18n: PluginI18nApi;
	readonly http: UNodeHttpApi;
	readonly storage: UNodeStorageApi;
	readonly events: UNodeEventsApi;
	readonly api: THostApi;
	dispatch(action: ActionRef): Promise<void>;
}

/**
 * Canonical route contract.
 *
 * `load()` fetches serializable data for the current route and `render()`
 * converts that data into an immutable semantic screen tree.
 */
export interface PluginRoute<TData = unknown, THostApi = unknown> {
	readonly pattern: string;
	load(ctx: PluginRenderContext<THostApi>): Promise<TData>;
	render(data: TData, ctx: PluginRenderContext<THostApi>): ImmutableScreen;
}

/** First step of the route builder API. */
export interface RouteBuilderNeedsLoad<THostApi = unknown> {
	load<TData>(
		load: (ctx: PluginRenderContext<THostApi>) => Promise<TData>
	): RouteBuilderNeedsRender<TData, THostApi>;
}

/** Second step of the route builder API. */
export interface RouteBuilderNeedsRender<TData, THostApi = unknown> {
	render(
		render: (data: TData, ctx: PluginRenderContext<THostApi>) => ImmutableScreen
	): PluginRoute<TData, THostApi>;
}

/** Contribution that can inject UI into a named screen slot. */
export interface SlotContribution<THostApi = unknown> {
	readonly id: string;
	readonly target: string;
	readonly priority?: number;
	when?(ctx: PluginRenderContext<THostApi>): boolean | Promise<boolean>;
	render(ctx: PluginRenderContext<THostApi>): Promise<ImmutableNode> | ImmutableNode;
}

/** Action execution context passed to custom action handlers. */
export interface ActionRunContext<THostApi = unknown> {
	readonly action: ActionRef;
	readonly pluginId: string;
	readonly route: ResolvedRoute;
	readonly host: THostApi;
	readonly i18n: PluginI18nApi;
}

/** Custom action handler registered by a plugin. */
export interface ActionDefinition<THostApi = unknown> {
	readonly id: string;
	readonly title: I18nText;
	run(ctx: ActionRunContext<THostApi>): void | Promise<void>;
}

/** Context passed to command availability checks and execution. */
export interface CommandContext<THostApi = unknown> {
	readonly pluginId: string;
	readonly route?: ResolvedRoute;
	readonly host: THostApi;
	readonly i18n: PluginI18nApi;
}

/** Command definition registered by a plugin. */
export interface CommandDefinition<THostApi = unknown> {
	readonly id: string;
	readonly title: I18nText;
	readonly category?: I18nText;
	readonly keywords?: readonly string[];
	when?(ctx: CommandContext<THostApi>): boolean | Promise<boolean>;
	run(ctx: CommandContext<THostApi>): void | Promise<void>;
}

/** Navigation item surfaced by the host shell. */
export interface NavigationItem<THostApi = unknown> {
	readonly id: string;
	readonly label: I18nText;
	readonly shortLabel?: I18nText;
	readonly to: string;
	readonly icon?: string;
	readonly section?: string;
	readonly priority?: number;
	when?(ctx: CommandContext<THostApi>): boolean | Promise<boolean>;
}

/** Provider execution context. */
export interface ProviderContext<THostApi = unknown> {
	readonly pluginId: string;
	readonly route?: ResolvedRoute;
	readonly host: THostApi;
}

/** Typed capability provider registered by a plugin. */
export interface ProviderDefinition<THostApi = unknown, TInput = unknown, TOutput = unknown> {
	readonly id: string;
	readonly capability: string;
	provide(input: TInput, ctx: ProviderContext<THostApi>): Promise<TOutput> | TOutput;
}

/** Route registration surface exposed during plugin setup. */
export interface RouteRegistryApi<THostApi = unknown> {
	register(def: PluginRoute<unknown, THostApi>): void;
}

/** Action registration surface exposed during plugin setup. */
export interface ActionRegistryApi<THostApi = unknown> {
	register(def: ActionDefinition<THostApi>): void;
}

/** Command registration surface exposed during plugin setup. */
export interface CommandRegistryApi<THostApi = unknown> {
	register(def: CommandDefinition<THostApi>): void;
}

/** Navigation registration surface exposed during plugin setup. */
export interface NavigationRegistryApi<THostApi = unknown> {
	register(item: NavigationItem<THostApi>): void;
}

/** Provider registration surface exposed during plugin setup. */
export interface ProviderRegistryApi<THostApi = unknown> {
	register(def: ProviderDefinition<THostApi>): void;
}

/** Slot registration surface exposed during plugin setup. */
export interface SlotRegistryApi<THostApi = unknown> {
	register(def: SlotContribution<THostApi>): void;
}

/**
 * Imperative plugin setup context.
 *
 * Prefer `definePlugin({...})` for the common case and reserve direct `setup(ctx)`
 * usage for advanced or dynamic registration logic.
 */
export interface PluginSetupContext<THostApi = unknown> {
	readonly plugin: PluginManifest;
	readonly routes: RouteRegistryApi<THostApi>;
	readonly actions: ActionRegistryApi<THostApi>;
	readonly commands: CommandRegistryApi<THostApi>;
	readonly navigation: NavigationRegistryApi<THostApi>;
	readonly providers: ProviderRegistryApi<THostApi>;
	readonly slots: SlotRegistryApi<THostApi>;
	readonly i18n: PluginI18nApi;
	readonly api: THostApi;
	readonly log: Logger;
}

/** Low-level plugin contract understood by the runtime. */
export interface PluginDefinition<THostApi = unknown> {
	readonly manifest: PluginManifest;
	activate(ctx: PluginSetupContext<THostApi>): void | Promise<void>;
	deactivate?(): void | Promise<void>;
}

/**
 * Declarative plugin definition used by the preferred authoring API.
 *
 * This shape makes the plugin contract visible in code:
 * manifest, i18n, navigation, commands, actions, routes, slots, and setup.
 */
export interface DeclarativePluginDefinition<THostApi = unknown> {
	readonly manifest: PluginManifest;
	readonly i18n?: MessageCatalogs;
	readonly routes?: readonly PluginRoute<any, THostApi>[];
	readonly actions?: readonly ActionDefinition<THostApi>[];
	readonly commands?: readonly CommandDefinition<THostApi>[];
	readonly navigation?: readonly NavigationItem<THostApi>[];
	readonly providers?: readonly ProviderDefinition<THostApi, unknown, unknown>[];
	readonly slots?: readonly SlotContribution<THostApi>[];
	setup?(ctx: PluginSetupContext<THostApi>): void | Promise<void>;
	deactivate?(): void | Promise<void>;
}

/**
 * Plain translation function used across view-model builders and screen helpers.
 *
 * This matches the preferred authoring style:
 * `const t = ctx.i18n.t; t('screen_title')`
 */
export type TranslateFn = (key: string, values?: MessageValues, fallback?: string) => string;

/** Typed identity helper for plain route object literals. */
export function defineRoute<TData = unknown, THostApi = unknown>(
	route: PluginRoute<TData, THostApi>
): PluginRoute<TData, THostApi> {
	return route;
}

/**
 * Starts a builder chain for route authoring.
 *
 * Example:
 * `route('/app/example').load(async (ctx) => data).render((data, ctx) => screen)`
 */
export function route<THostApi = unknown>(pattern: string): RouteBuilderNeedsLoad<THostApi> {
	return {
		load<TData>(
			load: (ctx: PluginRenderContext<THostApi>) => Promise<TData>
		): RouteBuilderNeedsRender<TData, THostApi> {
			return {
				render(
					render: (data: TData, ctx: PluginRenderContext<THostApi>) => ImmutableScreen
				): PluginRoute<TData, THostApi> {
					return {
						pattern,
						load,
						render
					};
				}
			};
		}
	};
}

/**
 * Preferred plugin authoring entrypoint.
 *
 * It registers declarative pieces like i18n catalogs, navigation items, commands,
 * actions, routes, slots, and providers in a predictable order, while still
 * allowing an optional `setup(ctx)` escape hatch.
 */
export function definePlugin<THostApi = unknown>(
	definition: DeclarativePluginDefinition<THostApi>
): PluginDefinition<THostApi> {
	return {
		manifest: definition.manifest,
		deactivate: definition.deactivate,
		async activate(ctx) {
			if (definition.i18n) {
				ctx.i18n.register(definition.i18n);
			}

			for (const item of definition.navigation ?? []) {
				ctx.navigation.register(item);
			}
			for (const command of definition.commands ?? []) {
				ctx.commands.register(command);
			}
			for (const action of definition.actions ?? []) {
				ctx.actions.register(action);
			}
			for (const provider of definition.providers ?? []) {
				ctx.providers.register(provider);
			}
			for (const slot of definition.slots ?? []) {
				ctx.slots.register(slot);
			}
			for (const routeDef of definition.routes ?? []) {
				ctx.routes.register(routeDef);
			}

			await definition.setup?.(ctx);
		}
	};
}
