// Shared bootstrap: instantiate the wasm host core + the counter plugin and
// hand back a mounted runtime. Framework-independent on purpose.
import { HostSession, StateWriteSink, WebPluginRegistry, WebRuntime } from "unode-web-core";

import * as webHostModule from "../pkg/unode_web_host.js";
import webHostWasmUrl from "../pkg/unode_web_host_bg.wasm?url";
import pluginWasmUrl from "./web_counter_plugin.wasm?url";

const PLUGIN_ROUTE_PATTERN = "/plugins/counter";
const pluginRegistry = new WebPluginRegistry().register({
  id: "dev.unode.counter",
  routePattern: PLUGIN_ROUTE_PATTERN,
  loadWasm: () => fetch(pluginWasmUrl),
});

function routeTargetForCurrentLocation() {
  const url = new URL(window.location.href);
  if (url.pathname === "/") {
    window.history.replaceState(null, "", `${PLUGIN_ROUTE_PATTERN}${url.search}${url.hash}`);
    url.pathname = PLUGIN_ROUTE_PATTERN;
  }
  return { pathname: url.pathname, query: Object.fromEntries(url.searchParams.entries()) };
}

export async function bootRuntime(): Promise<WebRuntime> {
  const session = await HostSession.create(webHostModule as never, webHostWasmUrl, "en");
  const sink = new StateWriteSink();
  const target = routeTargetForCurrentLocation();
  const { registration, plugin, route } = await pluginRegistry.instantiateForPath(
    target.pathname,
    target.query,
    sink.handler,
  );
  return new WebRuntime({ pluginId: registration.id, plugin, session, sink, route, locale: "en" });
}
