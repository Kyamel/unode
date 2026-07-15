// Demo entry: mounts the `web-counter` plugin through the full web slice.
//
// The two wasm artifacts are produced by `./build.sh` (see README):
//   - ../pkg/unode_web_host.js         (wasm-bindgen glue for the Rust core)
//   - web_counter_plugin.wasm          (the plugin, raw C ABI)

import { useEffect, useState } from "react";
import { createRoot } from "react-dom/client";

import {
  defineRenderer,
  h,
  HostSession,
  hostSlot,
  type HostComponentProps,
  ScreenStore,
  StateWriteSink,
  UnodeScreen,
  WebPluginRegistry,
  WebRuntime,
} from "../src";

// A host-owned React component. The plugin never imports it — it only asks for a
// semantic "Button" via `hostSlot`, and this app decides what that looks like.
function Button({ children, intent, dispatch, action }: HostComponentProps) {
  return (
    <button
      className={`ds-button ds-button--${String(intent ?? "secondary")}`}
      onClick={() => action && dispatch(action as { t: string })}
    >
      {String(children ?? "")}
    </button>
  );
}

// Recipes are written once, in the universal TS language. Here `action` nodes
// render as the host's native <Button> through a host slot; everything else
// falls back to the built-in DOM recipes.
const renderer = defineRenderer()
  .recipe("action", ({ label, prop, action }) =>
    hostSlot("Button", { children: label, intent: prop("intent"), action }),
  )
  .recipe("section", ({ title, children }) =>
    h("section", { class: "ds-card" }, title ? h("h2", {}, title) : null, children),
  )
  .build();

// wasm-bindgen generated module + its wasm asset (built into ../pkg).
import * as webHostModule from "../pkg/unode_web_host.js";
import webHostWasmUrl from "../pkg/unode_web_host_bg.wasm?url";
// The plugin wasm, served as a URL by Vite.
import pluginWasmUrl from "./web_counter_plugin.wasm?url";

const PLUGIN_ROUTE_PATTERN = "/plugins/web-counter";
const pluginRegistry = new WebPluginRegistry().register({
  id: "dev.unode.web-counter",
  routePattern: PLUGIN_ROUTE_PATTERN,
  loadWasm: () => fetch(pluginWasmUrl),
});

function routeTargetForCurrentLocation() {
  const url = new URL(window.location.href);

  if (url.pathname === "/") {
    window.history.replaceState(null, "", `${PLUGIN_ROUTE_PATTERN}${url.search}${url.hash}`);
    url.pathname = PLUGIN_ROUTE_PATTERN;
  }

  return {
    pathname: url.pathname,
    query: Object.fromEntries(url.searchParams.entries()),
  };
}

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
        const pluginTarget = routeTargetForCurrentLocation();
        const { plugin, route } = await pluginRegistry.instantiateForPath(
          pluginTarget.pathname,
          pluginTarget.query,
          sink.handler,
        );

        const runtime = new WebRuntime({
          plugin,
          session,
          sink,
          route,
          locale: "en",
        });
        const store = runtime.mount();
        setState({ store, runtime });
      } catch (e) {
        setError(String(e));
      }
    })();
  }, []);

  if (error) return <pre className="unode-error">{error}</pre>;
  if (!state) return <p>Loading unode runtime…</p>;

  return (
    <UnodeScreen
      store={state.store}
      onAction={state.runtime.onAction}
      renderer={renderer}
      components={{ Button }}
    />
  );
}

createRoot(document.getElementById("root")!).render(<App />);
