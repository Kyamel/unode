import type { ImmutableScreen } from '$lib/unode/core/ast';
import { ui } from '$lib/unode/core/dsl';
import type { TranslateFn } from '$lib/unode/core/runtime';
import { buildMangaWorkRouteTabs } from '$lib/plugins-bridge/domains/manga/route-tabs';
import type { WorkStaff } from '$lib/plugins-bridge/models';
import { withRouteTabs } from '$lib/plugins-bridge/screen-chrome/route-tabs';

type StaffScreenInput = {
	t: TranslateFn;
	workId: string;
	staff: WorkStaff[];
	error: string | null;
};

function buildStaffList(input: StaffScreenInput) {
	return ui.list(
		input.staff.map((person) =>
			ui.item(
				person.id,
				[
					ui.text(person.name ?? input.t('value_na'), {
						role: 'body'
					})
				],
				{
					secondary: person.role
						? [
								ui.text(person.role, {
									role: 'caption'
								})
							]
						: undefined
				}
			)
		),
	);
}

export function buildStaffScreen(input: StaffScreenInput): ImmutableScreen {
	const content =
		!input.error && input.staff.length
			? buildStaffList(input)
			: ui.empty(input.t('catalog_staff_empty'), {
					message: input.error ?? input.t('staff_screen_subtitle')
				});

	return withRouteTabs(
		ui.screen(
			{
				id: `work-staff:${input.workId}:screen`,
				title: input.t('manga_tab_staff')
			},
			[
				ui.stack(
					[
						ui.stack(
						[
							ui.text(input.t('manga_tab_staff'), {
								role: 'heading'
							}),
							ui.text(input.t('staff_screen_subtitle'), {
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
		buildMangaWorkRouteTabs(input.workId, input.t, 'staff')
	);
}
