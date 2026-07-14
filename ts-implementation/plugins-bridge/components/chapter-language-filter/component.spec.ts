import { describe, expect, it } from 'vitest';
import { createChapterLanguageFilterViewModel } from './view-model';
import { chapterLanguageFilterToolbar } from './component';

describe('chapterLanguageFilterToolbar', () => {
	it('builds a reusable language filter toolbar from a view model', () => {
		const t = (key: string, values?: Record<string, unknown>, fallback?: string) => {
			const messages: Record<string, string> = {
				filter_language_button: 'Language: {label}',
				filter_language_all: 'All',
				chapters_count_badge: '{count} chapters',
				filter_active_badge: 'Filter: {label}'
			};

			return (messages[key] ?? fallback ?? key)
				.replace('{label}', String(values?.label ?? ''))
				.replace('{count}', String(values?.count ?? ''));
		};

		const viewModel = createChapterLanguageFilterViewModel({
			routePath: '/app/tests/chapters',
			selectedLanguage: 'ja',
			languages: ['en', 'ja', 'pt-br'],
			visibleCount: 3,
			t
		});

		const node = chapterLanguageFilterToolbar(viewModel);

		expect(node.kind).toBe('inline');
		if (node.kind !== 'inline') throw new Error('Expected inline root');

		expect(node.children[0]).toMatchObject({
			kind: 'menu',
			label: 'Language: JA'
		});
		expect(node.children[1]).toMatchObject({
			kind: 'badge',
			label: '3 chapters'
		});
		expect(node.children[2]).toMatchObject({
			kind: 'badge',
			label: 'Filter: JA'
		});
	});
});
