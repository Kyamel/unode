import { definePlugin, msg, route, UNODE_CORE_API_VERSION } from '$lib/unode/core/runtime';
import type { MugenHostApi } from '$lib/plugins-bridge/host';
import type { WorkRelation } from '$lib/plugins-bridge/models';
import en from './messages/en.json';
import ptBr from './messages/pt-br.json';
import { loadWorkRelations } from './data';
import { buildRelatedScreen } from './ui';

const ROUTE = '/mangas/:mangaId/related';

type WorkRelatedData = {
	workId: string;
	relations: WorkRelation[];
	error: string | null;
};

export default definePlugin<MugenHostApi>({
	manifest: {
		id: 'org.mugens.core.work.related',
		name: 'Work Related',
		version: '2.0.0',
		apiVersion: UNODE_CORE_API_VERSION,
		permissions: [
			{
				permission: 'catalog.read',
				reason: 'Load related works for the selected work.'
			},
			{
				permission: 'navigation.write',
				reason: 'Open related work detail screens and tabs.'
			}
		]
	},
	i18n: {
		en,
		'pt-br': ptBr
	},
	commands: [
		{
			id: 'work.related',
			title: msg('manga_tab_related'),
			category: msg('nav_mangas'),
			run: ({ host, route }) => {
				const workId = route?.params?.mangaId;
				if (!workId) return;
				return host.navigation.navigate(`${ROUTE.replace(':mangaId', workId)}`);
			}
		}
	],
	routes: [
		route<MugenHostApi>(ROUTE)
			.load<WorkRelatedData>(async ({ api, route }) => {
				const workId = route.params.mangaId;
				const result = await loadWorkRelations(api, workId);
				return {
					workId,
					...result
				} satisfies WorkRelatedData;
			})
			.render((data, renderCtx) => {
				return buildRelatedScreen({
					t: renderCtx.i18n.t,
					workId: data.workId,
					relations: data.relations,
					error: data.error
				});
			})
	]
});
