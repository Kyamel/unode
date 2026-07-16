import webCounterWasm from './wasm/web_counter_plugin.wasm?url';
import sanityCheckWasm from './wasm/sanity_check_plugin.wasm?url';
import hostComponentsWasm from './wasm/host_components_plugin.wasm?url';
import actionsWasm from './wasm/actions_plugin.wasm?url';
import routeTabsWasm from './wasm/route_tabs_plugin.wasm?url';
import splitHostWasm from './wasm/slot_host_plugin.wasm?url';
import splitContributorWasm from './wasm/slot_contributor_plugin.wasm?url';
import complexLayoutWasm from './wasm/layout_plugin.wasm?url';
import complexStateWasm from './wasm/state_collections_plugin.wasm?url';

export type PluginManifestEnvelope = {
	abiVersion: string;
	manifest: PluginManifest;
};

import type { ManifestRouteDecl, ManifestRouteGroupDecl } from 'unode-web-core';

export type PluginRouteDecl = ManifestRouteDecl;
export type PluginRouteGroupDecl = ManifestRouteGroupDecl;

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
	/** Named route sets with a navigation intent (e.g. tabs). */
	routeGroups?: PluginRouteGroupDecl[];
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
		id: 'dev.unode.counter',
		name: 'Counter',
		routePattern: '/plugins/counter',
		sourcePath: 'plugins/counter',
		wasmUrl: webCounterWasm,
		tags: ['state basics', 'reactivity'],
	},
	{
		id: 'dev.unode.sanity-check',
		name: 'Sanity Check',
		routePattern: '/plugins/sanity-check',
		sourcePath: 'plugins/sanity-check',
		wasmUrl: sanityCheckWasm,
		tags: ['full ABI contract'],
	},
	{
		id: 'dev.unode.actions',
		name: 'Actions & Outcomes',
		routePattern: '/plugins/actions',
		sourcePath: 'plugins/actions',
		wasmUrl: actionsWasm,
		tags: ['one capability', 'dispatch outcomes'],
	},
	{
		id: 'dev.unode.host-components',
		name: 'Host Components',
		routePattern: '/plugins/host-components',
		sourcePath: 'plugins/host-components',
		wasmUrl: hostComponentsWasm,
		tags: ['one capability', 'host components'],
	},
	{
		id: 'dev.unode.route-tabs',
		name: 'Route Tabs',
		routePattern: '/plugins/route-tabs',
		sourcePath: 'plugins/route-tabs',
		wasmUrl: routeTabsWasm,
		tags: ['one capability', 'route groups', 'dynamic badge'],
	},
	{
		id: 'dev.unode.slot-host',
		name: 'Slot Host',
		routePattern: '/plugins/slot-host',
		sourcePath: 'plugins/slot-host',
		wasmUrl: splitHostWasm,
		tags: ['one capability', 'slots (host side)'],
	},
	{
		id: 'dev.unode.slot-contributor',
		name: 'Slot Contributor',
		routePattern: '/plugins/slot-contributor',
		sourcePath: 'plugins/slot-contributor',
		wasmUrl: splitContributorWasm,
		tags: ['one capability', 'slots (guest side)', 'trust boundary'],
	},
	{
		id: 'dev.unode.layout',
		name: 'Layout',
		routePattern: '/plugins/layout',
		sourcePath: 'plugins/layout',
		wasmUrl: complexLayoutWasm,
		tags: ['one capability', 'layout vocabulary'],
	},
	{
		id: 'dev.unode.state-collections',
		name: 'State Collections',
		routePattern: '/plugins/state-collections',
		sourcePath: 'plugins/state-collections',
		wasmUrl: complexStateWasm,
		tags: ['one capability', 'collections', 'granular patches'],
	},
];
