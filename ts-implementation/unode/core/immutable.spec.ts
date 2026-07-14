import { describe, expect, it } from 'vitest';
import { deepFreeze } from './immutable';

describe('deepFreeze', () => {
	it('freezes nested objects and arrays', () => {
		const value = deepFreeze({
			label: 'hello',
			nested: {
				count: 1
			},
			items: [{ id: '1' }]
		});

		expect(Object.isFrozen(value)).toBe(true);
		expect(Object.isFrozen(value.nested)).toBe(true);
		expect(Object.isFrozen(value.items)).toBe(true);
		expect(Object.isFrozen(value.items[0])).toBe(true);
	});
});
