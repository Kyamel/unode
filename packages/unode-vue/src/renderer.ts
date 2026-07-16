// Vue adapter: host-component portal for the universal Unode renderer.
//
// As with React/Svelte, the renderer itself lives in `unode-web-renderer` and
// produces DOM. Vue only fulfills `hostSlot(name)` holes: each mounted slot
// becomes a tiny Vue app whose props object is shallow-reactive, so slot
// updates patch the component in place instead of remounting it.

import { type App, type Component, createApp, h as vueH, shallowReactive } from "vue";
import type { HostPortalAdapter, HostSlotHandle, HostSlotRequest, OnAction } from "unode-web-renderer";

export type { OnAction };

/** Maps a `hostSlot(name)` to the host's Vue component. */
export type HostComponents = Record<string, Component>;

/** Props a host component receives: the plugin's props plus a `dispatch`. */
export type HostComponentProps = Record<string, unknown> & { dispatch: OnAction };

export class VuePortalAdapter implements HostPortalAdapter {
  constructor(public components: HostComponents) {}

  mount(container: Element, request: HostSlotRequest): HostSlotHandle {
    const component = this.components[request.name];
    if (!component) {
      return { update: () => {}, unmount: () => {} };
    }

    const props = shallowReactive<Record<string, unknown>>({
      ...request.props,
      dispatch: request.dispatch,
    });
    // Spreading the reactive object inside the render function tracks every
    // key, so `update` re-renders the slot with fresh props.
    const app: App = createApp({ render: () => vueH(component, { ...props }) });
    app.mount(container);

    return {
      update: (next) => {
        for (const key of Object.keys(props)) {
          if (key !== "dispatch" && !(key in next)) delete props[key];
        }
        Object.assign(props, next);
      },
      unmount: () => app.unmount(),
    };
  }
}
