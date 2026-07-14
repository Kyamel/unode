import type { ImmutableScreen } from '$lib/unode/core/ast';
import { ui } from '$lib/unode/core/dsl';
import type { TranslateFn } from '$lib/unode/core/runtime';
import {
	createWorkMetadataViewModel,
	workMetadataLayout
} from '$lib/plugins-bridge/components';
import { buildMangaWorkRouteTabs } from '$lib/plugins-bridge/domains/manga/route-tabs';
import type { WorkDetails } from '$lib/plugins-bridge/models';
import { withRouteTabs } from '$lib/plugins-bridge/screen-chrome/route-tabs';

type WorkMetaScreenInput = {
	t: TranslateFn;
	work: WorkDetails;
	error?: string | null;
};

export function buildWorkNotFoundScreen(
	t: TranslateFn,
	message?: string | null
): ImmutableScreen {
	return ui.screen(
		{
			id: 'work-meta:not-found:screen',
			title: t('work_not_found_title')
		},
		[
			ui.stack(
				[
					...(message
						? [
								ui.status('warning', message, {
									title: t('work_not_found_title')
								})
							]
						: []),
					ui.empty(t('work_not_found_title'), {
						message: message ?? t('work_not_found_message')
					})
				]
			)
		]
	);
}

export function buildWorkDetailsScreen(input: WorkMetaScreenInput): ImmutableScreen {
	const viewModel = createWorkMetadataViewModel(input.work, input.t);

	return withRouteTabs(
		ui.screen(
			{
				id: `work-meta:${input.work.id}:screen`,
				title: input.work.title
			},
			[
				ui.stack(
					[
						...(input.error
							? [
									ui.status('warning', input.error, {
										title: input.t('work_details_warning_title')
									})
								]
							: []),
						workMetadataLayout(viewModel, {
							keyPrefix: `work-meta:${input.work.id}:metadata`,
							disclosureBinding: `workMeta.${input.work.id}.detailsExpanded`
						})
					]
				)
			]
		),
		buildMangaWorkRouteTabs(input.work.id, input.t, 'meta')
	);
}
