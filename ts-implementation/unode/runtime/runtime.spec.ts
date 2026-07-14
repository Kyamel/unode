import { describe, expect, it } from 'vitest';
import { PluginRuntime } from './runtime';
import type { HostApi, HostEvent } from '../api/host';
import { defineRoute, msg, UNODE_CORE_API_VERSION, type PluginDefinition } from '../core/runtime';
import { ui as coreUi } from '../core/dsl';
import { normalizeScreen } from '../core/normalize';

function createHostApiStub(): HostApi {
	const listeners = new Map<string, Set<(event: HostEvent) => void | Promise<void>>>();

	return {
		navigation: {
			async navigate() {},
			async openExternal() {},
			async openScreen() {},
			async getCurrentRoute() {
				return {
					pathname: '/',
					params: {},
					screenKind: 'host.home',
					pluginId: 'host'
				};
			}
		},
		feedback: {
			toast() {},
			async confirm() {
				return true;
			}
		},
		storage: {
			async getScoped() {
				return null;
			},
			async setScoped() {}
		},
		http: {
			async request<T>() {
				return { status: 200, data: {} as T };
			}
		},
		i18n: {
			getLocale() {
				return 'en';
			},
			translate({ key, fallback }) {
				return fallback ?? key;
			},
			getTranslator() {
				return {
					t(key, _values, fallback) {
						return fallback ?? key;
					},
					locale() {
						return 'en';
					}
				};
			}
		},
		events: {
			emit(event) {
				const handlers = listeners.get(event.type);
				if (!handlers) return;
				for (const handler of handlers) {
					void handler(event);
				}
			},
			on(type, handler) {
				const handlers = listeners.get(type) ?? new Set();
				handlers.add(handler as (event: HostEvent) => void | Promise<void>);
				listeners.set(type, handlers);
				return () => {
					const current = listeners.get(type);
					if (!current) return;
					current.delete(handler as (event: HostEvent) => void | Promise<void>);
				};
			}
		},
		system: {
			async getRuntimeInfo() {
				return {
					platform: 'web',
					appVersion: 'test',
					pluginApiVersion: 'test'
				} as const;
			}
		}
	};
}

describe('PluginRuntime', () => {
	it('activates a core plugin directly and resolves its canonical screen', async () => {
		const runtime = new PluginRuntime();
		const hostApi = createHostApiStub();

		const plugin: PluginDefinition<HostApi> = {
			manifest: {
				id: 'core.demo',
				name: 'Core Demo',
				version: '0.0.1',
				apiVersion: UNODE_CORE_API_VERSION
			},
			async activate(ctx) {
				const route = defineRoute<{ greeting: string }, HostApi>({
					pattern: '/core-demo',
					async load() {
						return { greeting: 'Hello from core' };
					},
					render(data) {
						return coreUi.screen({ id: 'runtime-spec:core-demo-screen', title: 'Core Demo' }, [
							coreUi.section({ key: 'runtime-spec:core-demo:section', title: 'Body' }, [
								coreUi.text(data.greeting, {
									key: 'runtime-spec:core-demo:greeting',
									role: 'body'
								})
							])
						]);
					}
				});

				ctx.routes.register(route);
			}
		};

		await runtime.activateModules([plugin], hostApi);
		const resolved = await runtime.resolveScreen('/core-demo', new URLSearchParams(), hostApi);

		expect(resolved?.title).toBe('Core Demo');
		expect(resolved?.screenKind).toBe('core.demo.core-demo');
		expect(resolved?.body.kind).toBe('screen');
		if (!resolved || resolved.body.kind !== 'screen') {
			throw new Error('Expected resolved screen body to be a canonical screen node.');
		}
		expect(resolved.body.children[0]).toMatchObject({
			kind: 'section',
			title: 'Body'
		});
	});

	it('preserves screen metadata when composing the resolved screen', async () => {
		const runtime = new PluginRuntime();
		const hostApi = createHostApiStub();

		const resolved = await runtime.composeScreen(
			{
				screenKind: 'test.meta',
				title: 'Meta screen',
				meta: {
					routeTabs: {
						kind: 'route-tabs',
						active: 'meta',
						tabs: [{ id: 'meta', label: 'Meta', to: '/meta' }]
					}
				},
				body: normalizeScreen(coreUi.screen({ id: 'runtime-spec:meta-screen' }, []))
			},
			{
				pathname: '/meta',
				params: {},
				screenKind: 'test.meta',
				pluginId: 'test.plugin'
			},
			new URLSearchParams(),
			hostApi,
			'test.plugin'
		);

		expect(resolved.meta).toMatchObject({
			routeTabs: {
				kind: 'route-tabs',
				active: 'meta'
			}
		});
	});

	it('uses the host event bus so plugins can exchange events', async () => {
		const runtime = new PluginRuntime();
		const hostApi = createHostApiStub();
		const received: string[] = [];

		const listenerPlugin: PluginDefinition<HostApi> = {
			manifest: {
				id: 'listener.demo',
				name: 'Listener Demo',
				version: '0.0.1',
				apiVersion: UNODE_CORE_API_VERSION,
				permissions: [{ permission: 'events.read' }]
			},
			activate(ctx) {
				ctx.routes.register({
					pattern: '/listener',
					async load(renderCtx) {
						renderCtx.events.on('screen.refresh', async (payload) => {
							received.push(String(payload.pathname ?? ''));
						});
						return null;
					},
					render() {
						return coreUi.screen({ id: 'runtime-spec:listener-screen' }, []);
					}
				});
			}
		};

		const emitterPlugin: PluginDefinition<HostApi> = {
			manifest: {
				id: 'emitter.demo',
				name: 'Emitter Demo',
				version: '0.0.1',
				apiVersion: UNODE_CORE_API_VERSION,
				permissions: [{ permission: 'events.write' }]
			},
			activate(ctx) {
				ctx.routes.register({
					pattern: '/emitter',
					async load(renderCtx) {
						renderCtx.events.emit('screen.refresh', { pathname: '/from-emitter' });
						return null;
					},
					render() {
						return coreUi.screen({ id: 'runtime-spec:emitter-screen' }, []);
					}
				});
			}
		};

		await runtime.activateModules([listenerPlugin, emitterPlugin], hostApi);
		await runtime.resolveScreen('/listener', new URLSearchParams(), hostApi);
		await runtime.resolveScreen('/emitter', new URLSearchParams(), hostApi);

		expect(received).toEqual(['/from-emitter']);
	});

	it('enforces core storage permissions in render contexts', async () => {
		const runtime = new PluginRuntime();
		const hostApi = createHostApiStub();

		const plugin: PluginDefinition<HostApi> = {
			manifest: {
				id: 'storage.guard.demo',
				name: 'Storage Guard Demo',
				version: '0.0.1',
				apiVersion: UNODE_CORE_API_VERSION
			},
			activate(ctx) {
				ctx.routes.register({
					pattern: '/storage-guard',
					async load(renderCtx) {
						await renderCtx.storage.get('persistent', 'demo');
						return null;
					},
					render() {
						return coreUi.screen({ id: 'runtime-spec:storage-guard-screen' }, []);
					}
				});
			}
		};

		await runtime.activateModules([plugin], hostApi);

		await expect(
			runtime.resolveScreen('/storage-guard', new URLSearchParams(), hostApi)
		).rejects.toThrow('Missing permission: storage.persistent.read');
	});

	it('enforces core event permissions on the host api before bridge guards', async () => {
		const runtime = new PluginRuntime();
		const hostApi = createHostApiStub();

		const plugin: PluginDefinition<HostApi> = {
			manifest: {
				id: 'events.guard.demo',
				name: 'Events Guard Demo',
				version: '0.0.1',
				apiVersion: UNODE_CORE_API_VERSION
			},
			activate(ctx) {
				ctx.routes.register({
					pattern: '/events-guard',
					async load(renderCtx) {
						renderCtx.api.events.emit({ type: 'screen.refresh' });
						return null;
					},
					render() {
						return coreUi.screen({ id: 'runtime-spec:events-guard-screen' }, []);
					}
				});
			}
		};

		await runtime.activateModules([plugin], hostApi);

		await expect(
			runtime.resolveScreen('/events-guard', new URLSearchParams(), hostApi)
		).rejects.toThrow('Missing permission: events.write');
	});

	it('does not accept legacy core permission aliases', async () => {
		const runtime = new PluginRuntime();
		const hostApi = createHostApiStub();

		const legacyStoragePlugin: PluginDefinition<HostApi> = {
			manifest: {
				id: 'legacy.storage.demo',
				name: 'Legacy Storage Demo',
				version: '0.0.1',
				apiVersion: UNODE_CORE_API_VERSION,
				permissions: [{ permission: 'storage.local' }]
			},
			activate(ctx) {
				ctx.routes.register({
					pattern: '/legacy-storage',
					async load(renderCtx) {
						await renderCtx.storage.get('persistent', 'demo');
						return null;
					},
					render() {
						return coreUi.screen({ id: 'runtime-spec:legacy-storage-screen' }, []);
					}
				});
			}
		};

		const legacyHttpPlugin: PluginDefinition<HostApi> = {
			manifest: {
				id: 'legacy.http.demo',
				name: 'Legacy Http Demo',
				version: '0.0.1',
				apiVersion: UNODE_CORE_API_VERSION,
				permissions: [{ permission: 'http.external' }]
			},
			activate(ctx) {
				ctx.routes.register({
					pattern: '/legacy-http',
					async load(renderCtx) {
						await renderCtx.http.getJson('https://example.test/api');
						return null;
					},
					render() {
						return coreUi.screen({ id: 'runtime-spec:legacy-http-screen' }, []);
					}
				});
			}
		};

		await runtime.activateModules([legacyStoragePlugin, legacyHttpPlugin], hostApi);

		await expect(
			runtime.resolveScreen('/legacy-storage', new URLSearchParams(), hostApi)
		).rejects.toThrow('Missing permission: storage.persistent.read');
		await expect(
			runtime.resolveScreen('/legacy-http', new URLSearchParams(), hostApi)
		).rejects.toThrow('Missing permission: http.fetch');
	});

	it('resolves lazy i18n labels for navigation and commands even if catalogs are registered later in activate', async () => {
		const runtime = new PluginRuntime();
		const hostApi = createHostApiStub();

		const plugin: PluginDefinition<HostApi> = {
			manifest: {
				id: 'lazy.i18n.demo',
				name: 'Lazy I18n Demo',
				version: '0.0.1',
				apiVersion: UNODE_CORE_API_VERSION
			},
			activate(ctx) {
				ctx.navigation.register({
					id: 'lazy.i18n.nav',
					label: msg('nav_label'),
					shortLabel: msg('nav_short'),
					to: '/lazy'
				});

				ctx.commands.register({
					id: 'lazy.i18n.command',
					title: msg('command_title'),
					category: msg('command_category'),
					run() {}
				});

				ctx.i18n.register({
					en: {
						nav_label: 'Lazy nav',
						nav_short: 'LN',
						command_title: 'Lazy command',
						command_category: 'Lazy category'
					}
				});
			}
		};

		await runtime.activateModules([plugin], hostApi);

		const navItems = await runtime.registries.navigation.getAvailable({
			host: hostApi
		});
		const commands = await runtime.registries.commands.getAvailable({
			host: hostApi
		});

		expect(navItems).toEqual([
			expect.objectContaining({
				id: 'lazy.i18n.nav',
				label: 'Lazy nav',
				shortLabel: 'LN'
			})
		]);
		expect(commands).toEqual([
			expect.objectContaining({
				id: 'lazy.i18n.command',
				title: 'Lazy command',
				category: 'Lazy category'
			})
		]);
	});
});
