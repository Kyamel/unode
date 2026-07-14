import type { ImmutableScreen } from '$lib/unode/core/ast';
import { ui } from '$lib/unode/core/dsl';
import type { TranslateFn } from '$lib/unode/core/runtime';
import {
	chapterLanguageFilterToolbar,
	chapterList,
	createChapterLanguageFilterViewModel,
	createChapterListViewModel
} from '$lib/plugins-bridge/components';
import { buildMangaWorkRouteTabs } from '$lib/plugins-bridge/domains/manga/route-tabs';
import type { ChapterSummary } from '$lib/plugins-bridge/models';
import { navigateAction } from '$lib/plugins-bridge/navigation';
import { withRouteTabs } from '$lib/plugins-bridge/screen-chrome/route-tabs';

type ChaptersScreenInput = {
	t: TranslateFn;
	workId: string;
	routePath: string;
	languages: string[];
	selectedLanguage: string;
	chapters: ChapterSummary[];
	error: string | null;
};

export function buildChaptersScreen(input: ChaptersScreenInput): ImmutableScreen {
	const filterViewModel = createChapterLanguageFilterViewModel({
		routePath: input.routePath,
		selectedLanguage: input.selectedLanguage,
		languages: input.languages,
		visibleCount: input.chapters.length,
		t: input.t
	});
	const listViewModel = createChapterListViewModel({
		chapters: input.chapters,
		t: input.t,
		continuationBinding: `workChapters.${input.workId}.visibleCount`,
		actionForChapter: (chapter) =>
			navigateAction(`/mangas/${input.workId}/read`, {
				query: {
					chapter: chapter.id
				}
			})
	});

	return withRouteTabs(
		ui.screen(
			{
				id: `work-chapters:${input.workId}:screen`,
				title: input.t('manga_tab_chapters')
			},
			[
				ui.stack(
					[
						ui.stack(
						[
							ui.text(input.t('manga_tab_chapters'), {
								role: 'heading'
							}),
							ui.text(input.t('chapters_screen_subtitle'), {
								role: 'subtitle'
							})
						]
					),
						...(input.error
							? [
									ui.status('warning', input.error, {
										title: input.t('failed_to_load')
									})
								]
							: []),
						ui.section(
							{
								title: input.t('filter_section_title'),
								description: input.t('filter_section_description')
							},
							[
								chapterLanguageFilterToolbar(filterViewModel)
							]
						),
						chapterList(listViewModel)
					]
				)
			]
		),
		buildMangaWorkRouteTabs(input.workId, input.t, 'chapters')
	);
}
