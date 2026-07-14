import { describe, expect, it } from 'vitest';
import { createWorkMetadataViewModel } from './view-model';
import { workMetadataDetailsDisclosure, workMetadataLayout } from './component';
import type { WorkDetails } from '$lib/plugins-bridge/models';

function testTranslate(key: string, values?: Record<string, unknown>, fallback?: string) {
	const messages: Record<string, string> = {
		metadata_layout_subtitle: 'Metadata overview',
		show_more_details: 'Show more details',
		show_less_details: 'Show less details',
		fallback_type: 'Work',
		fallback_year: 'Unknown year',
		fallback_cover_alt: 'Work cover',
		cover_alt: 'Cover of {title}',
		field_value_unknown: 'Unknown',
		field_value_none: 'None',
		field_type: 'Type',
		field_status: 'Status',
		field_year: 'Year',
		field_format: 'Format',
		field_language: 'Language',
		field_demographic: 'Demographic',
		field_content_rating: 'Content Rating',
		field_source_kind: 'Source',
		field_genres: 'Genres',
		field_tags: 'Tags',
		field_themes: 'Themes',
		field_content_tags: 'Content Tags',
		field_work_id: 'Work ID',
		field_visibility: 'Visibility',
		field_created_at: 'Created',
		field_updated_at: 'Updated',
		'work_format.series': 'Series',
		'language.ja': 'Japanese',
		'work_demographic.shounen': 'Shounen',
		'work_content_rating.safe': 'Safe',
		'work_source_kind.external': 'External',
		'work_visibility.public': 'Public',
		'work_type.manga': 'Manga',
		'work_status.completed': 'Completed'
	};

	return (messages[key] ?? fallback ?? key).replace('{title}', String(values?.title ?? ''));
}

const sampleWork: WorkDetails = {
	id: 'work-1',
	title: 'Blue Period',
	cover: null,
	work_type: 'manga',
	status: 'completed',
	year: 2017,
	format: 'series',
	original_language: 'ja',
	publication_demographic: 'shounen',
	content_rating: 'safe',
	source_kind: 'external',
	visibility: 'public',
	genres: [{ id: 'genre-1', name: 'Drama' }],
	tags: [{ id: 'tag-1', name: 'School' }],
	themes: [],
	content: null,
	created_at: '2024-01-02T03:04:05.000Z',
	updated_at: '2024-02-03T04:05:06.000Z'
};

describe('workMetadata shared components', () => {
	it('builds a responsive metadata layout from a view model', () => {
		const viewModel = createWorkMetadataViewModel(sampleWork, testTranslate);
		const node = workMetadataLayout(viewModel, {
			disclosureBinding: 'details.open'
		});

		expect(node.kind).toBe('grid');
		if (node.kind !== 'grid') {
			throw new Error('Expected workMetadataLayout to render as a grid root.');
		}

		expect(node.columns).toEqual({
			base: 1,
			md: 2
		});
		expect(node.children[0]).toMatchObject({
			kind: 'media',
			expandable: true,
			ref: {
				type: 'placeholder',
				kind: 'cover',
				label: 'Blue Period'
			}
		});
		expect(node.children[1]).toMatchObject({
			kind: 'stack'
		});
	});

	it('renders taxonomy groups as badge lists inside the disclosure', () => {
		const viewModel = createWorkMetadataViewModel(sampleWork, testTranslate);
		const node = workMetadataDetailsDisclosure(viewModel, {
			binding: 'details.open'
		});

		expect(node.kind).toBe('disclosure');
		if (node.kind !== 'disclosure') {
			throw new Error('Expected workMetadataDetailsDisclosure to render as a disclosure root.');
		}

		expect(node.children[0]).toMatchObject({
			kind: 'stack'
		});

		const content = node.children[0];
		if (content.kind !== 'stack') {
			throw new Error('Expected disclosure content to render as a stack.');
		}

		const taxonomyGrid = content.children[1];
		expect(taxonomyGrid).toMatchObject({
			kind: 'grid'
		});
		if (taxonomyGrid.kind !== 'grid') {
			throw new Error('Expected taxonomy details to render as a grid.');
		}

		const genreField = taxonomyGrid.children[0];
		expect(genreField).toMatchObject({
			kind: 'stack'
		});
		if (genreField.kind !== 'stack') {
			throw new Error('Expected taxonomy field to render as a stack.');
		}

		expect(genreField.children[1]).toMatchObject({
			kind: 'inline'
		});
		if (genreField.children[1].kind !== 'inline') {
			throw new Error('Expected taxonomy badges to render as an inline row.');
		}

		expect(genreField.children[1].children[0]).toMatchObject({
			kind: 'badge',
			label: 'Drama'
		});
	});
});
