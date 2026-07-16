// Public surface of the unode Svelte package.
//
// The renderer, recipe/builder API, and `hostSlot` primitive come from
// `unode-renderer` (re-exported below). Svelte only contributes the
// `<UnodeScreen>` mount target and the host-component portal.
export * from "unode-renderer";

export { createSveltePortal, type HostComponents, type OnAction } from "./renderer.svelte";
export { default as UnodeScreen } from "./UnodeScreen.svelte";
