import { definePlugin, msg, route, UNODE_CORE_API_VERSION } from '$lib/unode/core/runtime';
import type { MugenHostApi } from '$lib/plugins-bridge/host';
import type { WorkDetails, WorkSummary } from '$lib/plugins-bridge/models';
import en from './messages/en.json';
import ptBr from './messages/pt-br.json';
import { buildBannerLabScreen } from './ui';

const BANNER_LAB_ROUTE = '/tests/banner';
const PAGE_SIZE = 6;
const BANNER_LAB_STORAGE_KEY = 'gallery-state';
const LOAD_MORE_ACTION_ID = 'tests.banner-lab.loadMore';

type BannerLabData = {
	works: WorkSummary[];
	featuredWork: WorkDetails | null;
	error: string | null;
	nextCursor: string | null;
};

type BannerLabStoredState = {
	works: WorkSummary[];
	nextCursor: string | null;
	source: 'live' | 'fallback';
};

const fallbackCover = {
	url: '/tests/banner-cover.svg'
} as const;

const fallbackWorks: WorkSummary[] = [
	{
		id: 'fallback-frieren',
		title: 'Frieren: Beyond Journey\'s End',
		cover: fallbackCover,
		work_type: 'manga',
		status: 'ongoing',
		year: 2020
	},
	{
		id: 'fallback-blue-box',
		title: 'Blue Box',
		cover: fallbackCover,
		work_type: 'manga',
		status: 'ongoing',
		year: 2021
	},
	{
		id: 'fallback-vinland',
		title: 'Vinland Saga',
		cover: fallbackCover,
		work_type: 'manga',
		status: 'ongoing',
		year: 2005
	},
	{
		id: 'fallback-dandadan',
		title: 'Dandadan',
		cover: fallbackCover,
		work_type: 'manga',
		status: 'ongoing',
		year: 2021
	},
	{
		id: 'fallback-sakamoto',
		title: 'Sakamoto Days',
		cover: fallbackCover,
		work_type: 'manga',
		status: 'ongoing',
		year: 2020
	},
	{
		id: 'fallback-witch-hat',
		title: 'Witch Hat Atelier',
		cover: fallbackCover,
		work_type: 'manga',
		status: 'ongoing',
		year: 2016
	},
	{
		id: 'fallback-kaiju',
		title: 'Kaiju No. 8',
		cover: fallbackCover,
		work_type: 'manga',
		status: 'ongoing',
		year: 2020
	},
	{
		id: 'fallback-yotsuba',
		title: 'Yotsuba&!',
		cover: fallbackCover,
		work_type: 'manga',
		status: 'ongoing',
		year: 2003
	},
	{
		id: 'fallback-golden-kamuy',
		title: 'Golden Kamuy',
		cover: fallbackCover,
		work_type: 'manga',
		status: 'completed',
		year: 2014
	},
	{
		id: 'fallback-ao-ashi',
		title: 'Ao Ashi',
		cover: fallbackCover,
		work_type: 'manga',
		status: 'ongoing',
		year: 2015
	},
	{
		id: 'fallback-chihayafuru',
		title: 'Chihayafuru',
		cover: fallbackCover,
		work_type: 'manga',
		status: 'completed',
		year: 2007
	},
	{
		id: 'fallback-space-brothers',
		title: 'Space Brothers',
		cover: fallbackCover,
		work_type: 'manga',
		status: 'ongoing',
		year: 2007
	}
];

const fallbackFeaturedWork: WorkDetails = {
	id: 'fallback-frieren',
	title: 'Frieren: Beyond Journey\'s End',
	cover: fallbackCover,
	work_type: 'manga',
	status: 'ongoing',
	year: 2020,
	format: 'serial',
	original_language: 'ja',
	publication_demographic: 'shounen',
	content_rating: 'teen',
	source_kind: 'original',
	visibility: 'public',
	genres: [
		{ id: 'genre-fantasy', name: 'Fantasy' },
		{ id: 'genre-adventure', name: 'Adventure' }
	],
	tags: [
		{ id: 'tag-journey', name: 'Journey' },
		{ id: 'tag-elf', name: 'Elf Protagonist' },
		{ id: 'tag-post-quest', name: 'After the Hero Party' }
	],
	themes: [
		{ id: 'theme-melancholy', name: 'Melancholy' },
		{ id: 'theme-memory', name: 'Memory' }
	],
	content: [{ id: 'content-safe', name: 'Safe Reading' }],
	created_at: '2020-04-28T00:00:00.000Z',
	updated_at: '2026-03-01T00:00:00.000Z'
};

function toErrorMessage(error: unknown): string {
	if (error instanceof Error && error.message.trim()) return error.message;
	if (typeof error === 'string' && error.trim()) return error;
	return 'Unknown error.';
}

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

function paginateFallback(cursor?: string | null): { works: WorkSummary[]; nextCursor: string | null } {
	const offset = cursor ? Number(cursor) : 0;
	const start = Number.isFinite(offset) && offset >= 0 ? offset : 0;
	const works = fallbackWorks.slice(start, start + PAGE_SIZE);
	const nextCursor = start + PAGE_SIZE < fallbackWorks.length ? String(start + PAGE_SIZE) : null;
	return { works, nextCursor };
}

async function loadStoredState(
	api: MugenHostApi,
	pluginId: string
): Promise<BannerLabStoredState | null> {
	return await api.storage.getScoped<BannerLabStoredState>(pluginId, BANNER_LAB_STORAGE_KEY);
}

async function saveStoredState(
	api: MugenHostApi,
	pluginId: string,
	state: BannerLabStoredState
): Promise<void> {
	await api.storage.setScoped(pluginId, BANNER_LAB_STORAGE_KEY, state);
}

export default definePlugin<MugenHostApi>({
	manifest: {
		id: 'org.mugens.tests.banner-lab',
		name: 'Banner Lab',
		version: '0.1.0',
		apiVersion: UNODE_CORE_API_VERSION,
		permissions: [
			{
				permission: 'catalog.read',
				reason: 'Load real works to preview the banner sugar against live catalog data.'
			},
			{
				permission: 'navigation.write',
				reason: 'Expose the banner lab through app navigation and commands.'
			},
			{
				permission: 'storage.persistent.read',
				reason: 'Reuse the current gallery state between remote continuation refreshes.'
			},
			{
				permission: 'storage.persistent.write',
				reason: 'Persist gallery state between remote continuation refreshes.'
			},
			{
				permission: 'events.write',
				reason: 'Refresh the current screen after loading more banners remotely.'
			}
		]
	},
	i18n: {
		en,
		'pt-br': ptBr
	},
	navigation: [
		{
			id: 'tests.banner-lab',
			label: msg('nav_label'),
			shortLabel: msg('nav_short'),
			to: BANNER_LAB_ROUTE,
			section: 'settings',
			priority: 10
		}
	],
	commands: [
		{
			id: 'tests.banner-lab.open',
			title: msg('command_title'),
			category: 'Tests',
			run: ({ host }) => host.navigation.navigate(BANNER_LAB_ROUTE)
		}
	],
	actions: [
		{
			id: LOAD_MORE_ACTION_ID,
			title: msg('load_more_banners'),
			async run({ action, host, pluginId, i18n }) {
				const t = i18n.t;
				const cursor = typeof action.params?.cursor === 'string' ? action.params.cursor : null;
				if (!cursor) return;

				const stored = await loadStoredState(host, pluginId);
				const currentWorks = stored?.works ?? [];

				try {
					if (stored?.source === 'fallback') {
						const page = paginateFallback(cursor);
						await saveStoredState(host, pluginId, {
							works: mergeWorks(currentWorks, page.works),
							nextCursor: page.nextCursor,
							source: 'fallback'
						});
					} else {
						const page = await host.catalog.listWorks({ limit: PAGE_SIZE, cursor });
						await saveStoredState(host, pluginId, {
							works: mergeWorks(currentWorks, page.data ?? []),
							nextCursor: page.lastCursor ?? null,
							source: 'live'
						});
					}

					host.events.emit({
						type: 'screen.refresh',
						pathname: BANNER_LAB_ROUTE
					});
				} catch (error: unknown) {
					host.feedback.toast({
						title: t('error_title'),
						message: t('error_message', undefined, toErrorMessage(error)),
						tone: 'danger'
					});
				}
			}
		}
	],
	routes: [
		route<MugenHostApi>(BANNER_LAB_ROUTE)
			.load<BannerLabData>(async ({ api, i18n, pluginId }) => {
				const t = i18n.t;
				try {
					const stored = await loadStoredState(api, pluginId);
					let works = stored?.works ?? [];
					let nextCursor = stored?.nextCursor ?? null;

					if (!works.length) {
						const page = await api.catalog.listWorks({ limit: PAGE_SIZE });
						works = page.data ?? [];
						nextCursor = page.lastCursor ?? null;

						if (works.length) {
							await saveStoredState(api, pluginId, {
								works,
								nextCursor,
								source: 'live'
							});
						}
					}

					if (!works.length) {
						const fallbackPage = paginateFallback();
						works = fallbackPage.works;
						nextCursor = fallbackPage.nextCursor;
						await saveStoredState(api, pluginId, {
							works,
							nextCursor,
							source: 'fallback'
						});
					}

					const featuredWork =
						works[0]?.id ? (await api.catalog.getWorkById(works[0].id).catch(() => null)) : null;
					return {
						works,
						featuredWork: featuredWork ?? fallbackFeaturedWork,
						error: null,
						nextCursor
					} satisfies BannerLabData;
				} catch (error: unknown) {
					const fallbackPage = paginateFallback();
					await saveStoredState(api, pluginId, {
						works: fallbackPage.works,
						nextCursor: fallbackPage.nextCursor,
						source: 'fallback'
					});
					return {
						works: fallbackPage.works,
						featuredWork: fallbackFeaturedWork,
						error: t('error_message', undefined, toErrorMessage(error)),
						nextCursor: fallbackPage.nextCursor
					} satisfies BannerLabData;
				}
			})
			.render((data, renderCtx) => {
				const t = renderCtx.i18n.t;
				return buildBannerLabScreen({
					t,
					works: data.works,
					featuredWork: data.featuredWork,
					error: data.error,
					galleryContinuation:
						data.nextCursor
							? {
									kind: 'remote',
									hasMore: true,
									loadMore: {
										type: LOAD_MORE_ACTION_ID,
									params: {
										cursor: data.nextCursor
									}
								},
									label: t('load_more_banners'),
									loadingLabel: t('action_loading', undefined, 'Loading...')
								}
							: undefined
				});
			})
	]
});
