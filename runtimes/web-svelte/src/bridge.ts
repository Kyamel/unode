// The orchestrator: wires the plugin wasm and the host-session wasm together
// and exposes `mount()` + `dispatch()` to the UI adapter.
//
// Update loop for a user interaction:
//   click → plugin.dispatch(action, snapshot) → stateWrites
//         → session.applyWrites(writes) → IrPatchOp[]
//         → store.applyPatches(ops) → only the affected node re-renders

import type { HostCallHandler } from "./pluginHost";
import { PluginInstance } from "./pluginHost";
import type { OnAction } from "./renderer";
import { HostSession } from "./session";
import { ScreenStore } from "./store";

export interface ResolvedRoute {
  pattern: string;
  params?: Record<string, string>;
  query?: Record<string, string>;
}

/**
 * Receives `state.set` calls the plugin makes through the `host_call` sandbox
 * boundary and buffers them until the host drains and applies them. The plugin
 * never returns UI state; it requests mutations through this capability, which
 * a real host could gate by permission.
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
  plugin: PluginInstance;
  session: HostSession;
  sink: StateWriteSink;
  route: ResolvedRoute;
  locale: string;
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

  /** The action handler handed to `<UnodeScreen onAction>`. */
  onAction: OnAction = (action) => {
    const { plugin, session, sink, route, locale } = this.opts;
    if (!this.store) return;

    const snapshot = session.stateSnapshot();
    // During dispatch the plugin makes `state.set` host calls, which the sink
    // buffers. Nothing is applied until dispatch returns.
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
