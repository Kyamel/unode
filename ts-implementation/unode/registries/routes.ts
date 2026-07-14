import type { RouteDefinition, RouteMatch, ResolvedRouteInfo } from '../api/contracts';
import type { HostApi } from '../api/host';
import { MemoryStateStore } from '../core/state';
import { normalizeScreen } from '../core/normalize';
import type { PluginI18nApi, PluginManifest, PluginRoute, ResolvedRoute } from '../core/runtime';
import { createRenderContext } from '../runtime/context';

export type RegisteredRoute = {
	pluginId: string;
	route: RouteDefinition<any>;
};

export type RegisteredRouteMatch = {
	route: RouteDefinition<any>;
	pluginId: string;
	params: Record<string, string>;
};

function normalizePath(pathname: string): string {
	if (pathname.length > 1 && pathname.endsWith('/')) return pathname.slice(0, -1);
	return pathname || '/';
}

function matchRoutePattern(pattern: string, pathname: string): RouteMatch | null {
	const normalizedPattern = normalizePath(pattern);
	const normalizedPath = normalizePath(pathname);

	if (normalizedPattern === normalizedPath) return { params: {} };

	const patternParts = normalizedPattern.split('/').filter(Boolean);
	const pathParts = normalizedPath.split('/').filter(Boolean);

	if (patternParts.length !== pathParts.length) return null;

	const params: Record<string, string> = {};
	for (let i = 0; i < patternParts.length; i += 1) {
		const patternPart = patternParts[i];
		const pathPart = pathParts[i];
		if (patternPart.startsWith(':')) {
			const key = patternPart.slice(1);
			if (!key) return null;
			params[key] = decodeURIComponent(pathPart);
			continue;
		}
		if (patternPart !== pathPart) return null;
	}

	return { params };
}

export class RouteRegistry {
	private routes: RegisteredRoute[] = [];

	register(def: RouteDefinition<any>, pluginId: string) {
		this.routes.push({ route: def, pluginId });
	}

	registerCore<THostApi extends HostApi>(
		def: PluginRoute<unknown, THostApi>,
		plugin: PluginManifest,
		host: THostApi,
		i18n: PluginI18nApi
	) {
		const screenKind = this.createScreenKind(plugin.id, def.pattern);

		this.register(
			{
				path: def.pattern,
				screenKind,
				async render(routeCtx) {
					const route: ResolvedRoute = {
						pattern: def.pattern,
						params: routeCtx.params,
						query: Object.fromEntries(routeCtx.query.entries())
					};
					const state = new MemoryStateStore();
					const renderCtx = createRenderContext(plugin, host, route, state, i18n);
					const data = await def.load(renderCtx);

					if (data && typeof data === 'object' && !Array.isArray(data)) {
						state.mergeData(data as Record<string, unknown>);
					}

					const body = normalizeScreen(def.render(data, renderCtx));
					if (body.initialState) {
						state.mergeData(body.initialState as Record<string, unknown>);
					}

					return {
						screenKind,
						title: typeof body.title === 'string' ? body.title : undefined,
						body,
						state,
						meta: body.meta && typeof body.meta === 'object' ? { ...body.meta } : undefined,
						layout: 'default'
					};
				}
			},
			plugin.id
		);
	}

	resolve(pathname: string): RegisteredRouteMatch | null {
		const normalizedPath = normalizePath(pathname);
		const matches = this.routes
			.map((entry) => {
				const match = entry.route.match
					? entry.route.match(normalizedPath)
					: matchRoutePattern(entry.route.path, normalizedPath);
				if (!match) return null;
				return { route: entry.route, pluginId: entry.pluginId, params: match.params };
			})
			.filter(Boolean) as RegisteredRouteMatch[];

		if (!matches.length) return null;
		matches.sort((a, b) => (b.route.priority ?? 0) - (a.route.priority ?? 0));
		return matches[0] ?? null;
	}

	resolveRouteInfo(pathname: string): ResolvedRouteInfo | null {
		const match = this.resolve(pathname);
		if (!match) return null;
		return {
			pathname,
			params: match.params,
			screenKind: match.route.screenKind,
			pluginId: match.pluginId
		};
	}

	private createScreenKind(pluginId: string, pattern: string): string {
		const cleaned = pattern
			.replaceAll('/', '.')
			.replaceAll(':', '$')
			.replaceAll(/^\.+|\.+$/g, '');
		return `${pluginId}.${cleaned || 'index'}`;
	}
}
