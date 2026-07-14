import { definePlugin, msg, route, UNODE_CORE_API_VERSION } from '$lib/unode/core/runtime';
import type { MugenHostApi } from '$lib/plugins-bridge/host';
import type { WorkStaff } from '$lib/plugins-bridge/models';
import en from './messages/en.json';
import ptBr from './messages/pt-br.json';
import { loadWorkStaff } from './data';
import { buildStaffScreen } from './ui';

const ROUTE = '/mangas/:mangaId/staff';

type WorkStaffData = {
	workId: string;
	staff: WorkStaff[];
	error: string | null;
};

export default definePlugin<MugenHostApi>({
	manifest: {
		id: 'org.mugens.core.work.staff',
		name: 'Work Staff',
		version: '2.0.0',
		apiVersion: UNODE_CORE_API_VERSION,
		permissions: [
			{
				permission: 'catalog.read',
				reason: 'Load staff credits for the selected work.'
			},
			{
				permission: 'navigation.write',
				reason: 'Open work detail tabs.'
			}
		]
	},
	i18n: {
		en,
		'pt-br': ptBr
	},
	commands: [
		{
			id: 'work.staff',
			title: msg('manga_tab_staff'),
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
			.load<WorkStaffData>(async ({ api, route }) => {
				const workId = route.params.mangaId;
				const result = await loadWorkStaff(api, workId);
				return {
					workId,
					...result
				} satisfies WorkStaffData;
			})
			.render((data, renderCtx) => {
				return buildStaffScreen({
					t: renderCtx.i18n.t,
					workId: data.workId,
					staff: data.staff,
					error: data.error
				});
			})
	]
});
