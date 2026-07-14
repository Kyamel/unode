import { ui } from '$lib/unode/core/dsl';
import type { Immutable, UiNode } from '$lib/unode/core';
import type {
	WorkMetadataDisclosureBinding,
	WorkMetadataFieldViewModel,
	WorkMetadataTaxonomyGroupViewModel,
	WorkMetadataViewModel
} from './view-model';

export type WorkMetadataNode = Immutable<UiNode>;

export type WorkMetadataDetailsDisclosureOptions = {
	keyPrefix?: string;
	binding: WorkMetadataDisclosureBinding;
};

export type WorkMetadataLayoutOptions = {
	keyPrefix?: string;
	disclosureBinding: WorkMetadataDisclosureBinding;
};

function metadataField(label: string, value: string | UiNode) {
	return ui.stack(
		[
			ui.text(label, { role: 'label' }),
			typeof value === 'string'
				? ui.text(value, { role: 'body' })
				: value
		]
	);
}

function fieldGrid(
	fields: readonly WorkMetadataFieldViewModel[],
): WorkMetadataNode {
	return ui.grid(
		{
			columns: {
				base: 1,
				sm: 2
			},
			gap: 'md'
		},
		fields.map((field) => metadataField(field.label, field.value))
	);
}

function taxonomyField(group: WorkMetadataTaxonomyGroupViewModel) {
	return ui.stack(
		[
			ui.text(group.label, { role: 'label' }),
			...(group.badges.length
				? [
						ui.inline(
							{
								gap: 'sm',
								wrap: true
							},
							group.badges.map((badge) =>
								ui.badge(badge.label, badge.tone)
							)
						)
					]
				: [
						ui.text(group.emptyLabel, {
							role: 'body'
						})
					])
		]
	);
}

function workMetadataTaxonomyGrid(
	viewModel: WorkMetadataViewModel,
): WorkMetadataNode {

	return ui.grid(
		{
			columns: {
				base: 1,
				sm: 2
			},
			gap: 'md'
		},
		viewModel.details.taxonomyGroups.map((group) => taxonomyField(group))
	);
}

export function workMetadataSummaryGrid(
	viewModel: WorkMetadataViewModel,
): WorkMetadataNode {

	return fieldGrid(viewModel.summaryFields);
}

function workMetadataTechnicalGrid(
	viewModel: WorkMetadataViewModel,
): WorkMetadataNode {
	return fieldGrid(viewModel.details.technicalFields);
}

export function workMetadataDetailsDisclosure(
	viewModel: WorkMetadataViewModel,
	options: WorkMetadataDetailsDisclosureOptions
): WorkMetadataNode {
	return ui.disclosure(
		{
			binding: options.binding,
			label: viewModel.details.showMoreLabel,
			labelExpanded: viewModel.details.showLessLabel
		},
		[
			ui.stack(
				[
					workMetadataTaxonomyGrid(viewModel),
					workMetadataTechnicalGrid(viewModel)
				]
			)
		]
	);
}

export function workMetadataLayout(
	viewModel: WorkMetadataViewModel,
	options: WorkMetadataLayoutOptions
): WorkMetadataNode {
	const keyPrefix = options.keyPrefix ?? `work-metadata:${viewModel.id}`;

	return ui.grid(
		{
			columns: {
				base: 1,
				md: 2
			},
			gap: 'lg'
		},
		[
			ui.media({
				ref: viewModel.coverRef,
				mediaKind: 'cover',
				alt: viewModel.coverAlt,
				aspectRatio: 'poster',
				expandable: viewModel.coverExpandable
			}),
			ui.stack(
				[
					ui.stack(
						[
							ui.text(viewModel.title, {
								role: 'heading',
								truncate: true
							}),
							ui.text(viewModel.subtitle, {
								role: 'subtitle'
							})
						]
					),
					workMetadataSummaryGrid(viewModel),
					workMetadataDetailsDisclosure(viewModel, {
						keyPrefix: `${keyPrefix}:disclosure`,
						binding: options.disclosureBinding
					})
				]
			)
		]
	);
}
