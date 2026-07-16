// Plugin loading and route resolution — the host-shell half of the
// playground. Every example plugin wasm is instantiated up front (each with
// its own state-write sink), and navigation targets are resolved against the
// routes their manifests declare, exact matches winning over `:param` ones.
import {
	PluginInstance,
	StateWriteSink,
	resolveRoutePattern,
	type ResolvedRoute,
} from 'unode-web-core';

import {
	playgroundPluginAssets,
	type PluginManifestEnvelope,
	type PlaygroundPluginAsset,
} from '../../playground/registry';

export type LoadedPlugin = {
	asset: PlaygroundPluginAsset;
	plugin: PluginInstance;
	sink: StateWriteSink;
	envelope: PluginManifestEnvelope;
};

export type ResolvedPlaygroundRoute = { entry: LoadedPlugin; route: ResolvedRoute };

const ROUTE_BASE = 'https://unode.dev';

/** Instantiates every registered plugin wasm and reads its manifest. */
export function loadPlugins(): Promise<LoadedPlugin[]> {
	return Promise.all(
		playgroundPluginAssets.map(async (asset) => {
			const sink = new StateWriteSink();
			const plugin = await PluginInstance.instantiate(fetch(asset.wasmUrl), sink.handler);
			const envelope = plugin.manifest<PluginManifestEnvelope>();
			return { asset, plugin, sink, envelope };
		}),
	);
}

/**
 * Route patterns the plugin answers for: the ones declared in its manifest,
 * plus the registry's legacy pattern as a fallback for plugins that declare
 * none.
 */
export function routePatternsFor(entry: LoadedPlugin): string[] {
	const declared = (entry.envelope.manifest.routes ?? []).map((route) => route.pattern);
	if (declared.includes(entry.asset.routePattern)) return declared;
	return [entry.asset.routePattern, ...declared];
}

/** The first static pattern is the plugin's landing screen. */
export function defaultRouteFor(entry: LoadedPlugin): string {
	return (
		routePatternsFor(entry).find((pattern) => !pattern.includes(':')) ?? entry.asset.routePattern
	);
}

export function resolvePluginRoute(entry: LoadedPlugin, to: string): ResolvedRoute | undefined {
	const url = new URL(to, ROUTE_BASE);
	const match = resolveRoutePattern(routePatternsFor(entry), url.pathname);
	if (!match) return undefined;
	return {
		pattern: match.pattern,
		params: match.params,
		query: Object.fromEntries(url.searchParams.entries()),
	};
}

/**
 * Resolves a destination against every loaded plugin, like a host shell's
 * route registry: exact matches win over `:param` matches.
 */
export function resolveAcrossPlugins(
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
export function parseHashRoute(): string | null {
	const hash = window.location.hash.replace(/^#/, '');
	return hash.startsWith('/') ? hash : null;
}
