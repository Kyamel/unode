// React adapter: a thin mount target for the universal Unode renderer.
//
// The renderer itself (recipes, reactivity, patching) lives in `unode-renderer`
// and produces DOM. React's only jobs here are (1) hosting that DOM in a ref and
// (2) fulfilling `hostSlot(name)` holes with the host app's own React components
// through React portals. There is no React-specific renderer to maintain.

import {
  useEffect,
  useRef,
  useState,
  type ComponentType,
} from "react";
import { createPortal } from "react-dom";

import {
  defineRenderer,
  type HostPortalAdapter,
  type HostSlotHandle,
  type HostSlotRequest,
  type OnAction,
  type Renderer,
  type ScreenStore,
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

class ReactPortalAdapter implements HostPortalAdapter {
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
 * `components` to back any `hostSlot` with native React components.
 */
export function UnodeScreen({
  store,
  onAction,
  renderer = defaultRenderer,
  components = {},
}: UnodeScreenProps) {
  const hostRef = useRef<HTMLDivElement | null>(null);
  const [, force] = useState(0);
  const adapterRef = useRef<ReactPortalAdapter | null>(null);
  if (!adapterRef.current) adapterRef.current = new ReactPortalAdapter(components);
  const adapter = adapterRef.current;
  adapter.components = components;

  useEffect(() => adapter.subscribe(() => force((n) => n + 1)), [adapter]);

  useEffect(() => {
    const el = hostRef.current;
    if (!el) return;
    const handle = renderer.mount(el, store, { onAction, portal: adapter });
    return () => handle.unmount();
  }, [store, renderer, onAction, adapter]);

  return (
    <div ref={hostRef} className="unode-root">
      {[...adapter.entries.values()].map((entry) => {
        const Component = adapter.components[entry.name];
        if (!Component) return null;
        return createPortal(
          <Component {...entry.props} dispatch={entry.dispatch} />,
          entry.container,
          String(entry.id),
        );
      })}
    </div>
  );
}
