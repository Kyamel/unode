import { ui } from '$lib/unode/core/dsl';
import type { Immutable, UiNode } from '$lib/unode/core';
import type { ChapterLanguageFilterViewModel } from './view-model';

export type ChapterLanguageFilterNode = Immutable<UiNode>;

export function chapterLanguageFilterToolbar(
	viewModel: ChapterLanguageFilterViewModel,
): ChapterLanguageFilterNode {

	return ui.inline(
		{
			gap: 'sm',
			wrap: true,
			align: 'start'
		},
		[
			ui.menu({
				label: viewModel.menuLabel,
				intent: 'secondary',
				align: 'start',
				items: viewModel.options.map((option) =>
					ui.menuItem(option.label, option.action, {
						selected: option.selected
					})
				)
			}),
			ui.badge(viewModel.countLabel, 'default'),
			...(viewModel.activeFilterLabel
				? [
						ui.badge(viewModel.activeFilterLabel, 'info')
					]
				: [])
		]
	);
}
