import { deepFreeze } from './immutable';

export type MessageValue = string | number | boolean | null | undefined;
export type MessageValues = Record<string, MessageValue>;

export interface MessageCatalog {
	[key: string]: string | MessageCatalog;
}

export type MessageCatalogs = Record<string, MessageCatalog>;

export interface CoreTranslator {
	locale(): string;
	t(key: string, values?: MessageValues, fallback?: string): string;
}

function normalizeLocale(locale: string): string {
	return locale.trim().toLowerCase();
}

function findCatalog(catalogs: MessageCatalogs, locale: string): MessageCatalog | null {
	const normalized = normalizeLocale(locale);
	const exact = catalogs[normalized];
	if (exact) return exact;

	const exactMatch = Object.entries(catalogs).find(
		([key]) => normalizeLocale(key) === normalized
	)?.[1];
	if (exactMatch) return exactMatch;

	const base = normalized.split('-')[0];
	const baseMatch = Object.entries(catalogs).find(([key]) => {
		const normalizedKey = normalizeLocale(key);
		return normalizedKey === base || normalizedKey.startsWith(`${base}-`);
	})?.[1];
	if (baseMatch) return baseMatch;

	return catalogs.en ?? Object.values(catalogs)[0] ?? null;
}

function lookupMessage(catalog: MessageCatalog, key: string): string | null {
	if (key in catalog && typeof catalog[key] === 'string') {
		return catalog[key] as string;
	}

	const segments = key.split('.');
	let current: string | MessageCatalog | undefined = catalog;
	for (const segment of segments) {
		if (!current || typeof current === 'string' || !(segment in current)) {
			return null;
		}
		current = current[segment] as string | MessageCatalog | undefined;
	}

	return typeof current === 'string' ? current : null;
}

function interpolate(template: string, values?: MessageValues): string {
	if (!values) return template;
	return template.replace(/\{(\w+)\}/g, (_match, key: string) => {
		const value = values[key];
		return value === null || value === undefined ? '' : String(value);
	});
}

export class CoreI18nRegistry {
	private readonly catalogsByPlugin = new Map<string, MessageCatalogs>();

	register(pluginId: string, catalogs: MessageCatalogs): void {
		this.catalogsByPlugin.set(pluginId, deepFreeze({ ...catalogs }));
	}

	has(pluginId: string): boolean {
		return this.catalogsByPlugin.has(pluginId);
	}

	getCatalogs(pluginId: string): MessageCatalogs | undefined {
		return this.catalogsByPlugin.get(pluginId);
	}

	translate(
		pluginId: string,
		locale: string,
		key: string,
		values?: MessageValues,
		fallback?: string
	): string {
		const catalogs = this.catalogsByPlugin.get(pluginId);
		if (!catalogs) return fallback ?? key;

		const activeCatalog = findCatalog(catalogs, locale);
		const fallbackCatalog = findCatalog(catalogs, 'en');
		const template =
			(activeCatalog ? lookupMessage(activeCatalog, key) : null) ??
			(fallbackCatalog ? lookupMessage(fallbackCatalog, key) : null) ??
			fallback ??
			key;

		return interpolate(template, values);
	}

	createTranslator(pluginId: string, locale: string): CoreTranslator {
		return {
			locale: () => locale,
			t: (key, values, fallback) => this.translate(pluginId, locale, key, values, fallback)
		};
	}
}
