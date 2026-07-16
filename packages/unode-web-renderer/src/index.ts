// Unode renderer: a single TypeScript renderer that turns compact IR into UI.
//
// The engine, the recipe/builder/override API, and the DOM backend all live here
// and are framework-free. Frameworks (React, Svelte, ...) are only mount targets
// and portal providers — they do not own renderer semantics.
//
//   import { defineRenderer, h, hostSlot } from "unode-web-renderer";
//
//   const renderer = defineRenderer()
//     .recipe("action", ({ label, intent, run }) =>
//       h("button", { class: `btn-${intent}`, onClick: run }, label))
//     .recipe("action", ({ label, intent, run }) =>
//       hostSlot("Button", { intent, onClick: run, children: label })) // host-native
//     .build();
//
//   renderer.mount(container, store, { onAction, portal });

// IR + prop helpers.
export {
  actionRefOf,
  booleanProp,
  literalOf,
  nodeKey,
  propOf,
  rendererPropsOf,
  stringProp,
  type ActionRef,
  type IrNode,
  type IrPatchOp,
  type IrScreen,
  type IrValue,
  type OnAction,
} from "./ir";

// Keyed screen store.
export {
  ScreenStore,
  type Listener,
  type RendererNodeSnapshot,
} from "./store";

// Virtual nodes + the host-slot primitive.
export {
  fragment,
  h,
  hostSlot,
  isVChildren,
  isVElement,
  isVHostSlot,
  normalizeChildren,
  type VChildren,
  type VElement,
  type VHostSlot,
  type VNode,
  type VProps,
} from "./vnode";

// Recipe context.
export {
  createRendererRecipeContext,
  type RecipeContext,
  type RendererRecipeContext,
  type RendererRecipeContextOptions,
} from "./context";

// Host-component portal contract.
export {
  type HostPortalAdapter,
  type HostSlotHandle,
  type HostSlotRequest,
} from "./portal";

// Renderer-definition API.
export {
  createRenderer,
  defaultFallback,
  defaultRecipes,
  defineRenderer,
  type NodeType,
  type Recipe,
  type Renderer,
  type RendererBuilder,
  type RendererSpec,
} from "./recipe";

// DOM backend types.
export {
  createDomRenderer,
  type MountOptions,
  type RendererHandle,
} from "./dom";
