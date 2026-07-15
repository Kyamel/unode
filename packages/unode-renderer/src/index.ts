// Shared renderer primitives produced from `unode-web-host` IR.
//
// Framework adapters should use this package for IR types, keyed patch storage,
// prop normalization, and fallback behavior instead of reimplementing runtime
// semantics in React, Svelte, Vue, etc.

export type ActionRef = { t: string; p?: Record<string, unknown> };

/** An action dispatch callback: receives the lowered action ref `{ t, p? }`. */
export type OnAction = (action: ActionRef) => void;

/** A lowered literal/binding value, e.g. `{ v: "Count: 3" }` or `{ b: "ui.x" }`. */
export type IrValue =
  | { v: unknown }
  | { b: string }
  | { pa: string }
  | unknown;

export interface IrNode {
  /** Node type tag: "text" | "stack" | "actions" | "action" | "screen" | ... */
  t: string;
  /** Props map. `_k` is the stable node key used to address patches. */
  p: Record<string, unknown>;
  /** Children. */
  c?: IrNode[];
}

export interface IrScreen {
  t: "screen";
  p: Record<string, unknown>;
  c?: IrNode[];
}

/** One patch op. `o` = "sp" (set prop) | "rn" (replace node) | "rc" (replace children). */
export interface IrPatchOp {
  o: "sp" | "rn" | "rc";
  /** Target node key. */
  k: string;
  /** Field code for "sp" (see FIELD_CODE_TO_PROP). */
  f?: string;
  /** New value for "sp". */
  v?: unknown;
  /** New node for "rn". */
  n?: IrNode;
  /** New children for "rc". */
  c?: IrNode[];
}

/**
 * Reactive field code (from `reactive_field_code` in ir.rs) to the prop key it
 * writes in `IrNode.p`. A `SetProp{f}` patch mutates `p[FIELD_CODE_TO_PROP[f]]`.
 */
export const FIELD_CODE_TO_PROP: Record<string, string> = {
  ti: "title",
  su: "subtitle",
  de: "desc",
  ct: "content",
  va: "value",
  lb: "label",
  lx: "labelExpanded",
  di: "dis",
  ph: "placeholder",
  ht: "help",
  ms: "message",
  pg: "progress",
  if: "if",
  bs: "state",
  co: "cont",
  mi: "items",
};

const PROP_ALIASES: Record<string, string> = {
  desc: "description",
  dis: "disabled",
  do: "action",
  fmt: "format",
  cur: "currencyCode",
  tr: "truncate",
  em: "emphasis",
  ar: "aspectRatio",
  exp: "expandable",
  cont: "continuation",
};

/** The `_k` prop is the node key. */
export function nodeKey(node: IrNode): string {
  return String(node.p["_k"] ?? "");
}

/** Unwrap a lowered literal value to a plain JS value for display. */
export function literalOf(value: unknown): unknown {
  if (value && typeof value === "object" && "v" in (value as object)) {
    return (value as { v: unknown }).v;
  }
  return value;
}

/**
 * Turns raw IR props into app-renderer props.
 *
 * The original IR keys remain available, but common compact keys are mirrored to
 * more ergonomic names such as `disabled` and `action`.
 */
export function rendererPropsOf(props: Record<string, unknown>): Record<string, unknown> {
  const next: Record<string, unknown> = {};

  for (const [key, value] of Object.entries(props)) {
    const resolved = literalOf(value);
    next[key] = resolved;

    const alias = PROP_ALIASES[key];
    if (alias && next[alias] === undefined) {
      next[alias] = resolved;
    }
  }

  return next;
}

export interface RendererNodeSnapshot {
  key: string;
  node: IrNode;
  type: string;
  props: Record<string, unknown>;
  children: IrNode[];
  replacement?: IrNode;
}

export type Listener = () => void;

export class ScreenStore {
  readonly screen: IrScreen;

  /** key -> latest props (structure props + applied SetProp overlays). */
  private props = new Map<string, Record<string, unknown>>();
  /** key -> replacement subtree (for "rn"). */
  private replaced = new Map<string, IrNode>();
  /** key -> replacement children (for "rc"). */
  private children = new Map<string, IrNode[]>();
  /** key -> version counter; changing it wakes that node's subscribers. */
  private versions = new Map<string, number>();
  private listeners = new Map<string, Set<Listener>>();

  constructor(screen: IrScreen) {
    this.screen = screen;
    this.index(screen as unknown as IrNode);
  }

  private index(node: IrNode): void {
    const key = nodeKey(node);
    if (key) {
      this.props.set(key, { ...node.p });
      if (!this.versions.has(key)) this.versions.set(key, 0);
    }
    for (const child of node.c ?? []) this.index(child);
  }

  propsOf(key: string): Record<string, unknown> {
    return this.props.get(key) ?? {};
  }

  replacementOf(key: string): IrNode | undefined {
    return this.replaced.get(key);
  }

  childrenOverrideOf(key: string): IrNode[] | undefined {
    return this.children.get(key);
  }

  version(key: string): number {
    return this.versions.get(key) ?? 0;
  }

  subscribe(key: string, listener: Listener): () => void {
    let set = this.listeners.get(key);
    if (!set) {
      set = new Set();
      this.listeners.set(key, set);
    }
    set.add(listener);
    return () => set.delete(listener);
  }

  applyPatches(ops: IrPatchOp[]): void {
    for (const op of ops) this.applyPatch(op);
  }

  snapshotOf(node: IrNode): RendererNodeSnapshot {
    const key = nodeKey(node);
    const rawProps = key ? this.propsOf(key) : node.p;
    const replacement = key ? this.replacementOf(key) : undefined;
    const activeNode = replacement && nodeKey(replacement) === key ? replacement : node;
    const children = (key ? this.childrenOverrideOf(key) : undefined) ?? activeNode.c ?? [];

    return {
      key,
      node: activeNode,
      type: activeNode.t,
      props: rendererPropsOf(rawProps),
      children,
      replacement,
    };
  }

  private applyPatch(op: IrPatchOp): void {
    switch (op.o) {
      case "sp": {
        const prop = op.f ? FIELD_CODE_TO_PROP[op.f] ?? op.f : undefined;
        if (!prop) return;
        const current = this.props.get(op.k) ?? {};
        this.props.set(op.k, { ...current, [prop]: op.v });
        break;
      }
      case "rn": {
        if (op.n) {
          this.replaced.set(op.k, op.n);
          this.index(op.n);
        }
        break;
      }
      case "rc": {
        if (op.c) {
          this.children.set(op.k, op.c);
          for (const child of op.c) this.index(child);
        }
        break;
      }
    }
    this.bump(op.k);
  }

  private bump(key: string): void {
    this.versions.set(key, this.version(key) + 1);
    this.listeners.get(key)?.forEach((listener) => listener());
  }
}
