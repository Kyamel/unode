import { describe, expect, it } from 'vitest';
import { createWorkListViewModel } from './view-model';
import { workList } from './component';

describe('workList', () => {
	it('builds a reusable work list from a view model', () => {
		const t = (key: string, values?: Record<string, unknown>, fallback?: string) => {
			const messages: Record<string, string> = {
				cover_alt: 'Cover of {title}',
				fallback_cover_alt: 'Work cover',
				fallback_type: 'Work',
				'work_type.manga': 'Manga',
				'work_status.ongoing': 'Ongoing'
			};

			return (messages[key] ?? fallback ?? key).replace('{title}', String(values?.title ?? ''));
		};

		const viewModel = createWorkListViewModel({
			works: [
				{
					id: 'work-1',
					title: 'Blue Box',
					work_type: 'manga',
					status: 'ongoing',
					year: 2021,
					cover: { url: 'https://example.com/cover.jpg' }
				}
			],
			t,
			emptyTitle: 'No works',
			emptyMessage: 'Try another filter.'
		});

		const node = workList(viewModel);

		expect(node.kind).toBe('list');
		if (node.kind !== 'list') throw new Error('Expected list root');

		expect(node.items[0]).toMatchObject({
			kind: 'item'
		});
	});
});
