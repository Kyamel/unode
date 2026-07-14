import type { HostApi } from '../api/host';
import type {
  ResolvedScreen,
  ScreenDefinition,
  ScreenContributionContext,
  ResolvedRouteInfo
} from '../api/contracts';
import type { PluginLoader } from './loader';
import type { RuntimePluginModule } from './loader';
import { createPluginSetupContext } from './context';
import { RouteRegistry } from '../registries/routes';
import { ScreenRegistry } from '../registries/screens';
import { ActionRegistry } from '../registries/actions';
import { CommandRegistry } from '../registries/commands';
import { NavigationRegistry } from '../registries/navigation';
import { ProviderRegistry } from '../registries/providers';
import { UNODE_CORE_API_VERSION } from '../core/runtime';
import type { PluginManifest } from '../core/runtime';
import { MemoryStateStore } from '../core/state';

export type RuntimeRegistries = {
  routes: RouteRegistry;
  screens: ScreenRegistry;
  actions: ActionRegistry;
  commands: CommandRegistry;
  navigation: NavigationRegistry;
  providers: ProviderRegistry;
};

export class PluginRuntime {
  readonly registries: RuntimeRegistries;
  private loader?: PluginLoader;

  private readonly hostId?: string;
  private readonly guardHostApi?: (plugin: PluginManifest, api: HostApi) => HostApi;

  constructor(
    options?: {
      loader?: PluginLoader;
      registries?: Partial<RuntimeRegistries>;
      hostId?: string;
      guardHostApi?: (plugin: PluginManifest, api: HostApi) => HostApi;
    }
  ) {
    const loader = options?.loader;
    const registries = options?.registries;
    this.hostId = options?.hostId;
    this.guardHostApi = options?.guardHostApi;
    this.registries = {
      routes: registries?.routes ?? new RouteRegistry(),
      screens: registries?.screens ?? new ScreenRegistry(),
      actions: registries?.actions ?? new ActionRegistry(),
      commands: registries?.commands ?? new CommandRegistry(),
      navigation: registries?.navigation ?? new NavigationRegistry(),
      providers: registries?.providers ?? new ProviderRegistry()
    };
    this.loader = loader;
  }

  async activate(urls: string[], hostApi: HostApi) {
    if (!this.loader) throw new Error('Plugin loader not configured');
    for (const url of urls) {
      const plugin = await this.loader.load(url, { hostId: this.hostId });
      await this.activateModule(plugin, hostApi);
    }
  }

  async activateModules(modules: RuntimePluginModule[], hostApi: HostApi) {
    for (const plugin of modules) {
      await this.activateModule(plugin, hostApi);
    }
  }

  async resolveScreen(pathname: string, query: URLSearchParams, hostApi: HostApi): Promise<ResolvedScreen | null> {
    const routeMatch = this.registries.routes.resolve(pathname);
    if (!routeMatch) return null;

    const requestId =
      typeof crypto !== 'undefined' && 'randomUUID' in crypto
        ? crypto.randomUUID()
        : `req_${Date.now()}_${Math.random().toString(16).slice(2)}`;
    const routeInfo: ResolvedRouteInfo = {
      pathname,
      params: routeMatch.params,
      screenKind: routeMatch.route.screenKind,
      pluginId: routeMatch.pluginId
    };

    const screen = await routeMatch.route.render({
      params: routeMatch.params,
      query,
      pluginId: routeMatch.pluginId,
      requestId,
      host: hostApi
    });

    return await this.composeScreen(screen, routeInfo, query, hostApi, routeMatch.pluginId);
  }

  async composeScreen(
    screen: ScreenDefinition,
    routeInfo: ResolvedRouteInfo,
    query: URLSearchParams,
    hostApi: HostApi,
    pluginId: string
  ): Promise<ResolvedScreen> {
    const slotKeys = Object.keys(screen.slots ?? {});
    const slots: Record<string, ResolvedScreen['slots'][string]> = {};
    for (const key of slotKeys) slots[key] = [];

    const ctx: ScreenContributionContext = {
      screenKind: screen.screenKind,
      entity: screen.entity,
      routeParams: routeInfo.params,
      query,
      host: hostApi,
      pluginId
    };

    const sections = await this.registries.screens.resolveSections(ctx, slotKeys);
    for (const section of sections) {
      slots[section.target]?.push(section.node);
    }

    return {
      title: screen.title,
      screenKind: screen.screenKind,
      entity: screen.entity,
      layout: screen.layout ?? 'default',
      body: screen.body,
      state: screen.state ?? new MemoryStateStore(),
      meta: screen.meta,
      slots,
      actions: {}
    };
  }

  private async activateModule(plugin: RuntimePluginModule, hostApi: HostApi) {
    this.assertCoreCompatible(plugin.manifest);
    const ctx = createPluginSetupContext(plugin, this.registries, hostApi, this.guardHostApi);
    await plugin.activate(ctx);
  }

  private assertCoreCompatible(manifest: PluginManifest) {
    if (manifest.apiVersion !== UNODE_CORE_API_VERSION) {
      throw new Error(`Core plugin API mismatch for ${manifest.id}: ${manifest.apiVersion}`);
    }
    if ('hostId' in manifest && manifest.hostId && this.hostId && manifest.hostId !== this.hostId) {
      throw new Error(`Plugin hostId mismatch for ${manifest.id}: ${manifest.hostId}`);
    }
  }
}
