import { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import {
	defineRenderer,
	h,
	hostSlot,
	ScreenStore,
	UnodeScreen,
	type ActionRef,
	type HostComponentProps,
} from 'unode-react';
import {
	HostSession,
	PluginInstance,
	StateWriteSink,
	resolveRoutePattern,
	routeTabsView,
	type ResolvedRoute,
	type RouteTabsView,
} from 'unode-web-core';
import { playgroundPluginAssets, type PluginManifest, type PluginManifestEnvelope, type PlaygroundPluginAsset } from '../../playground/registry';
import * as webHostModule from '../../playground/pkg/unode_web_host.js';
import webHostWasmUrl from '../../playground/pkg/unode_web_host_bg.wasm?url';
import './playground.css';

type LoadedPlugin = {
	asset: PlaygroundPluginAsset;
	plugin: PluginInstance;
	sink: StateWriteSink;
	envelope: PluginManifestEnvelope;
};

type EventLogEntry = {
	id: number;
	action: string;
	targetPluginId: string;
	message: string;
	originContributionId?: string;
};

type DispatchResponse = {
	handled?: boolean;
	message?: string;
	outcome?: { kind?: string; to?: string };
};

type SlotResponseEnvelope = {
	pluginId: string;
	contributionId: string;
	response: unknown;
};

const locale = 'en';

function literal(value: unknown): unknown {
	if (value && typeof value === 'object' && 'v' in value) {
		return (value as { v: unknown }).v;
	}
	return value;
}

function text(value: unknown): string {
	return String(literal(value) ?? '');
}

const ROUTE_BASE = 'https://unode.dev';

/**
 * Route patterns the plugin answers for: the ones declared in its manifest,
 * plus the registry's legacy pattern as a fallback for plugins that declare
 * none.
 */
function routePatternsFor(entry: LoadedPlugin): string[] {
	const declared = (entry.envelope.manifest.routes ?? []).map((route) => route.pattern);
	if (declared.includes(entry.asset.routePattern)) return declared;
	return [entry.asset.routePattern, ...declared];
}

/** The first static pattern is the plugin's landing screen. */
function defaultRouteFor(entry: LoadedPlugin): string {
	return (
		routePatternsFor(entry).find((pattern) => !pattern.includes(':')) ?? entry.asset.routePattern
	);
}

function resolvePluginRoute(entry: LoadedPlugin, to: string): ResolvedRoute | undefined {
	const url = new URL(to, ROUTE_BASE);
	const match = resolveRoutePattern(routePatternsFor(entry), url.pathname);
	if (!match) return undefined;
	return {
		pattern: match.pattern,
		params: match.params,
		query: Object.fromEntries(url.searchParams.entries()),
	};
}

type ResolvedPlaygroundRoute = { entry: LoadedPlugin; route: ResolvedRoute };

/**
 * Resolves a destination against every loaded plugin, like a host shell's
 * route registry: exact matches win over `:param` matches.
 */
function resolveAcrossPlugins(
	loaded: LoadedPlugin[],
	to: string,
): ResolvedPlaygroundRoute | undefined {
	let paramMatch: ResolvedPlaygroundRoute | undefined;
	for (const entry of loaded) {
		const route = resolvePluginRoute(entry, to);
		if (!route) continue;
		if (Object.keys(route.params ?? {}).length === 0) return { entry, route };
		paramMatch ??= { entry, route };
	}
	return paramMatch;
}

/** The plugin route mirrored into the browser URL: `/playground#/plugins/...`. */
function parseHashRoute(): string | null {
	const hash = window.location.hash.replace(/^#/, '');
	return hash.startsWith('/') ? hash : null;
}

function PluginButton({ children, intent = 'secondary', action, dispatch }: HostComponentProps) {
	const label = text(children);
	const actionRef = action as ActionRef | undefined;
	return (
		<button
			className={`pg-button intent-${text(intent) || 'secondary'}`}
			type="button"
			title={actionRef?.originPluginId ? `Dispatches to ${actionRef.originPluginId}` : undefined}
			onClick={() => actionRef && dispatch(actionRef)}
		>
			<span>{label}</span>
			{actionRef?.originPluginId && <small>{actionRef.originPluginId.split('.').pop()}</small>}
		</button>
	);
}

const renderer = defineRenderer()
	.screen(({ props, children }) =>
		h(
			'section',
			{ class: 'pg-screen' },
			h(
				'div',
				{ class: 'pg-screen-heading' },
				h('div', {}, h('h1', {}, text(props.title)), props.subtitle ? h('p', {}, text(props.subtitle)) : null),
			),
			h('div', { class: 'pg-node-stack' }, children),
		),
	)
	.recipe('action', ({ label, intent, action }) =>
		hostSlot('Button', { children: label, intent, action }),
	)
	.recipe('actions', ({ children }) => h('div', { class: 'pg-inline' }, children))
	.recipe('section', ({ title, prop, children }) =>
		h(
			'section',
			{ class: 'pg-section' },
			title || prop('description')
				? h(
						'div',
						{ class: 'pg-section-title' },
						title ? h('h2', {}, title) : null,
						prop('description') ? h('p', {}, text(prop('description'))) : null,
					)
				: null,
			h('div', { class: 'pg-node-stack' }, children),
		),
	)
	.recipe('stack', ({ children }) => h('div', { class: 'pg-node-stack' }, children))
	.recipe('inline', ({ children }) => h('div', { class: 'pg-inline' }, children))
	.recipe('text', ({ content, role, prop }) =>
		h('p', { class: `pg-text role-${role} tone-${text(prop('tone')) || 'neutral'}` }, content),
	)
	.recipe('grid', ({ children, prop }) =>
		h('div', { class: `pg-grid pg-grid-${Number(prop('maxColumns', 2)) || 2}` }, children),
	)
	.recipe('badge', ({ label, prop }) =>
		h('span', { class: `pg-badge tone-${text(prop('tone')) || 'neutral'}` }, label),
	)
	.recipe('value', ({ prop }) =>
		h('strong', { class: `pg-value tone-${text(prop('tone')) || 'neutral'}` }, text(prop('value'))),
	)
	.recipe('list', ({ childNodes, renderChildren }) =>
		h('div', { class: 'pg-list' }, renderChildren(childNodes)),
	)
	.recipe('item', ({ childNodes, renderChildren, props, dispatch }) =>
		h(
			'button',
			{
				class: `pg-list-item ${props.action ? 'is-clickable' : ''}`,
				disabled: !props.action,
				onClick: () => props.action && dispatch(props.action as ActionRef),
			},
			h('span', {}, renderChildren(childNodes)),
		),
	)
	.fallback(({ children }) => children)
	.build();

export default function PlaygroundApp() {
	const [session, setSession] = useState<HostSession | null>(null);
	const [plugins, setPlugins] = useState<LoadedPlugin[]>([]);
	const [selectedPluginId, setSelectedPluginId] = useState(playgroundPluginAssets[0]?.id ?? '');
	const [routeTo, setRouteTo] = useState<string | null>(null);
	const [tabsView, setTabsView] = useState<RouteTabsView | null>(null);
	const [store, setStore] = useState<ScreenStore | null>(null);
	const [error, setError] = useState<string | null>(null);
	const [events, setEvents] = useState<EventLogEntry[]>([]);
	const eventId = useRef(1);
	const storeRef = useRef<ScreenStore | null>(null);
	const selectedPluginIdRef = useRef(selectedPluginId);
	const routeRef = useRef<ResolvedRoute | null>(null);
	const pluginsRef = useRef<LoadedPlugin[]>([]);
	const sessionRef = useRef<HostSession | null>(null);

	const selected = useMemo(
		() => plugins.find((entry) => entry.asset.id === selectedPluginId),
		[plugins, selectedPluginId],
	);

	const activeRoutePattern = useMemo(
		() =>
			selected
				? resolvePluginRoute(selected, routeTo ?? defaultRouteFor(selected))?.pattern
				: undefined,
		[selected, routeTo],
	);

	const appendEvent = useCallback((entry: Omit<EventLogEntry, 'id'>) => {
		setEvents((current) => [{ ...entry, id: eventId.current++ }, ...current].slice(0, 10));
	}, []);

	const applyRoute = useCallback((to: string): ResolvedPlaygroundRoute | undefined => {
		const resolved = resolveAcrossPlugins(pluginsRef.current, to);
		if (!resolved) return undefined;
		setSelectedPluginId(resolved.entry.asset.id);
		setRouteTo(to);
		return resolved;
	}, []);

	const navigateTo = useCallback(
		(to: string) => {
			const resolved = resolveAcrossPlugins(pluginsRef.current, to);
			if (!resolved) {
				appendEvent({
					action: 'navigate',
					targetPluginId: selectedPluginIdRef.current,
					message: `No playground route matches ${to}.`,
				});
				return;
			}

			appendEvent({
				action: 'navigate',
				targetPluginId: resolved.entry.asset.id,
				message: `Navigated to ${to}.`,
			});
			// The URL hash is the source of truth; the hashchange listener
			// applies the state (and gives us browser history for free).
			if (window.location.hash !== `#${to}`) {
				window.location.hash = to;
			} else {
				applyRoute(to);
			}
		},
		[appendEvent, applyRoute],
	);

	// Deep links and browser back/forward.
	useEffect(() => {
		const onHashChange = () => {
			const to = parseHashRoute();
			if (to) applyRoute(to);
		};
		window.addEventListener('hashchange', onHashChange);
		return () => window.removeEventListener('hashchange', onHashChange);
	}, [applyRoute]);

	const mountActivePlugin = useCallback(async (seedState: Record<string, unknown> = {}) => {
		const activeSession = sessionRef.current;
		const loaded = pluginsRef.current;
		const active = loaded.find((entry) => entry.asset.id === selectedPluginIdRef.current);
		if (!activeSession || !active) return;

		// Resolve the requested path against the plugin's declared routes so
		// multi-screen plugins render the right screen.
		const route: ResolvedRoute = resolvePluginRoute(
			active,
			routeTo ?? defaultRouteFor(active),
		) ?? {
			pattern: active.asset.routePattern,
			params: {},
			query: {},
		};
		routeRef.current = route;
		activeSession.setRoute(route);

		const data = active.plugin.load({ route, locale });
		const screen = active.plugin.render({ route, data, stateSnapshot: seedState, locale });
		const stateSnapshot = {
			...((screen as { initialState?: Record<string, unknown> }).initialState ?? {}),
			...seedState,
		};

		// Route tabs are host chrome derived from the manifest's route groups;
		// dynamic labels/badges resolve against the state snapshot.
		setTabsView(routeTabsView(active.envelope.manifest, route.pattern, stateSnapshot) ?? null);

		const slotResponses: SlotResponseEnvelope[] = [];
		for (const contributor of loaded) {
			for (const contribution of contributor.envelope.manifest.slotContributions ?? []) {
				slotResponses.push({
					pluginId: contributor.envelope.manifest.id,
					contributionId: contribution.id,
					response: contributor.plugin.renderSlot({
						contributionId: contribution.id,
						slotName: contribution.target,
						route,
						stateSnapshot,
						locale,
					}),
				});
			}
		}

		const ir = activeSession.mountWithSlots(
			screen,
			seedState,
			loaded.map((entry) => entry.envelope.manifest),
			slotResponses,
		);
		const nextStore = new ScreenStore(ir);
		nextStore.applyPatches(activeSession.initialPatches());
		storeRef.current = nextStore;
		setStore(nextStore);
	}, [routeTo]);

	useEffect(() => {
		selectedPluginIdRef.current = selectedPluginId;
	}, [selectedPluginId]);

	useEffect(() => {
		pluginsRef.current = plugins;
	}, [plugins]);

	useEffect(() => {
		sessionRef.current = session;
	}, [session]);

	useEffect(() => {
		(async () => {
			try {
				const nextSession = await HostSession.create(webHostModule as never, webHostWasmUrl, locale);
				const loaded = await Promise.all(
					playgroundPluginAssets.map(async (asset) => {
						const sink = new StateWriteSink();
						const plugin = await PluginInstance.instantiate(fetch(asset.wasmUrl), sink.handler);
						const envelope = plugin.manifest<PluginManifestEnvelope>();
						return { asset, plugin, sink, envelope };
					}),
				);
				sessionRef.current = nextSession;
				pluginsRef.current = loaded;
				setSession(nextSession);
				setPlugins(loaded);
				// Honor a deep link like /playground#/plugins/sanity-check/inspect.
				const to = parseHashRoute();
				if (to) applyRoute(to);
			} catch (cause) {
				setError(cause instanceof Error ? cause.message : String(cause));
			}
		})();
	}, [applyRoute]);

	useEffect(() => {
		// Seed remounts with the session snapshot so plugin state survives
		// navigation between the plugin's own routes (mirrors the TUI host,
		// where PluginState lives for the whole app session).
		void mountActivePlugin(sessionRef.current?.stateSnapshot() ?? {}).catch((cause) =>
			setError(cause instanceof Error ? cause.message : String(cause)),
		);
	}, [mountActivePlugin, selectedPluginId, plugins.length, session]);

	const handleAction = useCallback(
		(action: ActionRef) => {
			const activeSession = sessionRef.current;
			const currentStore = storeRef.current;
			const route = routeRef.current;
			if (!activeSession || !currentStore || !route) return;

			// Core navigate actions are host concerns: route them against the
			// plugin's declared routes instead of dispatching into the plugin.
			if (action.t === 'navigate') {
				const to = text((action.p as Record<string, unknown> | undefined)?.to);
				if (to) {
					navigateTo(to);
					return;
				}
			}

			const targetPluginId = action.originPluginId ?? selectedPluginIdRef.current;
			const target = pluginsRef.current.find((entry) => entry.envelope.manifest.id === targetPluginId);
			if (!target) {
				appendEvent({
					action: action.t,
					targetPluginId,
					message: 'Dispatch blocked: no plugin target is registered for this origin.',
					originContributionId: action.originContributionId,
				});
				return;
			}

			const response = target.plugin.dispatch<DispatchResponse>({
				route,
				action: { type: action.t, ...(action.p ? { params: action.p } : {}) },
				stateSnapshot: activeSession.stateSnapshot(),
				locale,
			});
			const writes = target.sink.drain();
			if (Object.keys(writes).length > 0) {
				currentStore.applyPatches(activeSession.applyWrites(writes));
				// State writes may feed dynamic route-tab labels/badges.
				const activeEntry = pluginsRef.current.find(
					(entry) => entry.asset.id === selectedPluginIdRef.current,
				);
				if (activeEntry) {
					setTabsView(
						routeTabsView(
							activeEntry.envelope.manifest,
							route.pattern,
							activeSession.stateSnapshot(),
						) ?? null,
					);
				}
			}

			appendEvent({
				action: action.t,
				targetPluginId,
				message: response?.message ?? 'Action dispatched to plugin.',
				originContributionId: action.originContributionId,
			});

			if (response?.outcome?.kind === 'refreshCurrentScreen') {
				void mountActivePlugin(activeSession.stateSnapshot()).catch((cause) =>
					setError(cause instanceof Error ? cause.message : String(cause)),
				);
			}

			if (response?.outcome?.kind === 'navigate' && response.outcome.to) {
				navigateTo(response.outcome.to);
			}
		},
		[appendEvent, mountActivePlugin, navigateTo],
	);

	if (error) {
		return <pre className="pg-error">{error}</pre>;
	}

	return (
		<div className="playground-shell">
			<aside className="pg-sidebar" aria-label="Playground plugins">
				<div className="pg-brand">
					<a href="/">Unode</a>
					<span>WASM Playground</span>
				</div>
				<div className="pg-plugin-list">
					{playgroundPluginAssets.map((asset) => (
						<button
							key={asset.id}
							type="button"
							className={asset.id === selectedPluginId ? 'is-selected' : ''}
							onClick={() => {
								const entry = pluginsRef.current.find((loaded) => loaded.asset.id === asset.id);
								if (entry) {
									navigateTo(defaultRouteFor(entry));
								} else {
									setRouteTo(null);
									setSelectedPluginId(asset.id);
								}
							}}
						>
							<strong>{asset.name}</strong>
							<span>{asset.sourcePath}</span>
						</button>
					))}
				</div>
			</aside>

			<main className="pg-main">
				{tabsView && (
					<div className="pg-tabs" role="tablist">
						{tabsView.tabs.map((tab) => (
							<button
								key={tab.to}
								type="button"
								role="tab"
								aria-selected={tab.to === tabsView.active}
								className={tab.to === tabsView.active ? 'is-active' : ''}
								onClick={() => navigateTo(tab.to)}
							>
								<span>{tab.label}</span>
								{tab.badge && <small>{tab.badge}</small>}
							</button>
						))}
					</div>
				)}
				{store ? (
					<UnodeScreen
						store={store}
						onAction={handleAction}
						renderer={renderer}
						components={{ Button: PluginButton }}
					/>
				) : (
					<p className="pg-loading">Loading plugin WASM...</p>
				)}
			</main>

			<aside className="pg-details" aria-label="Plugin details">
				<section>
					<h2>{selected?.envelope.manifest.name ?? 'Loading'}</h2>
					<p>{selected?.asset.sourcePath}</p>
					<div className="pg-tags">
						{(selected?.asset.tags ?? []).map((tag) => <span key={tag}>{tag}</span>)}
					</div>
				</section>
				<section>
					<h2>Permissions</h2>
					<div className="pg-permission-list">
						{((selected?.envelope.manifest as PluginManifest | undefined)?.permissions ?? []).map((permission) => (
							<span key={permission.permission}>{permission.permission}</span>
						))}
					</div>
				</section>
				<section>
					<h2>Routes</h2>
					<div className="pg-route-list">
						{(selected?.envelope.manifest.routes ?? []).map((route) => (
							<button
								key={route.pattern}
								type="button"
								className={route.pattern === activeRoutePattern ? 'is-active' : ''}
								disabled={route.pattern.includes(':')}
								title={route.screenKind}
								onClick={() => navigateTo(route.pattern)}
							>
								{route.pattern}
							</button>
						))}
						{!selected?.envelope.manifest.routes?.length && (
							<p>No declared routes; using {selected?.asset.routePattern ?? 'registry pattern'}.</p>
						)}
					</div>
				</section>
				<section>
					<h2>Slot Contributions</h2>
					<div className="pg-permission-list">
						{(selected?.envelope.manifest.slotContributions ?? []).map((contribution) => (
							<span key={contribution.id}>{contribution.target}</span>
						))}
						{!selected?.envelope.manifest.slotContributions?.length && <p>No slot contributions.</p>}
					</div>
				</section>
				<section>
					<h2>Event Log</h2>
					<div className="pg-event-log">
						{events.length === 0 && <p>No actions yet.</p>}
						{events.map((event) => (
							<div key={event.id}>
								<strong>{event.action}</strong>
								<span>{event.message}</span>
								<small>target: {event.targetPluginId}</small>
								{event.originContributionId && <small>contribution: {event.originContributionId}</small>}
							</div>
						))}
					</div>
				</section>
			</aside>
		</div>
	);
}
