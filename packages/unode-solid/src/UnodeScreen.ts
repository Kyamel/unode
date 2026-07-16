// The Solid mount target: hosts the DOM produced by the universal renderer
// and wires the host-component portal. Solid components may return real DOM
// nodes, so no JSX is needed here.

import { onCleanup, onMount } from "solid-js";
import {
  type OnAction,
  type Renderer,
  type RendererHandle,
  ScreenStore,
  defineRenderer,
} from "unode-web-renderer";

import { type HostComponents, SolidPortalAdapter } from "./renderer";

const defaultRenderer = defineRenderer().build();

export interface UnodeScreenProps {
  store: ScreenStore;
  onAction?: OnAction;
  /** A renderer from `defineRenderer()`. Defaults to the built-in recipes. */
  renderer?: Renderer;
  /** Host components that fulfill `hostSlot(name)` holes. */
  components?: HostComponents;
}

/**
 * Mounts a Unode screen. Pass a `renderer` to customize recipes and
 * `components` to back any `hostSlot` with native Solid components.
 */
export function UnodeScreen(props: UnodeScreenProps): HTMLElement {
  const host = document.createElement("div");
  host.className = "unode-root";

  let handle: RendererHandle | undefined;
  onMount(() => {
    const adapter = new SolidPortalAdapter(props.components ?? {});
    handle = (props.renderer ?? defaultRenderer).mount(host, props.store, {
      onAction: props.onAction,
      portal: adapter,
    });
  });
  onCleanup(() => handle?.unmount());

  return host;
}
