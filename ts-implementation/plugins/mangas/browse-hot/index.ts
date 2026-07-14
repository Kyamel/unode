import type { CollectionContinuation } from '$lib/unode/core/ast';
import { definePlugin, msg, route, UNODE_CORE_API_VERSION } from '$lib/unode/core/runtime';
import type { MugenHostApi } from '$lib/plugins-bridge/host';
import type { WorkSummary } from '$lib/plugins-bridge/models';
import en from './messages/en.json';
import ptBr from './messages/pt-br.json';
import { loadBrowseWorksPage } from './data';
import { buildBrowseScreen } from './ui';

const PAGE_SIZE = 24;
const STORAGE_KEY = 'browse-hot-state';
const LOAD_MORE_ACTION_ID = 'mangas.hot.load_more';
const ROUTE = '/mangas/hot';

type BrowseHotState = {
	works: WorkSummary[];
	lastCursor: string | null;
	filterKey: string | null;
};

type BrowseHotData = {
	works: WorkSummary[];
	error: string | null;
	nextCursor: string | null;
};

function mergeWorks(current: WorkSummary[], next: WorkSummary[]): WorkSummary[] {
	const seen = new Set(current.map((work) => work.id));
	const merged = [...current];

	for (const work of next) {
		if (seen.has(work.id)) continue;
		seen.add(work.id);
		merged.push(work);
	}

	return merged;
}

export default definePlugin<MugenHostApi>({
	manifest: {
		id: 'org.mugens.core.mangas.hot',
		name: 'Mangas Hot',
		version: '2.0.0',
		apiVersion: UNODE_CORE_API_VERSION,
		permissions: [
			{
				permission: 'catalog.read',
				reason: 'Load manga collections from the catalog.'
			},
			{
				permission: 'navigation.write',
				reason: 'Open manga browse routes and detail screens.'
			},
			{
				permission: 'storage.persistent.read',
				reason: 'Reuse the current browse page between remote continuation refreshes.'
			},
			{
				permission: 'storage.persistent.write',
				reason: 'Persist the current browse page between remote continuation refreshes.'
			},
			{
				permission: 'events.write',
				reason: 'Refresh the current screen after loading another remote page.'
			}
		]
	},
	i18n: {
		en,
		'pt-br': ptBr
	},
	navigation: [
		{
			id: 'mangas.hot',
			label: msg('nav_mangas'),
			shortLabel: msg('nav_mangas_short'),
			to: ROUTE,
			priority: 100,
			section: 'main'
		},
	],
	commands: [
		{
			id: 'mangas.hot',
			title: msg('catalog_tab_hot'),
			category: msg('nav_mangas'),
			run: ({ host }) => host.navigation.navigate(ROUTE)
		}
	],
	actions: [
		{
			id: LOAD_MORE_ACTION_ID,
			title: msg('action_load_more'),
			async run({ action, host, pluginId, i18n }) {
				const t = i18n.t;
				const cursor = typeof action.params?.cursor === 'string' ? action.params.cursor : null;
				if (!cursor) return;

				const stored = await host.storage.getScoped<BrowseHotState>(pluginId, STORAGE_KEY);
				const page = await loadBrowseWorksPage(host, { limit: PAGE_SIZE, cursor });

				if (page.error) {
					host.feedback.toast({
						title: t('action_load_more'),
						message: page.error,
						tone: 'danger'
					});
					return;
				}

				const filterKey = page.filterKey ?? stored?.filterKey ?? null;
				const baseline = stored && stored.filterKey === filterKey ? stored.works : [];
				await host.storage.setScoped(pluginId, STORAGE_KEY, {
					works: mergeWorks(baseline, page.works),
					lastCursor: page.lastCursor,
					filterKey
				} satisfies BrowseHotState);

				host.events.emit({
					type: 'screen.refresh',
					pathname: ROUTE
				});
			}
		}
	],
	routes: [
		route<MugenHostApi>(ROUTE)
			.load<BrowseHotData>(async ({ api, pluginId }) => {
				const stored = await api.storage.getScoped<BrowseHotState>(pluginId, STORAGE_KEY);
				const page = await loadBrowseWorksPage(api, { limit: PAGE_SIZE });
				const filterKey = page.filterKey ?? stored?.filterKey ?? null;

				if (page.error && stored?.works.length) {
					return {
						works: stored.works,
						error: page.error,
						nextCursor: stored.lastCursor
					} satisfies BrowseHotData;
				}

				if (!stored || stored.filterKey !== filterKey) {
					await api.storage.setScoped(pluginId, STORAGE_KEY, {
						works: page.works,
						lastCursor: page.lastCursor,
						filterKey
					} satisfies BrowseHotState);

					return {
						works: page.works,
						error: page.error,
						nextCursor: page.lastCursor
					} satisfies BrowseHotData;
				}

				return {
					works: stored.works,
					error: null,
					nextCursor: stored.lastCursor
				} satisfies BrowseHotData;
			})
			.render((data, renderCtx) => {
				const t = renderCtx.i18n.t;
				const continuation: CollectionContinuation | undefined = data.nextCursor
					? {
							kind: 'remote',
							hasMore: true,
							loadMore: {
								type: LOAD_MORE_ACTION_ID,
								params: {
									cursor: data.nextCursor
								}
							},
							label: t('action_load_more'),
							loadingLabel: t('action_loading')
						}
					: undefined;

				return buildBrowseScreen({
					t,
					works: data.works,
					error: data.error,
					emptyText: t('catalog_empty_hot'),
					activeTab: 'hot',
					continuation
				});
			}),
	],
});
