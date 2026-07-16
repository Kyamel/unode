// React adapter: a thin mount target for the universal Unode renderer.
//
// The renderer itself (recipes, reactivity, patching) lives in `unode-renderer`
// and produces DOM. React's only jobs here are (1) hosting that DOM in a ref and
// (2) fulfilling `hostSlot(name)` holes with the host app's own React components
// through React portals. There is no React-specific renderer to maintain.

import {
  type ComponentType,
} from "react";

import {
  type HostPortalAdapter,
  type HostSlotHandle,
  type HostSlotRequest,
  type OnAction,

} from "unode-renderer";

export type { OnAction };

/** Maps a `hostSlot(name)` to the host's React component. */
export type HostComponents = Record<string, ComponentType<HostComponentProps>>;

/** Props a host component receives: the plugin's props plus a `dispatch`. */
export type HostComponentProps = Record<string, unknown> & { dispatch: OnAction };

interface PortalEntry {
  id: number;
  container: Element;
  name: string;
  props: Record<string, unknown>;
  dispatch: OnAction;
}

export class ReactPortalAdapter implements HostPortalAdapter {
  readonly entries = new Map<number, PortalEntry>();
  private seq = 0;
  private readonly listeners = new Set<() => void>();

  constructor(public components: HostComponents) {}

  subscribe(listener: () => void): () => void {
    this.listeners.add(listener);
    return () => this.listeners.delete(listener);
  }

  private emit(): void {
    this.listeners.forEach((listener) => listener());
  }

  mount(container: Element, request: HostSlotRequest): HostSlotHandle {
    const id = this.seq++;
    this.entries.set(id, {
      id,
      container,
      name: request.name,
      props: request.props,
      dispatch: request.dispatch,
    });
    this.emit();
    return {
      update: (props) => {
        const entry = this.entries.get(id);
        if (entry) {
          entry.props = props;
          this.emit();
        }
      },
      unmount: () => {
        this.entries.delete(id);
        this.emit();
      },
    };
  }
}

