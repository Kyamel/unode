import { ui } from '$lib/unode/core/dsl';
import type { Immutable, UiNode } from '$lib/unode/core';
import type { WorkListViewModel } from './view-model';

export type WorkListNode = Immutable<UiNode>;

export function workList(viewModel: WorkListViewModel): WorkListNode {

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
					leading: item.coverRef
						? [
								ui.media({
									ref: item.coverRef,
									mediaKind: 'cover',
									alt: item.coverAlt ?? item.title,
									aspectRatio: 'poster'
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
						...item.badges.map((badge) =>
							ui.badge(badge.label, badge.tone)
						)
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
