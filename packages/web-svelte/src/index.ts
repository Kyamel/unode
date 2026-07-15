// Public surface of the unode Svelte demo package.
//
// The renderer, recipe/builder API, and `hostSlot` primitive come from
// `unode-renderer` (re-exported below). Svelte only contributes the
// `<UnodeScreen>` mount target and the host-component portal.
export * from "unode-renderer";

export { PluginInstance, type HostCallHandler } from "unode-core";
export {
  WebPluginRegistry,
  type InstantiatedWebPlugin,
  type PluginWasmSource,
  type ResolvedWebPlugin,
  type WebPluginRegistration,
} from "unode-core";
export { HostSession, type WebHostModule } from "unode-core";
export { WebRuntime, StateWriteSink, type ResolvedRoute, type WebRuntimeOptions } from "unode-core";

export { createSveltePortal, type HostComponents, type OnAction } from "./renderer.svelte";
export { default as UnodeScreen } from "./UnodeScreen.svelte";
