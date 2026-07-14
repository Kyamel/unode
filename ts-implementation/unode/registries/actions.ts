import type { ActionDefinition, ActionRunContext, ActionAvailabilityContext, ResolvedScreen, ResolvedRouteInfo, EntityRef } from '../api/contracts';
import type { ActionRef } from '../core/ast';
import type { HostApi } from '../api/host';
import type { PluginI18nApi } from '../core/runtime';

type LazyText = string | (() => string);

export type RegisteredAction = Omit<ActionDefinition<any>, 'title'> & {
  pluginId: string;
  title: LazyText;
};

export type ActionRunBaseContext = {
  screen?: ResolvedScreen;
  entity?: EntityRef;
  route?: ResolvedRouteInfo;
  host: HostApi;
  i18n?: PluginI18nApi;
};

export class ActionRegistry {
  private actions = new Map<string, RegisteredAction>();

  register(def: Omit<ActionDefinition<any>, 'title'> & { title: LazyText }, pluginId: string) {
    this.actions.set(def.id, { ...def, pluginId });
  }

  async run(action: ActionRef, ctx: ActionRunBaseContext) {
    const def = this.actions.get(action.type);
    if (!def) throw new Error(`Action not found: ${action.type}`);
    const fallbackI18n: PluginI18nApi = {
      t: (key, _values, fallback) => fallback ?? key,
      locale: () => 'en',
      register() {},
      translator() {
        return this;
      }
    };

    const availability: ActionAvailabilityContext = {
      screen: ctx.screen,
      entity: ctx.entity,
      route: ctx.route,
      host: ctx.host
    };

    const allowed = def.when ? await def.when(availability) : true;
    if (!allowed) throw new Error(`Action not available: ${action.type}`);

    const runCtx: ActionRunContext = {
      action,
      screen: ctx.screen,
      entity: ctx.entity,
      route: ctx.route,
      host: ctx.host,
      pluginId: def.pluginId,
      i18n: ctx.i18n ?? fallbackI18n
    };

    await def.run(runCtx);
  }
}
