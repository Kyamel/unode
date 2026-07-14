// React adapter: turns an `IrScreen` into components and re-applies `PatchOp`s
// with per-node granularity. This is the "hands" layer — it owns DOM shape,
// while `unode-web-host` (wasm) owns normalization, reactivity, and patch
// planning. No unode semantics are re-implemented here.

import { createContext, useContext, useSyncExternalStore } from "react";

import { IrNode, literalOf, nodeKey } from "./ir";
import { ScreenStore } from "./store";

/** An action dispatch callback: receives the lowered action ref `{ t, p? }`. */
export type OnAction = (action: { t: string; p?: Record<string, unknown> }) => void;

interface RuntimeContext {
  store: ScreenStore;
  onAction: OnAction;
}

const Ctx = createContext<RuntimeContext | null>(null);

function useRuntime(): RuntimeContext {
  const ctx = useContext(Ctx);
  if (!ctx) throw new Error("UnodeNode used outside <UnodeScreen>");
  return ctx;
}

/** Subscribe a component to a single node key. Only patches to that key wake it. */
function useLiveProps(key: string): Record<string, unknown> {
  const { store } = useRuntime();
  useSyncExternalStore(
    (cb) => store.subscribe(key, cb),
    () => store.version(key),
    () => store.version(key),
  );
  return store.propsOf(key);
}

export function UnodeScreen(props: { store: ScreenStore; onAction: OnAction }) {
  const { store, onAction } = props;
  const title = literalOf((store.screen.p as Record<string, unknown>)["title"]);
  return (
    <Ctx.Provider value={{ store, onAction }}>
      <section className="unode-screen">
        {title != null && <h1 className="unode-title">{String(title)}</h1>}
        <Children nodes={store.screen.c ?? []} />
      </section>
    </Ctx.Provider>
  );
}

function Children({ nodes }: { nodes: IrNode[] }) {
  return (
    <>
      {nodes.map((node) => (
        <UnodeNode key={nodeKey(node) || undefined} node={node} />
      ))}
    </>
  );
}

export function UnodeNode({ node }: { node: IrNode }) {
  const { store } = useRuntime();
  const key = nodeKey(node);

  // Subscribe first (hooks must run unconditionally), then honor structural
  // replacements delivered by "rn" / "rc" patches.
  const liveProps = useLiveProps(key);
  const replacement = store.replacementOf(key);
  if (replacement && nodeKey(replacement) !== key) {
    return <UnodeNode node={replacement} />;
  }
  const childrenOverride = store.childrenOverrideOf(key);
  const children = childrenOverride ?? node.c ?? [];
  const p = key ? liveProps : node.p;

  switch (node.t) {
    case "text":
      return <TextView p={p} />;
    case "actions":
      return (
        <div className="unode-actions">
          <Children nodes={children} />
        </div>
      );
    case "action":
      return <ActionView p={p} />;
    case "stack":
      return (
        <div className="unode-stack">
          <Children nodes={children} />
        </div>
      );
    case "inline":
      return (
        <div className="unode-inline">
          <Children nodes={children} />
        </div>
      );
    case "section":
      return (
        <section className="unode-section">
          <Children nodes={children} />
        </section>
      );
    default:
      // Unknown container: still render its children so the tree survives.
      return children.length ? <Children nodes={children} /> : null;
  }
}

function TextView({ p }: { p: Record<string, unknown> }) {
  const content = literalOf(p["content"]);
  const role = String(p["role"] ?? "body");
  return <p className={`unode-text unode-text--${role}`}>{String(content ?? "")}</p>;
}

function ActionView({ p }: { p: Record<string, unknown> }) {
  const { onAction } = useRuntime();
  const label = literalOf(p["label"]);
  const intent = String(p["intent"] ?? "secondary");
  const action = p["do"] as { t: string; p?: Record<string, unknown> } | undefined;
  return (
    <button
      className={`unode-action unode-action--${intent}`}
      disabled={Boolean(literalOf(p["dis"]))}
      onClick={() => action && onAction(action)}
    >
      {String(label ?? "")}
    </button>
  );
}
