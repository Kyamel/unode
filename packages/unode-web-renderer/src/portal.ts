// Host-component portal contract.
//
// This is the seam that keeps the renderer framework-agnostic while allowing
// deep, native integration. When a recipe emits `hostSlot(name, props)`, the
// DOM backend creates a placeholder element and hands it to the host's portal
// adapter. The React adapter turns that into a React portal rendering the host
// component registered under `name`; the Svelte adapter mounts a Svelte
// component; a plain-DOM app can supply its own factory. The renderer never
// depends on any of these frameworks — it only speaks this interface.

import type { OnAction } from "./ir";

export interface HostSlotRequest {
  /** Logical component name the plugin asked for, e.g. "Button". */
  name: string;
  /** Props forwarded to the host component. */
  props: Record<string, unknown>;
  /** Action dispatcher, so host components can raise plugin actions. */
  dispatch: OnAction;
}

export interface HostSlotHandle {
  /** Called when the same-named slot re-renders with new props. */
  update(props: Record<string, unknown>): void;
  /** Called when the slot is removed; tear down the host component. */
  unmount(): void;
}

export interface HostPortalAdapter {
  /**
   * Mount a host-native component into `container` (an empty placeholder owned
   * by the renderer). Return a handle the renderer uses to update/unmount it.
   */
  mount(container: Element, request: HostSlotRequest): HostSlotHandle;
}
