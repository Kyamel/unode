import { ui } from '$lib/unode/core/dsl';
import type { Immutable, UiNode } from '$lib/unode/core';
import type { ChapterListViewModel } from './view-model';

export type ChapterListNode = Immutable<UiNode>;

export function chapterList(
	viewModel: ChapterListViewModel,
): ChapterListNode {

	if (!viewModel.items.length) {
		return ui.empty(viewModel.emptyTitle, {
			message: viewModel.emptyMessage
		});
	}

	return ui.list(
		viewModel.items.map((item) =>
			ui.item(
				item.key,
				[
					ui.text(item.title, {
						role: 'body'
					})
				],
				{
					leading: item.numberLabel
						? [
								ui.badge(item.numberLabel, 'default', {
								})
							]
						: undefined,
					secondary: [
						...(item.subtitle
							? [
									ui.text(item.subtitle, {
										role: 'caption'
									})
								]
							: []),
						ui.badge(item.languageLabel, 'info')
					],
					action: item.action
				}
			)
		),
		{
			continuation: viewModel.continuation
		}
	);
}
