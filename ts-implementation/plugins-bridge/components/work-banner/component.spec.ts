import { describe, expect, it } from 'vitest';
import { createWorkBannerViewModel } from './view-model';
import { workBanner } from './component';

describe('workBanner', () => {
	it('builds a banner from a work view model', () => {
		const viewModel = createWorkBannerViewModel(
			{
				id: 'work-1',
				title: 'Blue Period',
				cover: null,
				work_type: 'manga',
				status: 'completed',
				year: 2017
			},
			(key, values, fallback) => {
				const messages: Record<string, string> = {
					fallback_type: 'Work',
					fallback_year: 'Unknown year',
					fallback_cover_alt: 'Work cover',
					cover_alt: 'Cover of {title}',
					meta_separator: '•',
					'work_type.manga': 'Manga',
					'work_status.completed': 'Completed'
				};
				return (messages[key] ?? fallback ?? key).replace(
					'{title}',
					String(values?.title ?? '')
				);
			}
		);

		const node = workBanner(viewModel);

		expect(node.kind).toBe('stack');
		if (node.kind !== 'stack') {
			throw new Error('Expected workBanner to currently render as a stack root.');
		}
		expect(node.children[0]).toMatchObject({
			kind: 'media',
			mediaKind: 'cover',
			ref: {
				type: 'placeholder',
				kind: 'cover',
				label: 'Blue Period'
			},
			alt: 'Cover of Blue Period'
		});
		expect(node.children[1]).toMatchObject({
			kind: 'stack'
		});
	});
});
