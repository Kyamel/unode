import type { HostCallHandler } from "./pluginHost";
import { PluginInstance } from "./pluginHost";
import type { ResolvedRoute } from "./session";

export type PluginWasmSource = BufferSource | Response | PromiseLike<Response>;

export interface WebPluginRegistration {
  id: string;
  /** Primary route pattern (e.g. `/notes`). */
  routePattern: string;
  /**
   * Additional route patterns this plugin renders (e.g. `/notes/:id`), so one
   * plugin can own multiple screens. Patterns support `:param` segments.
   */
  routePatterns?: string[];
  loadWasm: () => PluginWasmSource | Promise<PluginWasmSource>;
}

export interface ResolvedWebPlugin {
  registration: WebPluginRegistration;
  route: ResolvedRoute;
}

export interface InstantiatedWebPlugin extends ResolvedWebPlugin {
  plugin: PluginInstance;
}

function normalizePath(pathname: string): string {
  if (pathname.length > 1 && pathname.endsWith("/")) return pathname.slice(0, -1);
  return pathname === "" ? "/" : pathname;
}

/** Mirrors the Rust `RouteRegistry` pattern matcher (`/notes/:id` style). */
export function matchRoutePattern(
  pattern: string,
  pathname: string,
): Record<string, string> | undefined {
  const normalizedPattern = normalizePath(pattern);
  const normalizedPath = normalizePath(pathname);

  if (normalizedPattern === normalizedPath) return {};

  const patternParts = normalizedPattern.split("/").filter((segment) => segment.length > 0);
  const pathParts = normalizedPath.split("/").filter((segment) => segment.length > 0);
  if (patternParts.length !== pathParts.length) return undefined;

  const params: Record<string, string> = {};
  for (let index = 0; index < patternParts.length; index += 1) {
    const patternPart = patternParts[index];
    const pathPart = pathParts[index];
    if (patternPart.startsWith(":")) {
      const name = patternPart.slice(1);
      if (name.length === 0) return undefined;
      params[name] = pathPart;
      continue;
    }
    if (patternPart !== pathPart) return undefined;
  }

  return params;
}

export interface RoutePatternMatch {
  pattern: string;
  params: Record<string, string>;
}

/**
 * Resolves `pathname` against a list of route patterns (e.g. a plugin
 * manifest's declared routes). Exact matches win over `:param` matches
 * regardless of order.
 */
export function resolveRoutePattern(
  patterns: string[],
  pathname: string,
): RoutePatternMatch | undefined {
  let paramMatch: RoutePatternMatch | undefined;

  for (const pattern of patterns) {
    const params = matchRoutePattern(pattern, pathname);
    if (!params) continue;
    if (Object.keys(params).length === 0) return { pattern, params };
    paramMatch ??= { pattern, params };
  }

  return paramMatch;
}

export class WebPluginRegistry {
  private readonly plugins: WebPluginRegistration[] = [];

  register(plugin: WebPluginRegistration): this {
    this.plugins.push(plugin);
    return this;
  }

  resolve(pathname: string, query: Record<string, string> = {}): ResolvedWebPlugin | undefined {
    let paramMatch: ResolvedWebPlugin | undefined;

    for (const registration of this.plugins) {
      const patterns = [registration.routePattern, ...(registration.routePatterns ?? [])];
      for (const pattern of patterns) {
        const params = matchRoutePattern(pattern, pathname);
        if (!params) continue;

        const resolved: ResolvedWebPlugin = { registration, route: { pattern, params, query } };
        // Exact matches win over `:param` matches regardless of registration order.
        if (Object.keys(params).length === 0) return resolved;
        paramMatch ??= resolved;
      }
    }

    return paramMatch;
  }

  async instantiateForPath(
    pathname: string,
    query: Record<string, string>,
    onHostCall?: HostCallHandler,
  ): Promise<InstantiatedWebPlugin> {
    const resolved = this.resolve(pathname, query);
    if (!resolved) {
      throw new Error(`No Unode plugin registered for route ${pathname}`);
    }

    const plugin = await PluginInstance.instantiate(
      await resolved.registration.loadWasm(),
      onHostCall,
    );

    return { ...resolved, plugin };
  }
}
