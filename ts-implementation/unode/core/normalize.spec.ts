import { describe, expect, it } from 'vitest';
import { expr, ui } from './dsl';
import { normalizeNode, normalizeScreen, toTransportScreen } from './normalize';

describe('normalize ui tree', () => {
	it('fills common defaults in canonical nodes', () => {
		const screen = ui.screen(
			{
				id: 'browse-screen',
				title: 'Browse'
			},
			[
				ui.stack({ id: 'browse:stack' }, [ui.text('Hello', { id: 'browse:hello' })]),
				ui.inline({ id: 'browse:inline' }, [ui.badge('New', undefined, { id: 'browse:new' })]),
				ui.grid(
					{
						id: 'browse:grid',
						maxColumns: 3
					},
					[ui.text('One', { id: 'browse:one' })]
				)
			]
		);

		const normalized = normalizeScreen(screen);

		expect(normalized.children[0]).toMatchObject({
			kind: 'stack',
			gap: 'md'
		});
		expect(normalized.children[1]).toMatchObject({
			kind: 'inline',
			gap: 'sm',
			wrap: false,
			align: 'start'
		});
		expect(normalized.children[2]).toMatchObject({
			kind: 'grid',
			gap: 'md',
			columns: {
				base: 3
			}
		});
	});

	it('normalizes nested pressable and action defaults', () => {
		const node = ui.pressable(
			ui.stack({ id: 'blue-box:stack' }, [ui.text('Blue Box', { id: 'blue-box:title' })]),
			{
				type: 'unode.navigate',
				params: {
					to: '/app/mangas/blue-box/meta'
				}
			},
			{
				id: 'blue-box:pressable'
			}
		);

		const normalized = normalizeNode(node);

		expect(normalized).toMatchObject({
			kind: 'pressable',
			_id: 'blue-box:pressable',
			_reactivity: 'static',
			_subtreeReactivity: 'static',
			child: {
				kind: 'stack',
				_id: 'blue-box:stack',
				gap: 'md'
			}
		});
	});

	it('canonicalizes literal expressions and tracks reactive fields', () => {
		const node = ui.text(expr.literal('Blue Box'), {
			id: 'hero-title',
			role: 'title'
		});
		const reactiveNode = ui.text(expr.binding('work.title'), {
			id: 'reactive-title',
			role: 'title'
		});

		const normalizedStatic = normalizeNode(node);
		const normalizedReactive = normalizeNode(reactiveNode);

		expect(normalizedStatic.kind).toBe('text');
		expect(normalizedReactive.kind).toBe('text');
		if (normalizedStatic.kind !== 'text' || normalizedReactive.kind !== 'text') {
			throw new Error('Expected text nodes');
		}

		expect(normalizedStatic.content).toBe('Blue Box');
		expect(normalizedStatic._key).toBe('hero-title');
		expect(normalizedStatic._reactivity).toBe('static');
		expect(normalizedStatic._subtreeReactivity).toBe('static');
		expect(normalizedStatic._staticFields).toMatchObject({
			content: 'Blue Box',
			role: 'title'
		});

		expect(normalizedReactive.content).toMatchObject({
			kind: 'binding',
			path: 'work.title'
		});
		expect(normalizedReactive._reactivity).toBe('reactive');
		expect(normalizedReactive._subtreeReactivity).toBe('reactive');
	});

	it('marks binding-driven conditionals as conditional canonical nodes', () => {
		const node = ui.when(
			expr.binding('details.expanded'),
			ui.text('Shown', { id: 'details:shown' }),
			undefined,
			{ id: 'details:conditional' }
		);
		const normalized = normalizeNode(node);

		expect(normalized.kind).toBe('conditional');
		if (normalized.kind !== 'conditional') {
			throw new Error('Expected conditional node');
		}

		expect(normalized._reactivity).toBe('conditional');
		expect(normalized._subtreeReactivity).toBe('conditional');
		expect(normalized.then).toMatchObject({
			_id: 'details:shown'
		});
	});

	it('propagates descendant reactivity into subtree metadata without mutating node-level reactivity', () => {
		const node = ui.stack({ id: 'reactive:stack' }, [
			ui.text(expr.binding('work.title'), {
				id: 'reactive:title',
				role: 'title'
			})
		]);

		const normalized = normalizeNode(node);

		expect(normalized.kind).toBe('stack');
		expect(normalized._reactivity).toBe('static');
		expect(normalized._subtreeReactivity).toBe('reactive');
	});

	it('keeps the transport layer equal to the normalized screen for now', () => {
		const screen = ui.screen(
			{
				id: 'transport-screen',
				title: 'Transport'
			},
			[
				ui.section({ id: 'transport:section' }, [ui.text('Body', { id: 'transport:body' })])
			]
		);

		expect(toTransportScreen(screen)).toEqual(normalizeScreen(screen));
	});

	it('uses structural path fallback when node has no explicit id', () => {
		// No id provided — normalizer falls back to ctx.path (e.g. "screen")
		// This is safe for static structures that never change position.
		const node = ui.text('No explicit id');
		const normalized = normalizeNode(node);
		expect(normalized._key).toBe('screen');
		expect(normalized._reactivity).toBe('static');
	});

	it('throws when siblings reuse the same id', () => {
		const node = ui.stack(
			{ id: 'duplicate-test:stack' },
			[
				ui.text('One', { id: 'duplicate-test:title' }),
				ui.text('Two', { id: 'duplicate-test:title' })
			]
		);

		expect(() => normalizeNode(node)).toThrow(/duplicate sibling identity/);
	});

	it('throws when a parent and child share the same id within a subtree', () => {
		// id is globally unique — parent and child cannot share it
		const node = ui.stack({ id: 'duplicate-test:shared' }, [
			ui.text('Child', { id: 'duplicate-test:shared' })
		]);

		expect(() => normalizeNode(node)).toThrow(/duplicate global id/);
	});

	it('throws when a parent and child share the same id in the global tree', () => {
		const screen = ui.screen(
			{
				id: 'duplicate-test:screen',
				title: 'Duplicate ids'
			},
			[
				ui.stack(
					{ id: 'duplicate-test:shared' },
					[ui.text('Child', { id: 'duplicate-test:shared' })]
				)
			]
		);

		expect(() => normalizeScreen(screen)).toThrow(/duplicate global id/);
	});

	it('includes node kinds in duplicate identity errors', () => {
		const screen = ui.screen(
			{
				id: 'duplicate-test:screen',
				title: 'Kinds'
			},
			[
				ui.stack(
					{ id: 'duplicate-test:stack' },
					[
						ui.text('One', { id: 'duplicate-test:title' }),
						ui.text('Two', { id: 'duplicate-test:title' })
					]
				)
			]
		);

		expect(() => normalizeScreen(screen)).toThrow(/screen > stack\[0] > text\[1]/);
	});

	it('throws when menu items reuse the same identity', () => {
		const node = ui.menu({
			id: 'duplicate-test:menu',
			label: 'Menu',
			items: [
				ui.menuItem('One', { type: 'noop' }, { id: 'duplicate-test:item' }),
				ui.menuItem('Two', { type: 'noop' }, { id: 'duplicate-test:item' })
			]
		});

		expect(() => normalizeNode(node)).toThrow(/duplicate menu item identity/);
	});
});