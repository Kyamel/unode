import type { ImmutableScreen } from '$lib/unode/core/ast';
import { ui } from '$lib/unode/core/dsl';
import type { TranslateFn } from '$lib/unode/core/runtime';
import { buildMangaWorkRouteTabs } from '$lib/plugins-bridge/domains/manga/route-tabs';
import type { WorkRelation } from '$lib/plugins-bridge/models';
import { navigateAction } from '$lib/plugins-bridge/navigation';
import { withRouteTabs } from '$lib/plugins-bridge/screen-chrome/route-tabs';

type RelatedScreenInput = {
	t: TranslateFn;
	workId: string;
	relations: WorkRelation[];
	error: string | null;
};

function formatRelationLabel(value: string | null | undefined): string {
	if (!value) return '';

	return value
		.split(/[_-]+/g)
		.map((segment) => segment.charAt(0).toUpperCase() + segment.slice(1).toLowerCase())
		.join(' ');
}

function buildRelationsList(input: RelatedScreenInput) {
	return ui.list(
		input.relations.map((relation, index) =>
			ui.item(
				String(relation.id || index),
				[
					ui.text(relation.related_work_title ?? input.t('value_na'), {
						role: 'body'
					})
				],
				{
					secondary: relation.relation_type
						? [
								ui.text(formatRelationLabel(relation.relation_type), {
									role: 'caption'
								})
							]
						: undefined,
					trailing: relation.relation_type
						? [
								ui.badge(formatRelationLabel(relation.relation_type), 'info')
							]
						: undefined,
					action: relation.related_work_id
						? navigateAction(`/app/mangas/${relation.related_work_id}/meta`)
						: undefined
				}
			)
		),
	);
}

export function buildRelatedScreen(input: RelatedScreenInput): ImmutableScreen {
	const content =
		!input.error && input.relations.length
			? buildRelationsList(input)
			: ui.empty(input.t('catalog_related_empty_title'), {
					message: input.error ?? input.t('catalog_related_placeholder')
				});

	return withRouteTabs(
		ui.screen(
			{
				id: `work-related:${input.workId}:screen`,
				title: input.t('manga_tab_related')
			},
			[
				ui.stack(
					[
						ui.stack(
						[
							ui.text(input.t('manga_tab_related'), {
								role: 'heading'
							}),
							ui.text(input.t('related_screen_subtitle'), {
								role: 'subtitle'
							})
						]
					),
						...(input.error
							? [
									ui.status('warning', input.error, {
										title: input.t('failed_to_load')
									})
								]
							: []),
						content
					]
				)
			]
		),
		buildMangaWorkRouteTabs(input.workId, input.t, 'related')
	);
}
