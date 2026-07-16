// The playground's router: the URL hash is the source of truth
// (`/playground#/plugins/...`), giving deep links and browser back/forward
// for free. Destinations are resolved against every loaded plugin's declared
// routes — selecting a plugin is just a navigation to its default route.
import { useCallback, useEffect, useRef, useState, type MutableRefObject } from 'react';

import {
	defaultRouteFor,
	parseHashRoute,
	resolveAcrossPlugins,
	type LoadedPlugin,
	type ResolvedPlaygroundRoute,
} from './plugins';

export function usePlaygroundRouter(
	pluginsRef: MutableRefObject<LoadedPlugin[]>,
	initialPluginId: string,
	onNavigate: (targetPluginId: string, message: string) => void,
) {
	const [selectedPluginId, setSelectedPluginId] = useState(initialPluginId);
	const [routeTo, setRouteTo] = useState<string | null>(null);
	const selectedPluginIdRef = useRef(selectedPluginId);

	useEffect(() => {
		selectedPluginIdRef.current = selectedPluginId;
	}, [selectedPluginId]);

	/** Applies an already-validated destination to the router state. */
	const applyRoute = useCallback((to: string): ResolvedPlaygroundRoute | undefined => {
		const resolved = resolveAcrossPlugins(pluginsRef.current, to);
		if (!resolved) return undefined;
		setSelectedPluginId(resolved.entry.asset.id);
		setRouteTo(to);
		return resolved;
	}, [pluginsRef]);

	const navigateTo = useCallback(
		(to: string) => {
			const resolved = resolveAcrossPlugins(pluginsRef.current, to);
			if (!resolved) {
				onNavigate(selectedPluginIdRef.current, `No playground route matches ${to}.`);
				return;
			}

			onNavigate(resolved.entry.asset.id, `Navigated to ${to}.`);
			// Writing the hash fires the hashchange listener below, which applies
			// the state — and records a history entry.
			if (window.location.hash !== `#${to}`) {
				window.location.hash = to;
			} else {
				applyRoute(to);
			}
		},
		[applyRoute, onNavigate, pluginsRef],
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

	/** Sidebar selection = navigate to the plugin's default route. */
	const selectPlugin = useCallback(
		(assetId: string) => {
			const entry = pluginsRef.current.find((loaded) => loaded.asset.id === assetId);
			if (entry) {
				navigateTo(defaultRouteFor(entry));
			} else {
				setRouteTo(null);
				setSelectedPluginId(assetId);
			}
		},
		[navigateTo, pluginsRef],
	);

	return { selectedPluginId, selectedPluginIdRef, routeTo, applyRoute, navigateTo, selectPlugin };
}
