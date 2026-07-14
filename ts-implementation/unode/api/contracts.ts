import type { HostApi, HostEvent } from './host';
import type { ActionRef } from '../core/ast';
import type { PluginI18nApi, StateStore } from '../core/runtime';
import type { CanonicalScreen, CanonicalUiNode } from '../core/normalize';

export type EntityRef = {
  kind: string;
  id: string;
};

export type ScreenLayout = 'default' | 'detail' | 'reader' | 'dashboard';

export type SlotDefinition = {
  accepts?: string[];
  order?: 'append' | 'prepend' | 'priority';
};

export type ScreenDefinition = {
  screenKind: string;
  title?: string;
  entity?: EntityRef;
  slots?: Record<string, SlotDefinition>;
  body: CanonicalScreen;
  state?: StateStore;
  layout?: ScreenLayout;
  meta?: Record<string, unknown>;
};

export type RouteDefinition<THostApi extends HostApi = HostApi> = {
  path: string;
  priority?: number;
  screenKind: string;
  match?: (path: string) => RouteMatch | null;
  render: (ctx: RouteRenderContext<THostApi>) => Promise<ScreenDefinition> | ScreenDefinition;
};

export type RouteMatch = {
  params: Record<string, string>;
};

export type RouteRenderContext<THostApi extends HostApi = HostApi> = {
  params: Record<string, string>;
  query: URLSearchParams;
  pluginId: string;
  requestId: string;
  signal?: AbortSignal;
  host: THostApi;
};

export type ScreenContributionContext<THostApi extends HostApi = HostApi> = {
  screenKind: string;
  entity?: EntityRef;
  routeParams: Record<string, string>;
  query: URLSearchParams;
  host: THostApi;
  pluginId: string;
};

export type SectionContribution<THostApi extends HostApi = HostApi> = {
  id: string;
  target: string;
  priority?: number;
  when?: (ctx: ScreenContributionContext<THostApi>) => boolean | Promise<boolean>;
  render: (ctx: ScreenContributionContext<THostApi>) => Promise<CanonicalUiNode> | CanonicalUiNode;
};

export type ActionAvailabilityContext<THostApi extends HostApi = HostApi> = {
  screen?: ResolvedScreen;
  entity?: EntityRef;
  route?: ResolvedRouteInfo;
  host: THostApi;
};

export type ActionRunContext<THostApi extends HostApi = HostApi> = {
  action: ActionRef;
  screen?: ResolvedScreen;
  entity?: EntityRef;
  route?: ResolvedRouteInfo;
  pluginId: string;
  host: THostApi;
  i18n: PluginI18nApi;
};

export type ActionDefinition<THostApi extends HostApi = HostApi> = {
  id: string;
  title: string;
  scope?: 'global' | 'screen' | 'entity';
  when?: (ctx: ActionAvailabilityContext<THostApi>) => boolean | Promise<boolean>;
  run: (ctx: ActionRunContext<THostApi>) => void | Promise<void>;
};

export type CommandContext<THostApi extends HostApi = HostApi> = {
  screenKind?: string;
  entity?: EntityRef;
  route?: ResolvedRouteInfo;
  host: THostApi;
  i18n?: PluginI18nApi;
};

export type CacheKey = string | [string, ...string[]];

export type CommandResult =
  | { ok: true; invalidates?: CacheKey[]; emits?: HostEvent[] }
  | { ok: false; error: { code: string; message: string } };

export type CommandDefinition<THostApi extends HostApi = HostApi> = {
  id: string;
  title: string;
  category?: string;
  keywords?: readonly string[];
  when?: (ctx: CommandContext<THostApi>) => boolean | Promise<boolean>;
  run: (ctx: CommandContext<THostApi>) => CommandResult | void | Promise<CommandResult | void>;
};

export type NavigationContext<THostApi extends HostApi = HostApi> = {
  screenKind?: string;
  entity?: EntityRef;
  route?: ResolvedRouteInfo;
  host: THostApi;
};

export type NavigationItem<THostApi extends HostApi = HostApi> = {
  id: string;
  label: string;
  shortLabel?: string;
  to: string;
  icon?: string;
  section?: string;
  priority?: number;
  when?: (ctx: NavigationContext<THostApi>) => boolean | Promise<boolean>;
};

export type ProviderContext<THostApi extends HostApi = HostApi> = {
  host: THostApi;
  pluginId: string;
};

export type ProviderDefinition<THostApi extends HostApi = HostApi, TInput = unknown, TOutput = unknown> = {
  id: string;
  capability: string;
  provide: (input: TInput, ctx: ProviderContext<THostApi>) => Promise<TOutput> | TOutput;
};

export type ResolvedRouteInfo = {
  pathname: string;
  params: Record<string, string>;
  screenKind: string;
  pluginId: string;
};

export type ResolvedScreen = {
  title?: string;
  screenKind: string;
  entity?: EntityRef;
  layout: ScreenLayout;
  body: CanonicalScreen;
  state: StateStore;
  meta?: Record<string, unknown>;
  slots: Record<string, CanonicalUiNode[]>;
  actions: Record<string, ActionDefinition[]>;
};
