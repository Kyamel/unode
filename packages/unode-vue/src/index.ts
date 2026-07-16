// Public surface of the unode Vue package.
//
// The renderer, recipe/builder API, and `hostSlot` primitive come straight
// from `unode-web-renderer` (re-exported below). Vue only contributes the
// `<UnodeScreen>` mount target and the host-component portal.
export * from "unode-web-renderer";

export { type HostComponentProps, type HostComponents, type OnAction, VuePortalAdapter } from "./renderer";

export { UnodeScreen } from "./UnodeScreen";
