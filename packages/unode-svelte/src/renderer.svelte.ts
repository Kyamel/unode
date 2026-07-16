// Svelte adapter: portal glue for the universal Unode renderer.
//
// As with React, the renderer itself lives in `unode-renderer` and produces DOM.
// Svelte only fulfills `hostSlot(name)` holes with native Svelte components.
//
// This is a `.svelte.ts` rune module so the portal can hand each mounted
// component a `$state` props object: updates mutate that object, so Svelte
// patches the component in place (preserving its state) instead of remounting —
// matching the React adapter's behavior.

import { mount, unmount, type Component } from "svelte";
import type {
  HostPortalAdapter,
  HostSlotHandle,
  HostSlotRequest,
} from "unode-renderer";

export type { OnAction } from "unode-renderer";

/** Maps a `hostSlot(name)` to the host's Svelte component. */
// eslint-disable-next-line @typescript-eslint/no-explicit-any
export type HostComponents = Record<string, Component<any>>;

/**
 * Builds a portal adapter that mounts host Svelte components into the renderer's
 * placeholder elements, with reactive prop updates.
 */
export function createSveltePortal(components: HostComponents): HostPortalAdapter {
  return {
    mount(container: Element, request: HostSlotRequest): HostSlotHandle {
      const Component = components[request.name];
      if (!Component) {
        const children = request.props.children;
        container.textContent =
          typeof children === "string" || typeof children === "number" ? String(children) : "";
        return { update: () => {}, unmount: () => {} };
      }

      // Reactive props object; mutating it updates the component in place.
      const state: Record<string, unknown> = $state({ ...request.props, dispatch: request.dispatch });
      const instance = mount(Component, { target: container, props: state });

      return {
        update(props) {
          for (const key of Object.keys(state)) {
            if (key !== "dispatch" && !(key in props)) delete state[key];
          }
          Object.assign(state, props);
        },
        unmount() {
          unmount(instance);
        },
      };
    },
  };
}
