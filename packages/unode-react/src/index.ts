// Public surface of the unode React package.
//
// The renderer, recipe/builder API, and `hostSlot` primitive come straight from
// `unode-renderer` (re-exported below). React only contributes the `<UnodeScreen>`
// mount target and the host-component portal.
export * from "unode-renderer";

export {
  type HostComponentProps,
  type HostComponents,
  type OnAction,
} from "./renderer";

export {
  UnodeScreen,
  type UnodeScreenProps,
} from "./UnodeScreen";
