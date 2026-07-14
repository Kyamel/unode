// Wire types produced by `unode-web-host` (Rust). The renderer never invents
// these shapes — they are exactly `IrScreen` / `IrNode` / `IrPatchOp` lowered
// by `crates/unode/src/core/ir.rs`.

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

/** The `_k` prop is the node key. */
export function nodeKey(node: IrNode): string {
  return String(node.p["_k"] ?? "");
}

/**
 * Reactive field code (from `reactive_field_code` in ir.rs) → the prop key it
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

/** Unwrap a lowered literal value to a plain JS value for display. */
export function literalOf(value: unknown): unknown {
  if (value && typeof value === "object" && "v" in (value as object)) {
    return (value as { v: unknown }).v;
  }
  return value;
}
