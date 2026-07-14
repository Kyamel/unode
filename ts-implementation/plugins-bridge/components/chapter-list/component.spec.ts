import { describe, expect, it } from 'vitest';
import { createChapterListViewModel } from './view-model';
import { chapterList } from './component';

describe('chapterList', () => {
	it('builds a reusable chapter list from a view model', () => {
		const t = (key: string, values?: Record<string, unknown>, fallback?: string) => {
			const messages: Record<string, string> = {
				chapter_title: 'Chapter {number}',
				chapter_title_fallback: 'Chapter',
				chapter_volume: 'Vol. {volume}',
				filter_language_all: 'All',
				chapter_language_unknown: 'Unknown language',
				chapters_empty_title: 'No chapters',
				chapters_empty_message: 'Try another filter.',
				load_more_chapters: 'Load more chapters'
			};

			return (messages[key] ?? fallback ?? key)
				.replace('{number}', String(values?.number ?? ''))
				.replace('{volume}', String(values?.volume ?? ''));
		};

		const viewModel = createChapterListViewModel({
			chapters: [
				{
					id: 'chapter-1',
					number: '1',
					title: 'The Blue of Longing',
					language_code: 'en',
					volume: 1
				}
			],
			t,
			continuationBinding: 'chapters.visibleCount'
		});

		const node = chapterList(viewModel);

		expect(node.kind).toBe('list');
		if (node.kind !== 'list') throw new Error('Expected list root');

		expect(node.items[0]).toMatchObject({
			kind: 'item'
		});
		expect(node.continuation).toMatchObject({
			kind: 'incremental',
			binding: 'chapters.visibleCount',
			initial: 3,
			step: 3,
			label: 'Load more chapters'
		});
	});
});
