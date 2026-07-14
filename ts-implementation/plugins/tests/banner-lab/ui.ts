import { ui, type CollectionContinuation, type ImmutableScreen } from '$lib/unode/core';
import type { TranslateFn } from '$lib/unode/core/runtime';
import {
	createWorkBannerViewModel,
	createWorkMetadataViewModel,
	workBanner,
	workMetadataLayout,
	type WorkBannerSource
} from '$lib/plugins-bridge/components';
import type { WorkDetails } from '$lib/plugins-bridge/models';

type BannerLabScreenInput = {
	t: TranslateFn;
	works: WorkBannerSource[];
	featuredWork: WorkDetails | null;
	error?: string | null;
	galleryContinuation?: CollectionContinuation;
};

const DETAILS_EXPANDED_PATH = 'bannerLab.detailsExpanded';

export function buildBannerLabScreen(input: BannerLabScreenInput): ImmutableScreen {
	const { t, works, featuredWork, error, galleryContinuation } = input;
	const featured = works[0];
	const featuredBanner = featured ? createWorkBannerViewModel(featured, t) : null;
	const galleryBanners = works.map((work) => createWorkBannerViewModel(work, t));
	const featuredMetadata = featuredWork ? createWorkMetadataViewModel(featuredWork, t) : null;

	if (!works.length && error) {
		return ui.screen(
			{
				id: 'error-screen',
				title: t('screen_title')
			},
			[
				ui.status('danger', error, {
					title: t('error_title')
				}),
				ui.empty( t('empty_title'), {
					message: t('error_message')
				})
			]
		);
	}

	if (!works.length) {
		return ui.screen(
			{
				id: 'empty-screen',
				title: t('screen_title')
			},
			[
				ui.empty( t('empty_title'), {
					message: t('empty_message')
				})
			]
		);
	}

	return ui.screen(
		{
			id: 'bannerLab:screen',
			title: t('screen_title')
		},
		[
			ui.stack(
				{ gap: 'lg' },
				[
					ui.stack(
						{ gap: 'xs' },
						[
							ui.text( t('screen_title'), { role: 'heading' }),
							ui.text( t('screen_subtitle'), { role: 'subtitle' })
						]
					),
					...(error
						? [
								ui.status('warning', error, {
									title: t('error_title')
								})
							]
						: []),
					ui.section(
						{
							title: t('featured_label')
						},
						featuredBanner ? [workBanner(featuredBanner)] : []
					),
					...(featuredMetadata
						? [
								ui.section(
									{
										title: t('metadata_layout_label'),
										description: t('metadata_layout_description')
									},
									[
										workMetadataLayout(featuredMetadata, { disclosureBinding: DETAILS_EXPANDED_PATH })
									]
								)
							]
						: []),
					ui.section(
						{
							title: t('gallery_label'),
							description: t('gallery_subtitle')
						},
						[
							ui.grid(
								{
									columns: {
										base: 1,
										sm: 2,
										md: 3,
										lg: 4,
										xl: 5
									},
									gap: 'lg',
									continuation: galleryContinuation
								},
								galleryBanners.map((viewModel) => workBanner(viewModel))
							)
						]
					)
				]
			)
		]
	);
}
