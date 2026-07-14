import type { TranslateFn } from '$lib/unode/core/runtime';
import type { MediaRef, Tone } from '$lib/unode/core/ast';
import type { WorkDetails, WorkTagPreview } from '$lib/plugins-bridge/models';
import { humanizeWorkToken, labelForWorkStatus, labelForWorkType } from '../workLabels';

export type WorkMetadataFieldViewModel = {
	key: string;
	label: string;
	value: string;
};

export type WorkMetadataBadgeViewModel = {
	key: string;
	label: string;
	tone?: Tone;
};

export type WorkMetadataTaxonomyGroupViewModel = {
	key: string;
	label: string;
	badges: readonly WorkMetadataBadgeViewModel[];
	emptyLabel: string;
};

export type WorkMetadataDisclosureViewModel = {
	showMoreLabel: string;
	showLessLabel: string;
	taxonomyGroups: readonly WorkMetadataTaxonomyGroupViewModel[];
	technicalFields: readonly WorkMetadataFieldViewModel[];
};

export type WorkMetadataViewModel = {
	id: string;
	title: string;
	subtitle: string;
	coverRef: MediaRef;
	coverAlt: string;
	coverExpandable?: boolean;
	summaryFields: readonly WorkMetadataFieldViewModel[];
	details: WorkMetadataDisclosureViewModel;
};

export type WorkMetadataDisclosureBinding = string;

export type CreateWorkMetadataViewModelOptions = {
	previewCover?: boolean;
};

type WorkMetadataMessages = {
	fallback: {
		type: string;
		year: string;
		unknown: string;
		none: string;
		coverAlt: string;
	};
	labels: {
		type: string;
		status: string;
		year: string;
		format: string;
		language: string;
		demographic: string;
		contentRating: string;
		source: string;
		genres: string;
		tags: string;
		themes: string;
		contentTags: string;
		workId: string;
		visibility: string;
		createdAt: string;
		updatedAt: string;
	};
	subtitle: string;
	showMoreDetails: string;
	showLessDetails: string;
};

function mediaRefForWork(work: WorkDetails): MediaRef {
	return work.cover?.url?.trim()
		? { type: 'url', src: work.cover.url }
		: { type: 'placeholder', kind: 'cover', label: work.title };
}

function createMessages(t: TranslateFn): WorkMetadataMessages {
	return {
		subtitle: t('metadata_layout_subtitle'),
		showMoreDetails: t('show_more_details'),
		showLessDetails: t('show_less_details'),
		fallback: {
			type: t('fallback_type'),
			year: t('fallback_year'),
			unknown: t('field_value_unknown'),
			none: t('field_value_none'),
			coverAlt: t('fallback_cover_alt')
		},
		labels: {
			type: t('field_type'),
			status: t('field_status'),
			year: t('field_year'),
			format: t('field_format'),
			language: t('field_language'),
			demographic: t('field_demographic'),
			contentRating: t('field_content_rating'),
			source: t('field_source_kind'),
			genres: t('field_genres'),
			tags: t('field_tags'),
			themes: t('field_themes'),
			contentTags: t('field_content_tags'),
			workId: t('field_work_id'),
			visibility: t('field_visibility'),
			createdAt: t('field_created_at'),
			updatedAt: t('field_updated_at')
		}
	};
}

function labelForEnumValue(
	t: TranslateFn,
	prefix: string,
	value: string | null | undefined,
	fallback: string
): string {
	if (!value) return fallback;
	return t(`${prefix}.${value}`, undefined, humanizeWorkToken(value));
}

function labelForFormat(
	work: WorkDetails,
	t: TranslateFn,
	messages: WorkMetadataMessages
): string {
	return labelForEnumValue(t, 'work_format', work.format, messages.fallback.unknown);
}

function labelForLanguage(
	work: WorkDetails,
	t: TranslateFn,
	messages: WorkMetadataMessages
): string {
	return labelForEnumValue(t, 'language', work.original_language, messages.fallback.unknown);
}

function labelForDemographic(
	work: WorkDetails,
	t: TranslateFn,
	messages: WorkMetadataMessages
): string {
	return labelForEnumValue(
		t,
		'work_demographic',
		work.publication_demographic,
		messages.fallback.unknown
	);
}

function labelForContentRating(
	work: WorkDetails,
	t: TranslateFn,
	messages: WorkMetadataMessages
): string {
	return labelForEnumValue(
		t,
		'work_content_rating',
		work.content_rating,
		messages.fallback.unknown
	);
}

function labelForSourceKind(
	work: WorkDetails,
	t: TranslateFn,
	messages: WorkMetadataMessages
): string {
	return labelForEnumValue(t, 'work_source_kind', work.source_kind, messages.fallback.unknown);
}

function labelForVisibility(
	work: WorkDetails,
	t: TranslateFn,
	messages: WorkMetadataMessages
): string {
	return labelForEnumValue(t, 'work_visibility', work.visibility, messages.fallback.unknown);
}

function formatTechnicalTimestamp(value: string | null | undefined, fallback: string): string {
	if (!value) return fallback;

	const date = new Date(value);
	if (Number.isNaN(date.getTime())) return value;

	return date.toISOString().slice(0, 10);
}

function badgesFromTags(
	groupKey: string,
	tags: WorkTagPreview[] | null | undefined,
	tone: Tone = 'info'
): readonly WorkMetadataBadgeViewModel[] {
	const badges: WorkMetadataBadgeViewModel[] = [];

	for (const tag of tags ?? []) {
		const label = tag.name?.trim();
		if (!label) continue;

		badges.push({
			key: `${groupKey}:badge:${tag.id || label}`,
			label,
			tone
		});
	}

	return badges;
}

export function createWorkMetadataViewModel(
	work: WorkDetails,
	t: TranslateFn,
	options?: CreateWorkMetadataViewModelOptions
): WorkMetadataViewModel {
	const messages = createMessages(t);

	return {
		id: work.id,
		title: work.title,
		subtitle: messages.subtitle,
		coverRef: mediaRefForWork(work),
		coverAlt: work.title
			? t('cover_alt', { title: work.title }, `Cover of ${work.title}`)
			: messages.fallback.coverAlt,
		coverExpandable: options?.previewCover === false ? undefined : true,
		summaryFields: [
			{
				key: `${work.id}:summary:type`,
				label: messages.labels.type,
				value: labelForWorkType(work.work_type, t, messages.fallback.type)
			},
			{
				key: `${work.id}:summary:status`,
				label: messages.labels.status,
				value: labelForWorkStatus(work.status, t) ?? messages.fallback.unknown
			},
			{
				key: `${work.id}:summary:year`,
				label: messages.labels.year,
				value: work.year ? String(work.year) : messages.fallback.year
			},
			{
				key: `${work.id}:summary:format`,
				label: messages.labels.format,
				value: labelForFormat(work, t, messages)
			},
			{
				key: `${work.id}:summary:language`,
				label: messages.labels.language,
				value: labelForLanguage(work, t, messages)
			},
			{
				key: `${work.id}:summary:demographic`,
				label: messages.labels.demographic,
				value: labelForDemographic(work, t, messages)
			},
			{
				key: `${work.id}:summary:content-rating`,
				label: messages.labels.contentRating,
				value: labelForContentRating(work, t, messages)
			},
			{
				key: `${work.id}:summary:source-kind`,
				label: messages.labels.source,
				value: labelForSourceKind(work, t, messages)
			}
		],
		details: {
			showMoreLabel: messages.showMoreDetails,
			showLessLabel: messages.showLessDetails,
			taxonomyGroups: [
				{
					key: `${work.id}:taxonomy:genres`,
					label: messages.labels.genres,
					badges: badgesFromTags(`${work.id}:taxonomy:genres`, work.genres),
					emptyLabel: messages.fallback.none
				},
				{
					key: `${work.id}:taxonomy:tags`,
					label: messages.labels.tags,
					badges: badgesFromTags(`${work.id}:taxonomy:tags`, work.tags),
					emptyLabel: messages.fallback.none
				},
				{
					key: `${work.id}:taxonomy:themes`,
					label: messages.labels.themes,
					badges: badgesFromTags(`${work.id}:taxonomy:themes`, work.themes),
					emptyLabel: messages.fallback.none
				},
				{
					key: `${work.id}:taxonomy:content`,
					label: messages.labels.contentTags,
					badges: badgesFromTags(`${work.id}:taxonomy:content`, work.content),
					emptyLabel: messages.fallback.none
				}
			],
			technicalFields: [
				{
					key: `${work.id}:technical:work-id`,
					label: messages.labels.workId,
					value: work.id
				},
				{
					key: `${work.id}:technical:visibility`,
					label: messages.labels.visibility,
					value: labelForVisibility(work, t, messages)
				},
				{
					key: `${work.id}:technical:created-at`,
					label: messages.labels.createdAt,
					value: formatTechnicalTimestamp(work.created_at, messages.fallback.unknown)
				},
				{
					key: `${work.id}:technical:updated-at`,
					label: messages.labels.updatedAt,
					value: formatTechnicalTimestamp(work.updated_at, messages.fallback.unknown)
				}
			]
		}
	};
}
