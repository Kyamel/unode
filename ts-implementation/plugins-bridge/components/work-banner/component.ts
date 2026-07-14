import { ui, type Immutable, type UiNode } from '$lib/unode/core';
import type { WorkBannerViewModel } from './view-model';

export type WorkBannerNode = Immutable<UiNode>;

export function workBanner(
	viewModel: WorkBannerViewModel,
): WorkBannerNode {

	return ui.stack(
		{ gap: 'sm' },
		[
			ui.media({
				ref: viewModel.coverRef,
				mediaKind: 'cover',
				alt: viewModel.coverAlt,
				aspectRatio: 'poster'
			}),
			ui.stack(
				[
					ui.text(viewModel.title, {
						role: 'title',
						truncate: true
					}),
					ui.inline(
						{
							gap: 'xs',
							wrap: true
						},
						viewModel.meta.map((part) =>
							ui.text(part.label, {
								role: part.role
							})
						)
					),
					ui.inline(
						{ gap: 'xs', wrap: true },
						viewModel.badges.map((badge) =>
							ui.badge(badge.label, badge.tone)
						)
					)
				]
			)
		]
	);
}
