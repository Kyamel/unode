// The ergonomic context every recipe receives.
//
// It keeps the raw IR node and props reachable while promoting the common
// semantic fields (`label`, `intent`, `run()`, ...) to first-class properties so
// the happy-path recipe stays a one-liner.

import {
  actionWithOrigin,
  actionRefOf,
  booleanProp,
  propOf,
  stringProp,
  type ActionRef,
  type IrNode,
  type OnAction,
} from "./ir";
import type { VNode } from "./vnode";

export interface RendererRecipeContext<TChildren> {
  node: IrNode;
  type: string;
  props: Record<string, unknown>;
  children: TChildren;
  childNodes: IrNode[];
  dispatch: OnAction;
  renderChildren(nodes?: IrNode[]): TChildren;
  action?: ActionRef;
  content: string;
  label: string;
  title: string;
  role: string;
  intent: string;
  disabled: boolean;
  prop<T = unknown>(name: string, fallback?: T): T | undefined;
  text(fallback?: string): string;
  run(): void;
}

export interface RendererRecipeContextOptions<TChildren> {
  node: IrNode;
  type: string;
  props: Record<string, unknown>;
  children: TChildren;
  childNodes: IrNode[];
  dispatch: OnAction;
  renderChildren(nodes?: IrNode[]): TChildren;
}

/** The context a `VNode`-producing recipe receives. */
export type RecipeContext = RendererRecipeContext<VNode>;

/**
 * Builds the ergonomic context consumed by recipes. It keeps raw IR details
 * available while promoting common semantic fields to first-class properties
 * such as `label`, `intent`, and `run()`.
 */
export function createRendererRecipeContext<TChildren>({
  node,
  type,
  props,
  children,
  childNodes,
  dispatch,
  renderChildren,
}: RendererRecipeContextOptions<TChildren>): RendererRecipeContext<TChildren> {
  const action = actionWithOrigin(actionRefOf(props.action), props);
  const prop = <T = unknown>(name: string, fallback?: T): T | undefined =>
    propOf(props, name, fallback);
  const text = (fallback = ""): string =>
    stringProp(props, "content", stringProp(props, "label", fallback));

  return {
    node,
    type,
    props,
    children,
    childNodes,
    dispatch,
    renderChildren,
    action,
    content: stringProp(props, "content"),
    label: stringProp(props, "label"),
    title: stringProp(props, "title"),
    role: stringProp(props, "role", "body"),
    intent: stringProp(props, "intent", "secondary"),
    disabled: booleanProp(props, "disabled"),
    prop,
    text,
    run() {
      if (action) dispatch(action);
    },
  };
}
