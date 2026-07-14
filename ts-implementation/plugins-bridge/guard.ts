import type { PluginManifest } from '$lib/unode/core/runtime';
import type { HostApi } from '$lib/unode/api/host';
import { guardPermission } from '$lib/unode/utils/permissions';
import type { MugenHostApi } from './host';

function guardAll<T extends object>(plugin: PluginManifest, permission: string, target: T): T {
	const entries = Object.entries(target).map(([key, value]) => {
		if (typeof value !== 'function') return [key, value] as const;
		return [
			key,
			guardPermission(plugin, permission, value as (...args: never[]) => unknown)
		] as const;
	});
	return Object.fromEntries(entries) as T;
}

export function guardMugenHostApi(plugin: PluginManifest, host: HostApi): HostApi {
	const api = host as MugenHostApi;

	return {
		catalog: guardAll(plugin, 'catalog.read', api.catalog),
		users: guardAll(plugin, 'users.read', api.users),
		auth: guardAll(plugin, 'auth.session.read', api.auth),
		navigation: guardAll(plugin, 'navigation.write', api.navigation),
		feedback: {
			toast: guardPermission(plugin, 'feedback.toast', api.feedback.toast),
			confirm: guardPermission(plugin, 'feedback.dialog', api.feedback.confirm)
		},
		reader: guardAll(plugin, 'reader.open', api.reader),
		storage: api.storage,
		http: api.http,
		i18n: api.i18n,
		events: api.events,
		system: api.system
	} as MugenHostApi;
}
