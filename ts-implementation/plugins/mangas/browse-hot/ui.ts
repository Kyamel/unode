import { ui, type CollectionContinuation, type ImmutableScreen } from '$lib/unode/core';
import type { TranslateFn } from '$lib/unode/core/runtime';
import {
	createWorkBannerViewModel,
	workBanner
} from '$lib/plugins-bridge/components';
import {
	buildMangaBrowseRouteTabs,
	type MangaBrowseRouteTabId
} from '$lib/plugins-bridge/domains/manga/route-tabs';
import { navigateAction } from '$lib/plugins-bridge/navigation';
import type { WorkSummary } from '$lib/plugins-bridge/models';
import { withRouteTabs } from '$lib/plugins-bridge/screen-chrome/route-tabs';

type BrowseScreenInput = {
	t: TranslateFn;
	works: WorkSummary[];
	error: string | null;
	emptyText: string;
	activeTab: MangaBrowseRouteTabId;
	continuation?: CollectionContinuation;
};

export function buildBrowseScreen(input: BrowseScreenInput): ImmutableScreen {
	const banners = input.works.map((work) => ({
		viewModel: createWorkBannerViewModel(work, input.t),
		action: navigateAction(`/mangas/${work.id}/meta`)
	}));

	return withRouteTabs(
		ui.screen(
			{
				id: 'screen',
				title: input.t('catalog_browse_title')
			},
			[
				ui.stack(
					{ gap: 'lg' },
					[
						ui.stack({ gap: 'xs' }, [
							ui.text(input.t('catalog_browse_title'), { role: 'heading' }),
							ui.text(input.t('catalog_browse_subtitle'), { role: 'subtitle' })
						]),
						...(input.error
							? [
									ui.status('warning', input.error, {
										title: input.t('failed_to_load')
									})
								]
							: []),
						...(banners.length
							? [
									ui.grid(
										{
											columns: {
												base: 1,
												sm: 2,
												md: 3,
												lg: 4,
												xl: 5
											},
											gap: 'lg'
										},
										banners.map((banner) =>
											ui.pressable(
												workBanner(banner.viewModel),
												banner.action,
												{
													label: banner.viewModel.title
												}
											)
										)
									)
								]
							: [
									ui.empty(input.t('catalog_browse_title'), {
										message: input.emptyText
									})
								])
					]
				)
			]
		),
		buildMangaBrowseRouteTabs(input.t as TranslateFn, input.activeTab)
	);
}
