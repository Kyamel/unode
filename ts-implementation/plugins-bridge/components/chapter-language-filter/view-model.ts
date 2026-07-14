import type { ActionRef } from '$lib/unode/core/ast';
import type { TranslateFn } from '$lib/unode/core/runtime';

export type ChapterLanguageOptionViewModel = {
	key: string;
	label: string;
	selected: boolean;
	action: ActionRef;
};

export type ChapterLanguageFilterViewModel = {
	menuLabel: string;
	options: readonly ChapterLanguageOptionViewModel[];
	countLabel: string;
	activeFilterLabel?: string;
};

export type CreateChapterLanguageFilterViewModelInput = {
	routePath: string;
	selectedLanguage: string;
	languages: readonly string[];
	visibleCount: number;
	t: TranslateFn;
};

export function chapterLanguageCodeLabel(code: string, t: TranslateFn): string {
	if (!code || code === 'all') return t('filter_language_all');
	return code.split('-')[0]?.toUpperCase() ?? code.toUpperCase();
}

function languageFilterAction(routePath: string, language: string): ActionRef {
	return {
		type: 'unode.navigate',
		params: {
			to: routePath,
			query: {
				language: language === 'all' ? null : language
			}
		}
	};
}

export function createChapterLanguageFilterViewModel(
	input: CreateChapterLanguageFilterViewModelInput
): ChapterLanguageFilterViewModel {
	const selectedLabel = chapterLanguageCodeLabel(input.selectedLanguage, input.t);

	return {
		menuLabel: input.t(
			'filter_language_button',
			{ label: selectedLabel },
			`Language: ${selectedLabel}`
		),
		options: [
			{
				key: 'chapter-language-filter:all',
				label: input.t('filter_language_all'),
				selected: input.selectedLanguage === 'all',
				action: languageFilterAction(input.routePath, 'all')
			},
			...input.languages.map((language) => ({
				key: `chapter-language-filter:${language}`,
				label: chapterLanguageCodeLabel(language, input.t),
				selected: input.selectedLanguage === language,
				action: languageFilterAction(input.routePath, language)
			}))
		],
		countLabel: input.t(
			'chapters_count_badge',
			{ count: input.visibleCount },
			`${input.visibleCount} chapters`
		),
		activeFilterLabel:
			input.selectedLanguage !== 'all'
				? input.t(
						'filter_active_badge',
						{ label: selectedLabel },
						`Filter: ${selectedLabel}`
					)
				: undefined
	};
}
