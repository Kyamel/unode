import { definePlugin, route, UNODE_CORE_API_VERSION } from '$lib/unode/core/runtime';
import type { MugenHostApi } from '$lib/plugins-bridge/host';
import type { WorkDetails } from '$lib/plugins-bridge/models';
import en from './messages/en.json';
import ptBr from './messages/pt-br.json';
import { loadWorkMeta } from './data';
import { buildWorkDetailsScreen, buildWorkNotFoundScreen } from './ui';

const ROUTE = '/mangas/:mangaId/meta';

type WorkMetaData = {
	work: WorkDetails | null;
	error: string | null;
};

export default definePlugin<MugenHostApi>({
	manifest: {
		id: 'org.mugens.core.work',
		name: 'Work Details',
		version: '2.0.0',
		apiVersion: UNODE_CORE_API_VERSION,
		permissions: [
			{
				permission: 'catalog.read',
				reason: 'Load work details from the catalog.'
			},
			{
				permission: 'navigation.write',
				reason: 'Open work detail tabs.'
			},
			{
				permission: 'storage.persistent.read',
				reason: 'Reuse cached work details between visits.'
			},
			{
				permission: 'storage.persistent.write',
				reason: 'Persist cached work details between visits.'
			}
		]
	},
	i18n: {
		en,
		'pt-br': ptBr
	},
	routes: [
		route<MugenHostApi>(ROUTE)
			.load<WorkMetaData>(async ({ api, pluginId, route }) => {
				const workId = route.params.mangaId;
				return await loadWorkMeta(api, pluginId, workId);
			})
			.render((data, renderCtx) => {
				if (!data.work) {
					return buildWorkNotFoundScreen(renderCtx.i18n.t, data.error);
				}

				return buildWorkDetailsScreen({
					t: renderCtx.i18n.t,
					work: data.work,
					error: data.error
				});
			})
	]
});
