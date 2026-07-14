import { definePlugin, msg, route, UNODE_CORE_API_VERSION } from '$lib/unode/core/runtime';
import type { MugenHostApi } from '$lib/plugins-bridge/host';
import type { ChapterSummary, WorkSummary } from '$lib/plugins-bridge/models';
import en from './messages/en.json';
import ptBr from './messages/pt-br.json';
import { buildChaptersLabScreen } from './ui';

const CHAPTERS_LAB_ROUTE = '/tests/chapters';

type ChaptersLabData = {
	work: WorkSummary;
	chapters: ChapterSummary[];
	error: string | null;
};

const fallbackWork: WorkSummary = {
	id: 'fallback-work',
	title: 'Blue Period',
	work_type: 'manga',
	status: 'ongoing',
	year: 2017
};

const fallbackChapters: ChapterSummary[] = [
	{
		id: 'chapter-en-1',
		number: '1',
		title: 'The Blue of Longing',
		language_code: 'en',
		volume: 1,
		sort_key: '0001'
	},
	{
		id: 'chapter-ja-1',
		number: '1',
		title: 'Aoi Akogare',
		language_code: 'ja',
		volume: 1,
		sort_key: '0002'
	},
	{
		id: 'chapter-pt-2',
		number: '2',
		title: 'Promessa Depois da Aula',
		language_code: 'pt-br',
		volume: 1,
		sort_key: '0003'
	},
	{
		id: 'chapter-en-3',
		number: '3',
		title: 'Brushes at Sunset',
		language_code: 'en',
		volume: 1,
		sort_key: '0004'
	},
	{
		id: 'chapter-es-4',
		number: '4',
		title: 'Luz en la Ventana',
		language_code: 'es',
		volume: 1,
		sort_key: '0005'
	},
	{
		id: 'chapter-ja-5',
		number: '5',
		title: 'Festival Night',
		language_code: 'ja',
		volume: 2,
		sort_key: '0006'
	},
	{
		id: 'chapter-en-6',
		number: '6',
		title: 'The Smell of Turpentine',
		language_code: 'en',
		volume: 2,
		sort_key: '0007'
	},
	{
		id: 'chapter-pt-7',
		number: '7',
		title: 'Noite de Atelier',
		language_code: 'pt-br',
		volume: 2,
		sort_key: '0008'
	},
	{
		id: 'chapter-ja-8',
		number: '8',
		title: 'Morning Critique',
		language_code: 'ja',
		volume: 3,
		sort_key: '0009'
	}
];

function toErrorMessage(error: unknown): string {
	if (error instanceof Error && error.message.trim()) return error.message;
	if (typeof error === 'string' && error.trim()) return error;
	return 'Unknown error.';
}

function normalizeLanguage(language: string | null | undefined): string | null {
	if (!language) return null;
	const normalized = language.trim().toLowerCase();
	return normalized || null;
}

function uniqueLanguages(chapters: ChapterSummary[]): string[] {
	return [...new Set(chapters.map((chapter) => normalizeLanguage(chapter.language_code)).filter(Boolean))].sort() as string[];
}

async function loadLiveWorkWithChapters(api: MugenHostApi['catalog']) {
	const page = await api.listWorks({ limit: 8 });

	for (const work of page.data) {
		const chapters = await api.listChaptersByWork(work.id).catch(() => []);
		const languages = uniqueLanguages(chapters);
		if (chapters.length >= 6 && languages.length >= 2) {
			return {
				work,
				chapters
			};
		}
	}

	return null;
}

export default definePlugin<MugenHostApi>({
	manifest: {
		id: 'org.mugens.tests.chapters-lab',
		name: 'Chapters Lab',
		version: '0.1.0',
		apiVersion: UNODE_CORE_API_VERSION,
		permissions: [
			{
				permission: 'catalog.read',
				reason: 'Load real chapters to exercise the unode menu and list primitives.'
			},
			{
				permission: 'navigation.write',
				reason: 'Expose the chapters lab through app navigation and commands.'
			}
		]
	},
	i18n: {
		en,
		'pt-br': ptBr
	},
	navigation: [
		{
			id: 'tests.chapters-lab',
			label: msg('nav_label'),
			shortLabel: msg('nav_short'),
			to: CHAPTERS_LAB_ROUTE,
			section: 'settings',
			priority: 11
		}
	],
	commands: [
		{
			id: 'tests.chapters-lab.open',
			title: msg('command_title'),
			category: 'Tests',
			run: ({ host }) => host.navigation.navigate(CHAPTERS_LAB_ROUTE)
		}
	],
	routes: [
		route<MugenHostApi>(CHAPTERS_LAB_ROUTE)
			.load<ChaptersLabData>(async ({ api, i18n }) => {
				const t = i18n.t;
				try {
					const live = await loadLiveWorkWithChapters(api.catalog);
					if (live) {
						return {
							work: live.work,
							chapters: live.chapters,
							error: null
						} satisfies ChaptersLabData;
					}
				} catch (error: unknown) {
					return {
						work: fallbackWork,
						chapters: fallbackChapters,
						error: t('error_message', { reason: toErrorMessage(error) })
					} satisfies ChaptersLabData;
				}

				return {
					work: fallbackWork,
					chapters: fallbackChapters,
					error: t('error_message', { reason: 'No live work with chapters was found.' })
				} satisfies ChaptersLabData;
			})
			.render((data, renderCtx) => {
				const t = renderCtx.i18n.t;
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

				return buildChaptersLabScreen({
					t,
					routePath: CHAPTERS_LAB_ROUTE,
					workTitle: data.work.title,
					languages: availableLanguages,
					selectedLanguage,
					chapters,
					error: data.error
				});
			})
	]
});
