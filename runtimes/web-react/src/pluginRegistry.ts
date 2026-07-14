import type { ResolvedRoute } from "./bridge";
import { PluginInstance, type HostCallHandler } from "./pluginHost";

export type PluginWasmSource = BufferSource | Response | PromiseLike<Response>;

export interface WebPluginRegistration {
  id: string;
  routePattern: string;
  loadWasm: () => PluginWasmSource | Promise<PluginWasmSource>;
}

export interface ResolvedWebPlugin {
  registration: WebPluginRegistration;
  route: ResolvedRoute;
}

export interface InstantiatedWebPlugin extends ResolvedWebPlugin {
  plugin: PluginInstance;
}

export class WebPluginRegistry {
  private readonly plugins = new Map<string, WebPluginRegistration>();

  register(plugin: WebPluginRegistration): this {
    this.plugins.set(plugin.routePattern, plugin);
    return this;
  }

  resolve(pathname: string, query: Record<string, string> = {}): ResolvedWebPlugin | undefined {
    const registration = this.plugins.get(pathname);
    if (!registration) return undefined;

    return {
      registration,
      route: {
        pattern: registration.routePattern,
        params: {},
        query,
      },
    };
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
