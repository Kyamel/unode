import type { ActionRef, CollectionContinuation } from '$lib/unode/core/ast';
import type { TranslateFn } from '$lib/unode/core/runtime';
import type { ChapterSummary } from '$lib/plugins-bridge/models';
import { chapterLanguageCodeLabel } from '../chapter-language-filter/view-model';

export type ChapterListItemViewModel = {
	key: string;
	title: string;
	subtitle?: string;
	numberLabel?: string;
	languageLabel: string;
	action?: ActionRef;
};

export type ChapterListContinuationViewModel = {
	binding: string;
	initial: number;
	step: number;
	label: string;
};

export type ChapterListViewModel = {
	items: readonly ChapterListItemViewModel[];
	emptyTitle: string;
	emptyMessage: string;
	continuation?: CollectionContinuation;
};

export type CreateChapterListViewModelInput = {
	chapters: readonly ChapterSummary[];
	t: TranslateFn;
	continuationBinding: string;
	initial?: number;
	step?: number;
	actionForChapter?: (chapter: ChapterSummary) => ActionRef | undefined;
	continuation?: CollectionContinuation;
};

function chapterTitle(chapter: ChapterSummary, t: TranslateFn): string {
	if (chapter.number === null || chapter.number === undefined || chapter.number === '') {
		return t('chapter_title_fallback');
	}

	return t('chapter_title', { number: chapter.number }, `Chapter ${chapter.number}`);
}

function chapterSubtitle(chapter: ChapterSummary, t: TranslateFn): string {
	const parts: string[] = [];
	if (chapter.title) parts.push(chapter.title);
	if (chapter.volume !== null && chapter.volume !== undefined) {
		parts.push(t('chapter_volume', { volume: chapter.volume }, `Vol. ${chapter.volume}`));
	}
	return parts.join(' • ');
}

export function createChapterListViewModel(
	input: CreateChapterListViewModelInput
): ChapterListViewModel {
	return {
		items: input.chapters.map((chapter) => ({
			key: chapter.id,
			title: chapterTitle(chapter, input.t),
			subtitle: chapterSubtitle(chapter, input.t) || undefined,
			numberLabel:
				chapter.number === null || chapter.number === undefined || chapter.number === ''
					? undefined
					: String(chapter.number),
			languageLabel:
				chapterLanguageCodeLabel(chapter.language_code ?? '', input.t) ||
				input.t('chapter_language_unknown'),
			action: input.actionForChapter?.(chapter)
		})),
		emptyTitle: input.t('chapters_empty_title'),
		emptyMessage: input.t('chapters_empty_message'),
		continuation:
			input.continuation ??
			({
				kind: 'incremental',
				binding: input.continuationBinding,
				initial: input.initial ?? 3,
				step: input.step ?? 3,
				label: input.t('load_more_chapters')
			} satisfies CollectionContinuation)
	};
}
