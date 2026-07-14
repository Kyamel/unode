import type { HostApi, HostEvent } from '../api/host';
import type { RuntimeRegistries } from './runtime';
import { createPluginStorage } from '../storage/pluginStorage';
import { CoreI18nRegistry } from '../core/i18n';
import type { CoreTranslator, MessageCatalogs } from '../core/i18n';
import type { JsonValue } from '../core/ast';
import { hasPermission } from '../utils/permissions';
import type {
	I18nText,
	PluginDefinition,
	PluginI18nApi,
	PluginManifest,
	PluginRenderContext,
	PluginSetupContext,
	ResolvedRoute,
	StateStore
} from '../core/runtime';
import { assertCoreStoragePermission, guardCoreHostApi } from './guard';

function createLogger(pluginId: string) {
	return {
		info: (message: string, meta?: Record<string, unknown>) =>
			console.info(`[plugin:${pluginId}] ${message}`, meta ?? ''),
		warn: (message: string, meta?: Record<string, unknown>) =>
			console.warn(`[plugin:${pluginId}] ${message}`, meta ?? ''),
		error: (message: string, meta?: Record<string, unknown>) =>
			console.error(`[plugin:${pluginId}] ${message}`, meta ?? '')
	};
}

function toLazyText(text: I18nText, i18n: PluginI18nApi): string | (() => string) {
	if (typeof text === 'string') return text;
	return () => i18n.t(text.key, text.values, text.fallback);
}

function createEventBus(
	plugin: PluginManifest,
	hostEvents: Pick<HostApi['events'], 'emit' | 'on'>
) {
	return {
		emit(type: string, payload?: Record<string, JsonValue>) {
			if (!hasPermission(plugin, 'events.write')) {
				throw new Error('Missing permission: events.write');
			}
			hostEvents.emit({ type, payload, from: plugin.id });
		},
		on(
			type: string,
			handler: (payload: Record<string, JsonValue>, from: string) => void | Promise<void>
		) {
			if (!hasPermission(plugin, 'events.read')) {
				throw new Error('Missing permission: events.read');
			}
			return hostEvents.on(type, async (event) => {
				const source =
					event && typeof event === 'object' && 'from' in event && typeof event.from === 'string'
						? event.from
						: 'host';
				const payloadValue =
					event &&
					typeof event === 'object' &&
					'payload' in event &&
					event.payload &&
					typeof event.payload === 'object'
						? (event.payload as Record<string, JsonValue>)
						: {};
				await handler(payloadValue, source);
			});
		}
	};
}

function createPluginI18n(
	pluginId: string,
	registry: CoreI18nRegistry,
	getLocale: () => string
): PluginI18nApi {
	const translator = (): CoreTranslator => registry.createTranslator(pluginId, getLocale());
	return {
		register(catalogs: MessageCatalogs) {
			registry.register(pluginId, catalogs);
		},
		translator,
		locale: () => getLocale(),
		t: (key, values, fallback) => translator().t(key, values, fallback)
	};
}

function createHttpAdapter(plugin: PluginManifest, host: HostApi) {
	return {
		async fetch(url: string, init?: RequestInit) {
			if (!hasPermission(plugin, 'http.fetch')) {
				throw new Error('Missing permission: http.fetch');
			}
			if (typeof fetch !== 'function') {
				throw new Error('Global fetch is not available in this runtime.');
			}
			return await fetch(url, init);
		},
		async getJson<T = unknown>(url: string, headers?: Record<string, string>) {
			if (!hasPermission(plugin, 'http.fetch')) {
				throw new Error('Missing permission: http.fetch');
			}
			const response = await host.http.request<T>({ method: 'GET', url, headers });
			return response.data;
		},
		async postJson<T = unknown>(url: string, body: unknown, headers?: Record<string, string>) {
			if (!hasPermission(plugin, 'http.fetch')) {
				throw new Error('Missing permission: http.fetch');
			}
			const response = await host.http.request<T>({ method: 'POST', url, headers, body });
			return response.data;
		}
	};
}

function createStorageAdapter(
	plugin: PluginManifest,
	storage: ReturnType<typeof createPluginStorage>
) {
	return {
		async get(scope: 'session' | 'persistent', key: string) {
			assertCoreStoragePermission(plugin, scope, 'read');
			return (await storage.get(`${scope}:${plugin.id}:${key}`)) as JsonValue | undefined;
		},
		async set(scope: 'session' | 'persistent', key: string, value: JsonValue) {
			assertCoreStoragePermission(plugin, scope, 'write');
			await storage.set(`${scope}:${plugin.id}:${key}`, value);
		},
		async delete(scope: 'session' | 'persistent', key: string) {
			assertCoreStoragePermission(plugin, scope, 'write');
			await storage.delete(`${scope}:${plugin.id}:${key}`);
		},
		async keys() {
			return [];
		}
	};
}

export function createPluginSetupContext<THostApi extends HostApi>(
	plugin: PluginDefinition<THostApi>,
	registries: RuntimeRegistries,
	hostApi: THostApi,
	guardHostApi?: (plugin: PluginManifest, api: THostApi) => THostApi
): PluginSetupContext<THostApi> {
	const manifest = plugin.manifest;
	const coreGuardedApi = guardCoreHostApi(manifest, hostApi);
	const guardedApi = guardHostApi ? guardHostApi(manifest, coreGuardedApi) : coreGuardedApi;
	const i18nRegistry = new CoreI18nRegistry();
	const i18n = createPluginI18n(manifest.id, i18nRegistry, () => guardedApi.i18n.getLocale());

	return {
		plugin: manifest,
		routes: {
			register: (def) => registries.routes.registerCore(def, manifest, guardedApi, i18n)
		},
		actions: {
			register: (def) =>
				registries.actions.register(
					{
						...def,
						title: toLazyText(def.title, i18n)
					} as never,
					manifest.id
				)
		},
		commands: {
			register: (def) =>
				registries.commands.register(
					{
						...def,
						title: toLazyText(def.title, i18n),
						category: def.category ? toLazyText(def.category, i18n) : undefined
					} as never,
					manifest.id
				)
		},
		navigation: {
			register: (item) =>
				registries.navigation.register(
					{
						...item,
						label: toLazyText(item.label, i18n),
						shortLabel: item.shortLabel ? toLazyText(item.shortLabel, i18n) : undefined
					} as never,
					manifest.id
				)
		},
		providers: {
			register: (def) => registries.providers.register(def as never, manifest.id)
		},
		slots: {
			register: (def) => registries.screens.registerCoreSection(def, manifest, guardedApi, i18n)
		},
		i18n,
		api: guardedApi,
		log: createLogger(manifest.id)
	};
}

export function createRenderContext<THostApi extends HostApi>(
	plugin: PluginManifest,
	host: THostApi,
	route: ResolvedRoute,
	state: StateStore,
	i18n: PluginI18nApi
): PluginRenderContext<THostApi> {
	return {
		pluginId: plugin.id,
		route,
		state,
		navigate: (to, options) => {
			void host.navigation.navigate(to, {
				replace: options?.mode === 'replace',
				state: {
					params: options?.params,
					query: options?.query
				}
			});
		},
		locale: host.i18n.getLocale(),
		i18n,
		http: createHttpAdapter(plugin, host),
		storage: createStorageAdapter(plugin, createPluginStorage(plugin.id)),
		events: createEventBus(plugin, host.events),
		api: host,
		dispatch: async (action) => {
			if (action.type === 'unode.navigate') {
				const target = typeof action.params?.to === 'string' ? action.params.to : '';
				if (!target) return;
				await host.navigation.navigate(target, {
					replace: action.params?.mode === 'replace',
					state: {
						query:
							action.params?.query &&
							typeof action.params.query === 'object' &&
							!Array.isArray(action.params.query)
								? action.params.query
								: undefined
					}
				});
				return;
			}

			if (action.type === 'unode.setState') {
				const path = typeof action.params?.path === 'string' ? action.params.path : '';
				if (!path) return;
				state.set(path, (action.params?.value ?? null) as JsonValue);
				return;
			}

			throw new Error(
				`Runtime dispatch cannot invoke custom core action "${action.type}" during render.`
			);
		}
	};
}

export function toRouteInfo(pathname: string, pattern: string, params: Record<string, string>) {
	return {
		pathname,
		params,
		screenKind: pattern,
		pluginId: ''
	};
}

export function createHostEventBus(hostEvents: Pick<HostApi['events'], 'emit' | 'on'>) {
	return {
		emit(event: HostEvent) {
			hostEvents.emit(event);
		},
		on<T extends HostEvent['type']>(
			type: T,
			handler: (event: Extract<HostEvent, { type: T }>) => void | Promise<void>
		) {
			return hostEvents.on(type, handler);
		}
	};
}
