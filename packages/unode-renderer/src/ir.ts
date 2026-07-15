// IR types and prop helpers shared by every Unode renderer backend.
//
// These describe the compact IR produced by `unode-web-host` and the ergonomic
// prop normalization applied before recipes see a node. Nothing here knows about
// the DOM, React, Svelte, or any concrete UI toolkit.

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

/** Returns a typed action ref when a normalized prop contains one. */
export function actionRefOf(value: unknown): ActionRef | undefined {
  if (value && typeof value === "object" && "t" in value) {
    return value as ActionRef;
  }
  return undefined;
}

/** Reads a renderer prop with a fallback while keeping call sites compact. */
export function propOf<T = unknown>(
  props: Record<string, unknown>,
  name: string,
  fallback?: T,
): T | undefined {
  const value = props[name];
  return (value === undefined ? fallback : value) as T | undefined;
}

export function stringProp(
  props: Record<string, unknown>,
  name: string,
  fallback = "",
): string {
  return String(propOf(props, name, fallback) ?? fallback);
}

export function booleanProp(
  props: Record<string, unknown>,
  name: string,
  fallback = false,
): boolean {
  return Boolean(propOf(props, name, fallback));
}
