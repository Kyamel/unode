import { definePlugin, msg, route, UNODE_CORE_API_VERSION } from '$lib/unode/core/runtime';
import type { MugenHostApi } from '$lib/plugins-bridge/host';
import type { ChapterSummary } from '$lib/plugins-bridge/models';
import en from './messages/en.json';
import ptBr from './messages/pt-br.json';
import { loadWorkChapters } from './data';
import { buildChaptersScreen } from './ui';

const ROUTE = '/mangas/:mangaId/chapters';

type WorkChaptersData = {
	workId: string;
	chapters: ChapterSummary[];
	error: string | null;
};

function normalizeLanguage(language: string | null | undefined): string | null {
	if (!language) return null;
	const normalized = language.trim().toLowerCase();
	return normalized || null;
}

function uniqueLanguages(chapters: ChapterSummary[]): string[] {
	return [...new Set(chapters.map((chapter) => normalizeLanguage(chapter.language_code)).filter(Boolean))].sort() as string[];
}

export default definePlugin<MugenHostApi>({
	manifest: {
		id: 'org.mugens.core.work.chapters',
		name: 'Work Chapters',
		version: '2.0.0',
		apiVersion: UNODE_CORE_API_VERSION,
		permissions: [
			{
				permission: 'catalog.read',
				reason: 'Load chapters for the selected work.'
			},
			{
				permission: 'navigation.write',
				reason: 'Open work tabs and reading screens.'
			}
		]
	},
	i18n: {
		en,
		'pt-br': ptBr
	},
	commands: [
		{
			id: 'work.chapters',
			title: msg('manga_tab_chapters'),
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
			.load<WorkChaptersData>(async ({ api, route }) => {
				const workId = route.params.mangaId;
				const result = await loadWorkChapters(api, workId);
				return {
					workId,
					...result
				} satisfies WorkChaptersData;
			})
			.render((data, renderCtx) => {
				const availableLanguages = uniqueLanguages(data.chapters);
				const requestedLanguage = normalizeLanguage(renderCtx.route.query.language);
				const selectedLanguage =
					requestedLanguage && availableLanguages.includes(requestedLanguage) ? requestedLanguage : 'all';
				const chapters =
					selectedLanguage === 'all'
						? data.chapters
						: data.chapters.filter(
								(chapter) => normalizeLanguage(chapter.language_code) === selectedLanguage
							);

				return buildChaptersScreen({
					t: renderCtx.i18n.t,
					workId: data.workId,
					routePath: `${ROUTE.replace(':mangaId', data.workId)}`,
					languages: availableLanguages,
					selectedLanguage,
					chapters,
					error: data.error
				});
			})
	]
});
