// Public surface of the unode React demo package.
//
// The renderer, recipe/builder API, and `hostSlot` primitive come straight from
// `unode-renderer` (re-exported below). React only contributes the `<UnodeScreen>`
// mount target and the host-component portal.
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

export {
  UnodeScreen,
  type HostComponentProps,
  type HostComponents,
  type OnAction,
  type UnodeScreenProps,
} from "./renderer";
