import webCounterWasm from './wasm/web_counter_plugin.wasm?url';
import sanityCheckWasm from './wasm/sanity_check_plugin.wasm?url';
import hostSlotWasm from './wasm/playground_host_slot_plugin.wasm?url';
import routeTabsWasm from './wasm/playground_route_tabs_plugin.wasm?url';
import splitHostWasm from './wasm/playground_split_host_plugin.wasm?url';
import splitContributorWasm from './wasm/playground_split_contributor_plugin.wasm?url';
import complexLayoutWasm from './wasm/playground_complex_layout_plugin.wasm?url';
import complexStateWasm from './wasm/playground_complex_state_plugin.wasm?url';

export type PluginManifestEnvelope = {
	abiVersion: string;
	manifest: PluginManifest;
};

export type PluginRouteDecl = {
	pattern: string;
	screenKind?: string;
	priority?: number;
};

export type PluginManifest = {
	id: string;
	name: string;
	version: string;
	description?: string;
	permissions?: Array<{ permission: string; required?: boolean; reason?: string }>;
	slotContributions?: Array<{
		id: string;
		target: string;
		priority?: number;
		when?: unknown;
	}>;
	/** Screens the plugin declares; the playground routes navigations against these. */
	routes?: PluginRouteDecl[];
};

export type PlaygroundPluginAsset = {
	id: string;
	name: string;
	routePattern: string;
	sourcePath: string;
	wasmUrl: string;
	tags: string[];
};

export const playgroundPluginAssets: PlaygroundPluginAsset[] = [
	{
		id: 'dev.unode.web-counter',
		name: 'Web Counter',
		routePattern: '/plugins/web-counter',
		sourcePath: 'plugins/web-counter',
		wasmUrl: webCounterWasm,
		tags: ['real plugin', 'state', 'reactivity'],
	},
	{
		id: 'dev.mugens.sanity-check',
		name: 'Sanity Check',
		routePattern: '/plugins/sanity-check',
		sourcePath: 'plugins/sanity-check',
		wasmUrl: sanityCheckWasm,
		tags: ['real plugin', 'route tabs', 'abi'],
	},
	{
		id: 'dev.unode.playground.host-slot',
		name: 'Host Slot Buttons',
		routePattern: '/playground/host-slot',
		sourcePath: 'plugins/playground-host-slot',
		wasmUrl: hostSlotWasm,
		tags: ['host slot', 'button intents'],
	},
	{
		id: 'dev.unode.playground.route-tabs',
		name: 'Route Tabs',
		routePattern: '/playground/route-tabs',
		sourcePath: 'plugins/playground-route-tabs',
		wasmUrl: routeTabsWasm,
		tags: ['route tabs', 'navigation'],
	},
	{
		id: 'dev.unode.playground.split-host',
		name: 'Split Screen Host',
		routePattern: '/playground/split-host',
		sourcePath: 'plugins/playground-split-host',
		wasmUrl: splitHostWasm,
		tags: ['slots', 'host plugin'],
	},
	{
		id: 'dev.unode.playground.split-contributor',
		name: 'Split Contributor',
		routePattern: '/playground/split-contributor',
		sourcePath: 'plugins/playground-split-contributor',
		wasmUrl: splitContributorWasm,
		tags: ['slot contributor', 'trust boundary'],
	},
	{
		id: 'dev.unode.playground.complex-layout',
		name: 'Complex Layout',
		routePattern: '/playground/complex-layout',
		sourcePath: 'plugins/playground-complex-layout',
		wasmUrl: complexLayoutWasm,
		tags: ['layout', 'dashboard'],
	},
	{
		id: 'dev.unode.playground.complex-state',
		name: 'Complex State',
		routePattern: '/playground/complex-state',
		sourcePath: 'plugins/playground-complex-state',
		wasmUrl: complexStateWasm,
		tags: ['state', 'patches'],
	},
];
