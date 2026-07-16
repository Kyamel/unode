// Wires plugin wasm and host-session wasm together.
//
// Update loop:
//   action -> plugin.dispatch(action, snapshot) -> state writes
//          -> session.applyWrites(writes) -> IrPatchOp[]
//          -> store.applyPatches(ops) -> only affected node keys wake

import { ScreenStore, type OnAction } from "unode-web-renderer";
import type { HostCallHandler } from "./pluginHost";
import { PluginInstance } from "./pluginHost";
import { HostSession, type ResolvedRoute } from "./session";

export type { ResolvedRoute };

/**
 * Receives `state.set` calls the plugin makes through the `host_call` sandbox
 * boundary and buffers them until the host drains and applies them.
 */
export class StateWriteSink {
  private writes: Record<string, unknown> = {};

  readonly handler: HostCallHandler = (operation, params) => {
    if (operation === "state.set") {
      this.writes[String(params.path)] = (params as { value: unknown }).value;
      return { ok: true };
    }
    return { ok: false, error: `unhandled host call: ${operation}` };
  };

  /** Take and clear the buffered writes. */
  drain(): Record<string, unknown> {
    const taken = this.writes;
    this.writes = {};
    return taken;
  }
}

export interface WebRuntimeOptions {
  pluginId?: string;
  plugin: PluginInstance;
  session: HostSession;
  sink: StateWriteSink;
  route: ResolvedRoute;
  locale: string;
  actionTargetForPlugin?: (pluginId: string) => WebRuntimeActionTarget | undefined;
}

export interface WebRuntimeActionTarget {
  plugin: PluginInstance;
  sink: StateWriteSink;
  route: ResolvedRoute;
}

interface DispatchResponse {
  handled?: boolean;
  outcome?: { kind: string; to?: string };
}

export class WebRuntime {
  private store?: ScreenStore;

  constructor(private readonly opts: WebRuntimeOptions) {
    opts.session.setRoute(opts.route);
  }

  /** Run load + render + mount, returning the store the adapter renders. */
  mount(): ScreenStore {
    const { plugin, session, route, locale } = this.opts;
    const data = plugin.load({ route, locale });
    const screen = plugin.render({ route, data, locale });
    const ir = session.mount(screen, {});
    this.store = new ScreenStore(ir);
    // The mounted IR keeps bindings symbolic; resolve them once against the
    // seeded state so the first paint shows concrete values.
    this.store.applyPatches(session.initialPatches());
    return this.store;
  }

  /** The action handler handed to adapter renderers. */
  onAction: OnAction = (action) => {
    const { session, locale } = this.opts;
    if (!this.store) return;

    const currentPluginId = this.opts.pluginId;
    const target =
      action.originPluginId && action.originPluginId !== currentPluginId
        ? this.opts.actionTargetForPlugin?.(action.originPluginId)
        : undefined;

    if (action.originPluginId && action.originPluginId !== currentPluginId && !target) {
      console.warn(
        "[unode] refused to dispatch contributed action without plugin target",
        action.originPluginId,
      );
      return;
    }

    const { plugin, sink, route } = target ?? this.opts;
    const snapshot = session.stateSnapshot();
    const response = plugin.dispatch<DispatchResponse>({
      route,
      action: { type: action.t, ...(action.p ? { params: action.p } : {}) },
      stateSnapshot: snapshot,
      locale,
    });

    const writes = sink.drain();
    if (Object.keys(writes).length > 0) {
      const ops = session.applyWrites(writes);
      this.store.applyPatches(ops);
    }

    if (response?.outcome?.kind === "navigate") {
      // A real host would route here; the slice just surfaces the intent.
      console.info("[unode] plugin requested navigation to", response.outcome.to);
    }
  };
}
