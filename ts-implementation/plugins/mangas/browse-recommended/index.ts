import { definePlugin, msg, route, UNODE_CORE_API_VERSION } from '$lib/unode/core/runtime';
import type { MugenHostApi } from '$lib/plugins-bridge/host';
import type { WorkSummary } from '$lib/plugins-bridge/models';
import en from './messages/en.json';
import ptBr from './messages/pt-br.json';
import { loadBrowseWorks } from './data';
import { buildBrowseScreen } from './ui';

const ROUTE = '/mangas/recommended';

type BrowseRecommendedData = {
	works: WorkSummary[];
	error: string | null;
};

export default definePlugin<MugenHostApi>({
	manifest: {
		id: 'org.mugens.core.mangas.recommended',
		name: 'Mangas Recommended',
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
			}
		]
	},
	i18n: {
		en,
		'pt-br': ptBr
	},
	commands: [
		{
			id: 'mangas.recommended',
			title: msg('catalog_tab_recommended'),
			category: msg('nav_mangas'),
			run: ({ host }) => host.navigation.navigate(ROUTE)
		}
	],
	routes: [
		route<MugenHostApi>(ROUTE)
			.load<BrowseRecommendedData>(async ({ api }) => {
				return await loadBrowseWorks(api);
			})
			.render((data, renderCtx) => {
				const t = renderCtx.i18n.t;
				return buildBrowseScreen({
					t,
					works: data.works,
					error: data.error,
					emptyText: t('catalog_empty_recommended'),
					activeTab: 'recommended'
				});
			})
	]
});
