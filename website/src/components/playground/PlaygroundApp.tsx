// The playground orchestrator. Each concern lives in its own module:
//
// - runtime.ts      — wasm host session bootstrap
// - plugins.ts      — plugin loading + manifest-route resolution
// - recipes.ts      — how each node type looks (styling)
// - renderer.ts     — the engine assembled from those recipes
// - Button.tsx      — the host component behind `hostSlot("Button")`
// - router.ts       — hash routing (deep links, history, plugin selection)
// - Sidebar.tsx     — left column: plugin catalog
// - PluginShell.tsx — center: route-tab chrome + mounted plugin screen
// - DetailsPanel.tsx— right column: manifest x-ray + event log
//
// What remains here is the host-shell state machine: mounting the active
// plugin and dispatching actions back into the sandbox.
import { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import { ScreenStore, type ActionRef } from 'unode-react';
import { routeTabsView, type HostSession, type ResolvedRoute, type RouteTabsView } from 'unode-web-core';

import { playgroundPluginAssets } from '../../playground/registry';
import { DetailsPanel, type EventLogEntry } from './DetailsPanel';
import { PluginShell } from './PluginShell';
import { Sidebar } from './Sidebar';
import {
	defaultRouteFor,
	loadPlugins,
	parseHashRoute,
	resolvePluginRoute,
	type LoadedPlugin,
} from './plugins';
import { usePlaygroundRouter } from './router';
import { text } from './recipes';
import { LOCALE, createHostSession } from './runtime';
import './playground.css';

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

export default function PlaygroundApp() {
	const [session, setSession] = useState<HostSession | null>(null);
	const [plugins, setPlugins] = useState<LoadedPlugin[]>([]);
	const [tabsView, setTabsView] = useState<RouteTabsView | null>(null);
	const [store, setStore] = useState<ScreenStore | null>(null);
	const [error, setError] = useState<string | null>(null);
	const [events, setEvents] = useState<EventLogEntry[]>([]);
	const eventId = useRef(1);
	const storeRef = useRef<ScreenStore | null>(null);
	const routeRef = useRef<ResolvedRoute | null>(null);
	const pluginsRef = useRef<LoadedPlugin[]>([]);
	const sessionRef = useRef<HostSession | null>(null);

	const appendEvent = useCallback((entry: Omit<EventLogEntry, 'id'>) => {
		setEvents((current) => [{ ...entry, id: eventId.current++ }, ...current].slice(0, 10));
	}, []);

	const { selectedPluginId, selectedPluginIdRef, routeTo, applyRoute, navigateTo, selectPlugin } =
		usePlaygroundRouter(
			pluginsRef,
			playgroundPluginAssets[0]?.id ?? '',
			useCallback(
				(targetPluginId: string, message: string) =>
					appendEvent({ action: 'navigate', targetPluginId, message }),
				[appendEvent],
			),
		);

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

		const data = active.plugin.load({ route, locale: LOCALE });
		const screen = active.plugin.render({ route, data, stateSnapshot: seedState, locale: LOCALE });
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
						locale: LOCALE,
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
	}, [routeTo, selectedPluginIdRef]);

	useEffect(() => {
		pluginsRef.current = plugins;
	}, [plugins]);

	useEffect(() => {
		sessionRef.current = session;
	}, [session]);

	useEffect(() => {
		(async () => {
			try {
				const nextSession = await createHostSession();
				const loaded = await loadPlugins();
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
				locale: LOCALE,
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
		[appendEvent, mountActivePlugin, navigateTo, selectedPluginIdRef],
	);

	if (error) {
		return <pre className="pg-error">{error}</pre>;
	}

	return (
		<div className="playground-shell">
			<Sidebar selectedPluginId={selectedPluginId} onSelect={selectPlugin} />
			<PluginShell store={store} tabsView={tabsView} onAction={handleAction} onNavigate={navigateTo} />
			<DetailsPanel
				selected={selected}
				activeRoutePattern={activeRoutePattern}
				events={events}
				onNavigate={navigateTo}
			/>
		</div>
	);
}
