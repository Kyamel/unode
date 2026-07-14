import { ui } from '$lib/unode/core/dsl';
import type { ImmutableScreen } from '$lib/unode/core/ast';
import type { TranslateFn } from '$lib/unode/core/runtime';
import {
	chapterLanguageFilterToolbar,
	chapterList,
	createChapterLanguageFilterViewModel,
	createChapterListViewModel
} from '$lib/plugins-bridge/components';
import type { ChapterSummary } from '$lib/plugins-bridge/models';

type ChaptersLabScreenInput = {
	t: TranslateFn;
	routePath: string;
	workTitle: string;
	languages: string[];
	selectedLanguage: string;
	chapters: ChapterSummary[];
	error?: string | null;
};

export function buildChaptersLabScreen(input: ChaptersLabScreenInput): ImmutableScreen {
	const { t, routePath, workTitle, languages, selectedLanguage, chapters, error } = input;
	const filterViewModel = createChapterLanguageFilterViewModel({
		routePath,
		selectedLanguage,
		languages,
		visibleCount: chapters.length,
		t
	});
	const chapterListViewModel = createChapterListViewModel({
		chapters,
		t,
		continuationBinding: 'chaptersLab.visibleCount'
	});

	return ui.screen(
		{
			id: 'chapters-lab:screen',
			title: t('screen_title'),
			initialState: {
				'chaptersLab.selectedLanguage': selectedLanguage
			}
		},
		[
			ui.stack(
				[
					ui.stack(
						{ gap: 'xs' },
						[
							ui.text(t('screen_title'), { role: 'heading' }),
							ui.text(t('screen_subtitle'), {
								role: 'subtitle'
							})
						]
					),
					...(error
						? [
								ui.status('warning', error, {
									title: t('error_title')
								})
							]
						: []),
					ui.section(
						{
							title: t('filter_section_title'),
							description: t('filter_section_description')
						},
						[
							chapterLanguageFilterToolbar(filterViewModel)
						]
					),
					ui.section(
						{
							title: t('chapters_section_title', { workTitle }, `Chapters for ${workTitle}`),
							description: t('chapters_section_description')
						},
						[
							chapterList(chapterListViewModel)
						]
					)
				]
			)
		]
	);
}
