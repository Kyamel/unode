// IR types and prop helpers shared by every Unode renderer backend.
//
// These describe the compact IR produced by `unode-web-host` and the ergonomic
// prop normalization applied before recipes see a node. Nothing here knows about
// the DOM, React, Svelte, or any concrete UI toolkit.

export type ActionRef = {
  t: string;
  p?: Record<string, unknown>;
  originPluginId?: string;
  originContributionId?: string;
};

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
  /** Prop key for "sp" — the explicit name to overlay in `IrNode.p`. */
  f?: string;
  /** New value for "sp". */
  v?: unknown;
  /** New node for "rn". */
  n?: IrNode;
  /** New children for "rc". */
  c?: IrNode[];
}

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
 * Turns raw IR props into app-renderer props by unwrapping lowered literals
 * (`{ v: ... }`). Prop keys are already explicit and self-describing in the IR,
 * so no aliasing is needed.
 */
export function rendererPropsOf(props: Record<string, unknown>): Record<string, unknown> {
  const next: Record<string, unknown> = {};
  for (const [key, value] of Object.entries(props)) {
    next[key] = literalOf(value);
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

export function actionWithOrigin(
  action: ActionRef | undefined,
  props: Record<string, unknown>,
): ActionRef | undefined {
  if (!action) return undefined;

  const originPluginId = props["_originPluginId"];
  if (typeof originPluginId !== "string" || originPluginId.length === 0) {
    return action;
  }

  const originContributionId = props["_originContributionId"];
  return {
    ...action,
    originPluginId,
    ...(typeof originContributionId === "string" && originContributionId.length > 0
      ? { originContributionId }
      : {}),
  };
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
