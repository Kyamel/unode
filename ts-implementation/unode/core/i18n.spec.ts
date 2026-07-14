import { describe, expect, it } from 'vitest';
import { CoreI18nRegistry } from './i18n';

describe('CoreI18nRegistry', () => {
	it('resolves exact locale matches and interpolates values', () => {
		const registry = new CoreI18nRegistry();
		registry.register('demo.plugin', {
			en: {
				greeting: 'Hello {name}'
			},
			'pt-br': {
				greeting: 'Ola {name}'
			}
		});

		expect(registry.translate('demo.plugin', 'pt-BR', 'greeting', { name: 'Lucas' })).toBe(
			'Ola Lucas'
		);
	});

	it('falls back to english when the requested locale is missing', () => {
		const registry = new CoreI18nRegistry();
		registry.register('demo.plugin', {
			en: {
				title: 'Browse hot'
			}
		});

		expect(registry.translate('demo.plugin', 'es-ES', 'title')).toBe('Browse hot');
	});
});
