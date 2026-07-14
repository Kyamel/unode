import type { TranslateFn } from '$lib/unode/core/runtime';
import type { MediaRef, TextRole, Tone } from '$lib/unode/core/ast';
import type { WorkSummary } from '$lib/plugins-bridge/models';
import { labelForWorkStatus, labelForWorkType } from '../workLabels';

export type WorkBannerSource = Pick<
	WorkSummary,
	'id' | 'title' | 'cover' | 'year' | 'work_type' | 'status'
>;

export type WorkBannerMetaPartViewModel = {
	key: string;
	label: string;
	role: TextRole;
};

export type WorkBannerBadgeViewModel = {
	key: string;
	label: string;
	tone?: Tone;
};

export type WorkBannerViewModel = {
	id: string;
	title: string;
	coverRef: MediaRef;
	coverAlt: string;
	meta: readonly WorkBannerMetaPartViewModel[];
	badges: readonly WorkBannerBadgeViewModel[];
};

function mediaRefForWork(work: WorkBannerSource): MediaRef {
	const src = work.cover?.url?.trim();
	if (src) {
		return { type: 'url', src };
	}

	return {
		type: 'placeholder',
		kind: 'cover',
		label: work.title
	};
}

export function createWorkBannerViewModel(
	work: WorkBannerSource,
	t: TranslateFn
): WorkBannerViewModel {
	const fallbackType = t('fallback_type', undefined, 'Work');
	const fallbackYear = t('fallback_year', undefined, 'Unknown year');
	const fallbackCoverAlt = t('fallback_cover_alt', undefined, 'Work cover');
	const workTypeLabel = labelForWorkType(work.work_type, t, fallbackType);
	const statusLabel = labelForWorkStatus(work.status, t);

	return {
		id: work.id,
		title: work.title,
		coverRef: mediaRefForWork(work),
		coverAlt: work.title
			? t('cover_alt', { title: work.title }, `Cover of ${work.title}`)
			: fallbackCoverAlt,
		meta: [
			{
				key: `${work.id}:year`,
				label: work.year ? String(work.year) : fallbackYear,
				role: 'caption'
			},
			{
				key: `${work.id}:separator`,
				label: t('meta_separator', undefined, '•'),
				role: 'hint'
			},
			{
				key: `${work.id}:type`,
				label: workTypeLabel,
				role: 'caption'
			}
		],
		badges: [
			{
				key: `${work.id}:badge:type`,
				label: workTypeLabel,
				tone: 'info'
			},
			...(statusLabel
				? [
						{
							key: `${work.id}:badge:status`,
							label: statusLabel,
							tone: 'default' as const
						}
					]
				: [])
		]
	};
}
