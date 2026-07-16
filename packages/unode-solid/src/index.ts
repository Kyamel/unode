// Public surface of the unode Solid package.
//
// The renderer, recipe/builder API, and `hostSlot` primitive come straight
// from `unode-web-renderer` (re-exported below). Solid only contributes the
// `UnodeScreen` mount target and the host-component portal.
export * from "unode-web-renderer";

export { type HostComponentProps, type HostComponents, type OnAction, SolidPortalAdapter } from "./renderer";

export { UnodeScreen, type UnodeScreenProps } from "./UnodeScreen";
