/* eslint-disable react-hooks/refs --
 * The portal adapter is a deliberate external-store pattern: a lazily
 * ref-initialized registry whose entries are rendered as portals, kept in
 * sync via subscribe + forced re-render. */
import { defineRenderer, OnAction, Renderer, ScreenStore } from "unode-web-renderer";
import { HostComponents } from "./renderer";
import { useEffect, useRef, useState } from "react";
import { createPortal } from "react-dom";
import { ReactPortalAdapter } from "./renderer";

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

export default UnodeScreen;