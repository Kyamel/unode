// Framework-neutral virtual nodes.
//
// Recipes return `VNode`s, never real DOM or framework elements. This is what
// lets a single TypeScript recipe language drive the DOM backend directly while
// still deferring to host-native components through `hostSlot`.

import type { IrNode } from "./ir";

export type VProps = Record<string, unknown>;

/** A plain element, e.g. `h("button", { onClick }, "Save")`. */
export interface VElement {
  readonly $: "el";
  tag: string;
  props: VProps;
  children: VNode[];
  /** Identity for keyed reconciliation (from `props.key`). */
  key?: string | number;
}

/**
 * A hole filled by a host-native component. The renderer creates a placeholder
 * element and asks the framework adapter's portal to mount `name` into it with
 * `props`. This is the deep-integration primitive: a plugin node can render as
 * the host app's own design-system component.
 */
export interface VHostSlot {
  readonly $: "host";
  name: string;
  props: VProps;
}

/**
 * The mount point for a node's Unode children. Each child becomes its own keyed,
 * independently reactive unit — this is not a static element subtree.
 */
export interface VChildren {
  readonly $: "children";
  nodes: IrNode[];
}

export type VNode =
  | VElement
  | VHostSlot
  | VChildren
  | string
  | number
  | boolean
  | null
  | undefined
  | VNode[];

/** Creates an element VNode. `props.children` is ignored; use varargs. */
export function h(tag: string, props?: VProps | null, ...children: VNode[]): VElement {
  const key = props?.key;
  return {
    $: "el",
    tag,
    props: props ?? {},
    children,
    key: typeof key === "string" || typeof key === "number" ? key : undefined,
  };
}

/** Fragment helper: groups children without a wrapper element. */
export function fragment(...children: VNode[]): VNode[] {
  return children;
}

/**
 * Declares a hole to be filled by a host-native component named `name`. The
 * framework adapter's portal maps the name to a real component (e.g. the host's
 * `<Button>`); a plain DOM mount without a portal falls back gracefully.
 */
export function hostSlot(name: string, props: VProps = {}): VHostSlot {
  return { $: "host", name, props };
}

export function isVElement(v: VNode): v is VElement {
  return typeof v === "object" && v !== null && !Array.isArray(v) && (v as { $?: string }).$ === "el";
}

export function isVHostSlot(v: VNode): v is VHostSlot {
  return typeof v === "object" && v !== null && !Array.isArray(v) && (v as { $?: string }).$ === "host";
}

export function isVChildren(v: VNode): v is VChildren {
  return typeof v === "object" && v !== null && !Array.isArray(v) && (v as { $?: string }).$ === "children";
}

/**
 * Flattens a recipe's output into a positional list. Arrays are spliced in;
 * `null`/`undefined`/booleans are kept as placeholder slots so that conditional
 * branches (`cond ? h(...) : null`) stay position-stable across re-renders.
 */
export function normalizeChildren(v: VNode): VNode[] {
  const out: VNode[] = [];
  flatten(v, out);
  return out;
}

function flatten(v: VNode, out: VNode[]): void {
  if (Array.isArray(v)) {
    for (const child of v) flatten(child, out);
    return;
  }
  out.push(v);
}
