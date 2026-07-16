// Node styling — how each semantic node type looks in THIS app. Recipes are
// plain functions from recipe context to DOM vnodes; no framework involved.
// The `action` recipe punches a host-slot hole that `Button.tsx` fills.
import { h, hostSlot, type ActionRef, type Recipe } from 'unode-react';

/** Unwraps the IR literal wrapper (`{ v: ... }`) when present. */
export function literal(value: unknown): unknown {
	if (value && typeof value === 'object' && 'v' in value) {
		return (value as { v: unknown }).v;
	}
	return value;
}

export function text(value: unknown): string {
	return String(literal(value) ?? '');
}

/** The screen root: heading (title/subtitle) plus the node stack. */
export const screenRecipe: Recipe = ({ props, children }) =>
	h(
		'section',
		{ class: 'pg-screen' },
		h(
			'div',
			{ class: 'pg-screen-heading' },
			h('div', {}, h('h1', {}, text(props.title)), props.subtitle ? h('p', {}, text(props.subtitle)) : null),
		),
		h('div', { class: 'pg-node-stack' }, children),
	);

export const nodeRecipes: Record<string, Recipe> = {
	// Semantic action -> the host's native Button (see Button.tsx).
	action: ({ label, intent, action }) => hostSlot('Button', { children: label, intent, action }),
	actions: ({ children }) => h('div', { class: 'pg-inline' }, children),
	section: ({ title, prop, children }) =>
		h(
			'section',
			{ class: 'pg-section' },
			title || prop('description')
				? h(
						'div',
						{ class: 'pg-section-title' },
						title ? h('h2', {}, title) : null,
						prop('description') ? h('p', {}, text(prop('description'))) : null,
					)
				: null,
			h('div', { class: 'pg-node-stack' }, children),
		),
	stack: ({ children }) => h('div', { class: 'pg-node-stack' }, children),
	inline: ({ children }) => h('div', { class: 'pg-inline' }, children),
	text: ({ content, role, prop }) =>
		h('p', { class: `pg-text role-${role} tone-${text(prop('tone')) || 'neutral'}` }, content),
	grid: ({ children, prop }) =>
		h('div', { class: `pg-grid pg-grid-${Number(prop('maxColumns', 2)) || 2}` }, children),
	badge: ({ label, prop }) =>
		h('span', { class: `pg-badge tone-${text(prop('tone')) || 'neutral'}` }, label),
	value: ({ prop }) =>
		h('strong', { class: `pg-value tone-${text(prop('tone')) || 'neutral'}` }, text(prop('value'))),
	list: ({ childNodes, renderChildren }) => h('div', { class: 'pg-list' }, renderChildren(childNodes)),
	item: ({ childNodes, renderChildren, props, dispatch }) =>
		h(
			'button',
			{
				class: `pg-list-item ${props.action ? 'is-clickable' : ''}`,
				disabled: !props.action,
				onClick: () => props.action && dispatch(props.action as ActionRef),
			},
			h('span', {}, renderChildren(childNodes)),
		),
};

export const fallbackRecipe: Recipe = ({ children }) => children;
