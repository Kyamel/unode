import type { HostApi } from '../api/host';
import type { PluginManifest } from '../core/runtime';
import { hasPermission } from '../utils/permissions';

function hasStorageReadPermission(plugin: PluginManifest): boolean {
	return (
		hasPermission(plugin, 'storage.session.read') ||
		hasPermission(plugin, 'storage.persistent.read')
	);
}

function hasStorageWritePermission(plugin: PluginManifest): boolean {
	return (
		hasPermission(plugin, 'storage.session.write') ||
		hasPermission(plugin, 'storage.persistent.write')
	);
}

function hasHttpPermission(plugin: PluginManifest): boolean {
	return hasPermission(plugin, 'http.fetch');
}

export function assertCoreStoragePermission(
	plugin: PluginManifest,
	scope: 'session' | 'persistent',
	access: 'read' | 'write'
): void {
	const allowed =
		access === 'read'
			? hasPermission(plugin, `storage.${scope}.read`)
			: hasPermission(plugin, `storage.${scope}.write`);

	if (!allowed) {
		throw new Error(`Missing permission: storage.${scope}.${access}`);
	}
}

export function guardCoreHostApi<THostApi extends HostApi>(
	plugin: PluginManifest,
	host: THostApi
): THostApi {
	return {
		...host,
		storage: {
			...host.storage,
			getScoped: (async (...args) => {
				if (!hasStorageReadPermission(plugin)) {
					throw new Error('Missing permission: storage.read');
				}
				return await host.storage.getScoped(...args);
			}) as THostApi['storage']['getScoped'],
			setScoped: (async (...args) => {
				if (!hasStorageWritePermission(plugin)) {
					throw new Error('Missing permission: storage.write');
				}
				return await host.storage.setScoped(...args);
			}) as THostApi['storage']['setScoped']
		},
		http: {
			...host.http,
			request: (async (...args) => {
				if (!hasHttpPermission(plugin)) {
					throw new Error('Missing permission: http.fetch');
				}
				return await host.http.request(...args);
			}) as THostApi['http']['request']
		},
		events: {
			...host.events,
			emit: ((...args) => {
				if (!hasPermission(plugin, 'events.write')) {
					throw new Error('Missing permission: events.write');
				}
				return host.events.emit(...args);
			}) as THostApi['events']['emit'],
			on: ((...args) => {
				if (!hasPermission(plugin, 'events.read')) {
					throw new Error('Missing permission: events.read');
				}
				return host.events.on(...args);
			}) as THostApi['events']['on']
		}
	} as THostApi;
}
