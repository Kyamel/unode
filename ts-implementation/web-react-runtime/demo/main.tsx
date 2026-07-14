// Demo entry: mounts the `web-counter` plugin through the full web slice.
//
// The two wasm artifacts are produced by `./build.sh` (see README):
//   - ../pkg/unode_web_host.js         (wasm-bindgen glue for the Rust core)
//   - web_counter_plugin.wasm          (the plugin, raw C ABI)

import { useEffect, useState } from "react";
import { createRoot } from "react-dom/client";

import {
  HostSession,
  PluginInstance,
  ScreenStore,
  StateWriteSink,
  UnodeScreen,
  WebRuntime,
} from "../src";

// wasm-bindgen generated module + its wasm asset (built into ../pkg).
import * as webHostModule from "../pkg/unode_web_host.js";
import webHostWasmUrl from "../pkg/unode_web_host_bg.wasm?url";
// The plugin wasm, served as a URL by Vite.
import pluginWasmUrl from "./web_counter_plugin.wasm?url";

const ROUTE = { pattern: "/plugins/web-counter", params: {}, query: {} };

function App() {
  const [state, setState] = useState<{ store: ScreenStore; runtime: WebRuntime } | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    (async () => {
      try {
        const session = await HostSession.create(
          webHostModule as never,
          webHostWasmUrl,
          "en",
        );
        // The plugin writes state through the `host_call` sandbox boundary; the
        // sink services `state.set`. A real app would extend the handler with
        // `navigation.navigate`, domain APIs, etc.
        const sink = new StateWriteSink();
        const plugin = await PluginInstance.instantiate(await fetch(pluginWasmUrl), sink.handler);

        const runtime = new WebRuntime({ plugin, session, sink, route: ROUTE, locale: "en" });
        const store = runtime.mount();
        setState({ store, runtime });
      } catch (e) {
        setError(String(e));
      }
    })();
  }, []);

  if (error) return <pre className="unode-error">{error}</pre>;
  if (!state) return <p>Loading unode runtime…</p>;

  return <UnodeScreen store={state.store} onAction={state.runtime.onAction} />;
}

createRoot(document.getElementById("root")!).render(<App />);
