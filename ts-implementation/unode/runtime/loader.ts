import type { PluginDefinition as CorePluginDefinition } from '../core/runtime';

export type RuntimePluginModule = CorePluginDefinition;

export interface PluginLoader {
  load(url: string, options?: { hostId?: string }): Promise<RuntimePluginModule>;
}

export class EsmPluginLoader implements PluginLoader {
  async load(url: string, options?: { hostId?: string }): Promise<RuntimePluginModule> {
    const mod = await import(/* @vite-ignore */ url);
    const plugin = mod.default as RuntimePluginModule;
    if (!plugin?.manifest?.id) {
      throw new Error(`Plugin invalido em ${url}`);
    }
    if ('hostId' in plugin.manifest && plugin.manifest.hostId && options?.hostId && plugin.manifest.hostId !== options.hostId) {
      throw new Error(
        `Plugin ${plugin.manifest.id} e incompativel com host ${options.hostId} (hostId=${plugin.manifest.hostId})`
      );
    }
    return plugin;
  }
}
