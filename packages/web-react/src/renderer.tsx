// React adapter: mounts Unode IR with per-node subscriptions while letting apps
// provide their own semantic-node-to-design-system mapping.

import {
  createContext,
  useContext,
  useSyncExternalStore,
  type ReactNode as ReactNodeValue,
} from "react";

import {
  nodeKey,
  rendererPropsOf,
  type ActionRef,
  type IrNode,
  type OnAction,
  ScreenStore,
} from "unode-renderer";

export type { OnAction };

export interface ReactRendererNodeContext {
  node: IrNode;
  type: string;
  props: Record<string, unknown>;
  children: ReactNodeValue;
  childNodes: IrNode[];
  dispatch: OnAction;
  renderChildren(nodes?: IrNode[]): ReactNodeValue;
}

export type ReactNodeRenderer = (ctx: ReactRendererNodeContext) => ReactNodeValue;

export interface ReactRendererSpec {
  screen?: ReactNodeRenderer;
  nodes?: Record<string, ReactNodeRenderer>;
  fallback?: ReactNodeRenderer;
}

interface ResolvedReactRendererSpec {
  screen: ReactNodeRenderer;
  nodes: Record<string, ReactNodeRenderer>;
  fallback: ReactNodeRenderer;
}

interface RuntimeContext {
  store: ScreenStore;
  onAction: OnAction;
  spec: ResolvedReactRendererSpec;
}

export interface UnodeScreenProps {
  store: ScreenStore;
  onAction: OnAction;
}

export interface UnodeNodeProps {
  node: IrNode;
}

function classToken(value: unknown, fallback: string): string {
  return String(value ?? fallback);
}

function actionValue(value: unknown): ActionRef | undefined {
  if (value && typeof value === "object" && "t" in value) {
    return value as ActionRef;
  }
  return undefined;
}

const defaultFallback: ReactNodeRenderer = ({ children }) => children;

const defaultScreen: ReactNodeRenderer = ({ props, children }) => (
  <section className="unode-screen">
    {props.title != null && <h1 className="unode-title">{String(props.title)}</h1>}
    {children}
  </section>
);

export const defaultReactNodes: Record<string, ReactNodeRenderer> = {
  text({ props }) {
    const role = classToken(props.role, "body");
    return <p className={`unode-text unode-text--${role}`}>{String(props.content ?? "")}</p>;
  },

  actions({ children }) {
    return <div className="unode-actions">{children}</div>;
  },

  action({ props, dispatch }) {
    const action = actionValue(props.action);
    const intent = classToken(props.intent, "secondary");

    return (
      <button
        className={`unode-action unode-action--${intent}`}
        disabled={Boolean(props.disabled)}
        onClick={() => action && dispatch(action)}
      >
        {String(props.label ?? "")}
      </button>
    );
  },

  stack({ children }) {
    return <div className="unode-stack">{children}</div>;
  },

  inline({ children }) {
    return <div className="unode-inline">{children}</div>;
  },

  section({ children }) {
    return <section className="unode-section">{children}</section>;
  },
};

export const defaultReactRendererSpec: ResolvedReactRendererSpec = {
  screen: defaultScreen,
  nodes: defaultReactNodes,
  fallback: defaultFallback,
};

function resolveSpec(spec: ReactRendererSpec = {}): ResolvedReactRendererSpec {
  return {
    screen: spec.screen ?? defaultReactRendererSpec.screen,
    nodes: { ...defaultReactRendererSpec.nodes, ...(spec.nodes ?? {}) },
    fallback: spec.fallback ?? defaultReactRendererSpec.fallback,
  };
}

export function createReactRenderer(spec: ReactRendererSpec = {}) {
  const resolvedSpec = resolveSpec(spec);
  const Ctx = createContext<RuntimeContext | null>(null);

  function useRuntime(): RuntimeContext {
    const ctx = useContext(Ctx);
    if (!ctx) throw new Error("UnodeNode used outside <UnodeScreen>");
    return ctx;
  }

  function useNodeVersion(key: string, store: ScreenStore): void {
    useSyncExternalStore(
      (wake) => store.subscribe(key, wake),
      () => store.version(key),
      () => store.version(key),
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

  function renderChildren(nodes: IrNode[] = []): ReactNodeValue {
    return <Children nodes={nodes} />;
  }

  function UnodeScreen({ store, onAction }: UnodeScreenProps) {
    const props = rendererPropsOf(store.screen.p);
    const children = renderChildren(store.screen.c ?? []);

    return (
      <Ctx.Provider value={{ store, onAction, spec: resolvedSpec }}>
        {resolvedSpec.screen({
          node: store.screen as unknown as IrNode,
          type: "screen",
          props,
          children,
          childNodes: store.screen.c ?? [],
          dispatch: onAction,
          renderChildren,
        })}
      </Ctx.Provider>
    );
  }

  function UnodeNode({ node }: UnodeNodeProps) {
    const { store, onAction, spec } = useRuntime();
    const key = nodeKey(node);

    // Subscribe first (hooks must run unconditionally), then honor structural
    // replacements delivered by "rn" / "rc" patches.
    useNodeVersion(key, store);

    const snapshot = store.snapshotOf(node);
    if (snapshot.replacement && nodeKey(snapshot.replacement) !== snapshot.key) {
      return <UnodeNode node={snapshot.replacement} />;
    }

    const children = renderChildren(snapshot.children);
    const renderer = spec.nodes[snapshot.type] ?? spec.fallback;

    return (
      <>
        {renderer({
          node: snapshot.node,
          type: snapshot.type,
          props: snapshot.props,
          children,
          childNodes: snapshot.children,
          dispatch: onAction,
          renderChildren,
        })}
      </>
    );
  }

  return { UnodeScreen, UnodeNode };
}

const defaultRenderer = createReactRenderer();

export const UnodeScreen = defaultRenderer.UnodeScreen;
export const UnodeNode = defaultRenderer.UnodeNode;
