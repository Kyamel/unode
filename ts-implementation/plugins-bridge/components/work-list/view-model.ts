import type { ActionRef, CollectionContinuation, MediaRef, Tone } from '$lib/unode/core/ast';
import type { TranslateFn } from '$lib/unode/core/runtime';
import type { WorkSummary } from '$lib/plugins-bridge/models';
import { labelForWorkStatus, labelForWorkType } from '../workLabels';

export type WorkListBadgeViewModel = {
	key: string;
	label: string;
	tone?: Tone;
};

export type WorkListItemViewModel = {
	key: string;
	title: string;
	subtitle?: string;
	coverRef?: MediaRef;
	coverAlt?: string;
	badges: readonly WorkListBadgeViewModel[];
	action?: ActionRef;
};

export type WorkListViewModel = {
	items: readonly WorkListItemViewModel[];
	emptyTitle: string;
	emptyMessage: string;
	continuation?: CollectionContinuation;
};

export type CreateWorkListViewModelInput = {
	works: readonly WorkSummary[];
	t: TranslateFn;
	emptyTitle: string;
	emptyMessage: string;
	actionForWork?: (work: WorkSummary) => ActionRef | undefined;
	continuation?: CollectionContinuation;
};

function mediaRefForWork(work: WorkSummary): MediaRef | undefined {
	const src = work.cover?.url?.trim();
	if (!src) return undefined;
	return { type: 'url', src };
}

function workSubtitle(work: WorkSummary, t: TranslateFn): string | undefined {
	const parts: string[] = [];
	const typeLabel = labelForWorkType(work.work_type, t, '');
	if (typeLabel) parts.push(typeLabel);
	if (work.year) parts.push(String(work.year));
	return parts.join(' • ') || undefined;
}

export function createWorkListViewModel(
	input: CreateWorkListViewModelInput
): WorkListViewModel {
	return {
		items: input.works.map((work) => {
			const statusLabel = labelForWorkStatus(work.status, input.t);
			const typeLabel = labelForWorkType(work.work_type, input.t, '');

			return {
				key: work.id,
				title: work.title,
				subtitle: workSubtitle(work, input.t),
				coverRef: mediaRefForWork(work),
				coverAlt: work.title
					? input.t('cover_alt', { title: work.title }, `Cover of ${work.title}`)
					: input.t('fallback_cover_alt', undefined, 'Work cover'),
				badges: [
					...(typeLabel
						? [{ key: `${work.id}:type`, label: typeLabel, tone: 'info' as const }]
						: []),
					...(statusLabel
						? [{ key: `${work.id}:status`, label: statusLabel, tone: 'warning' as const }]
						: [])
				],
				action: input.actionForWork?.(work)
			};
		}),
		emptyTitle: input.emptyTitle,
		emptyMessage: input.emptyMessage,
		continuation: input.continuation
	};
}
