// Solid adapter: host-component portal for the universal Unode renderer.
//
// The renderer itself lives in `unode-web-renderer` and produces DOM. Solid
// only fulfills `hostSlot(name)` holes: each mounted slot is a `render` root
// whose props live in a Solid store, so slot updates patch the component
// fine-grained instead of remounting it.

import type { Component } from "solid-js";
import { createComponent, render } from "solid-js/web";
import { createStore, reconcile } from "solid-js/store";
import type { HostPortalAdapter, HostSlotHandle, HostSlotRequest, OnAction } from "unode-web-renderer";

export type { OnAction };

/** Maps a `hostSlot(name)` to the host's Solid component. */
export type HostComponents = Record<string, Component<HostComponentProps>>;

/** Props a host component receives: the plugin's props plus a `dispatch`. */
export type HostComponentProps = Record<string, unknown> & { dispatch: OnAction };

export class SolidPortalAdapter implements HostPortalAdapter {
  constructor(public components: HostComponents) {}

  mount(container: Element, request: HostSlotRequest): HostSlotHandle {
    const component = this.components[request.name];
    if (!component) {
      return { update: () => {}, unmount: () => {} };
    }

    const [props, setProps] = createStore<HostComponentProps>({
      ...request.props,
      dispatch: request.dispatch,
    });
    const dispose = render(() => createComponent(component, props), container);

    return {
      update: (next) =>
        setProps(reconcile({ ...next, dispatch: request.dispatch } as HostComponentProps)),
      unmount: dispose,
    };
  }
}
