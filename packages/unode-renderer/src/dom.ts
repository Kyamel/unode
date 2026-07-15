// The DOM backend: the one renderer.
//
// It walks the IR, subscribes each node to its own store key (keyed reactivity),
// runs the resolved recipe to get a `VNode` tree, and reconciles that tree into
// real DOM. `hostSlot` VNodes are handed to the optional portal adapter so a
// plugin node can render as a host-native component.
//
// Two reconciliation levels keep this small:
//   - NodeInstance: one per IrNode. Re-runs its recipe when *its* key bumps and
//     patches only its own element subtree. Children are separate instances.
//   - ChildrenRegion: a keyed list of child NodeInstances between two anchors.

import { nodeKey, type IrNode, type OnAction } from "./ir";
import { createRendererRecipeContext } from "./context";
import type { ScreenStore } from "./store";
import type { Recipe, RendererSpec } from "./recipe";
import type { HostPortalAdapter, HostSlotHandle } from "./portal";
import {
  isVChildren,
  isVElement,
  isVHostSlot,
  normalizeChildren,
  type VChildren,
  type VElement,
  type VNode,
  type VProps,
} from "./vnode";

export interface MountOptions {
  /** Dispatches lowered plugin actions raised by recipes/host components. */
  onAction?: OnAction;
  /** Resolves `hostSlot(name)` VNodes to host-native components. */
  portal?: HostPortalAdapter;
}

export interface RendererHandle {
  unmount(): void;
}

interface Env {
  store: ScreenStore;
  resolve(type: string): Recipe;
  onAction: OnAction;
  portal?: HostPortalAdapter;
}

/** Builds a DOM renderer bound to a resolved recipe spec. */
export function createDomRenderer(spec: RendererSpec) {
  const resolve = (type: string): Recipe => spec.nodes[type] ?? spec.fallback;

  const envFor = (store: ScreenStore, options: MountOptions): Env => ({
    store,
    resolve,
    onAction: options.onAction ?? (() => {}),
    portal: options.portal,
  });

  function mount(container: Element, store: ScreenStore, options: MountOptions = {}): RendererHandle {
    const env = envFor(store, options);
    const root = new NodeInstance(env, store.screen as unknown as IrNode);
    root.mountBefore(container, null);
    return { unmount: () => root.destroy() };
  }

  function mountNodes(
    container: Element,
    nodes: IrNode[],
    store: ScreenStore,
    options: MountOptions = {},
  ): RendererHandle {
    const env = envFor(store, options);
    const region = new ChildrenRegion(env, nodes);
    region.mountInto(container, null);
    return { unmount: () => region.destroy() };
  }

  return { mount, mountNodes };
}

// --- Rendered state -------------------------------------------------------

type Rendered =
  | { k: "text"; dom: Text }
  | { k: "el"; dom: HTMLElement; vnode: VElement; children: Rendered[]; listeners: Map<string, EventListener>; key?: string | number }
  | { k: "host"; dom: HTMLElement; name: string; handle: HostSlotHandle | null }
  | { k: "list"; region: ChildrenRegion }
  | { k: "none"; dom: Comment };

function firstDomOf(r: Rendered): Node {
  return r.k === "list" ? r.region.startAnchor : r.dom;
}

/** Moves a rendered unit before `ref` (handles the multi-node list case). */
function moveRenderedBefore(r: Rendered, parent: Node, ref: Node | null): void {
  if (r.k !== "list") {
    parent.insertBefore(r.dom, ref);
    return;
  }
  let node: Node | null = r.region.startAnchor;
  const range: Node[] = [];
  while (node) {
    range.push(node);
    if (node === r.region.endAnchor) break;
    node = node.nextSibling;
  }
  for (const member of range) parent.insertBefore(member, ref);
}

function removeRendered(r: Rendered): void {
  switch (r.k) {
    case "el":
      for (const child of r.children) removeRendered(child);
      r.dom.remove();
      break;
    case "host":
      r.handle?.unmount();
      r.dom.remove();
      break;
    case "list":
      r.region.destroy();
      break;
    default:
      r.dom.remove();
  }
}

function createRendered(env: Env, v: VNode, parent: Node, ref: Node | null): Rendered {
  if (isVElement(v)) {
    const el = document.createElement(v.tag);
    const listeners = new Map<string, EventListener>();
    applyProps(el, {}, v.props, listeners);
    const children = mountList(env, normalizeChildren(v.children), el, null);
    parent.insertBefore(el, ref);
    return { k: "el", dom: el, vnode: v, children, listeners, key: v.key };
  }
  if (isVHostSlot(v)) {
    const el = document.createElement("span");
    el.setAttribute("data-unode-host-slot", v.name);
    parent.insertBefore(el, ref);
    const handle = env.portal
      ? env.portal.mount(el, { name: v.name, props: v.props, dispatch: env.onAction })
      : fallbackHostSlot(el, v.props);
    return { k: "host", dom: el, name: v.name, handle };
  }
  if (isVChildren(v)) {
    const region = new ChildrenRegion(env, v.nodes);
    region.mountInto(parent, ref);
    return { k: "list", region };
  }
  if (v == null || typeof v === "boolean") {
    const dom = document.createComment("unode:empty");
    parent.insertBefore(dom, ref);
    return { k: "none", dom };
  }
  const dom = document.createTextNode(String(v));
  parent.insertBefore(dom, ref);
  return { k: "text", dom };
}

function compatible(r: Rendered, v: VNode): boolean {
  if (isVElement(v)) return r.k === "el" && r.dom.tagName.toLowerCase() === v.tag.toLowerCase();
  if (isVHostSlot(v)) return r.k === "host";
  if (isVChildren(v)) return r.k === "list";
  if (v == null || typeof v === "boolean") return r.k === "none";
  return r.k === "text";
}

function patchRendered(env: Env, r: Rendered, v: VNode, parent: Node): Rendered {
  if (!compatible(r, v)) {
    const created = createRendered(env, v, parent, firstDomOf(r));
    removeRendered(r);
    return created;
  }
  if (isVElement(v) && r.k === "el") {
    applyProps(r.dom, r.vnode.props, v.props, r.listeners);
    r.children = patchList(env, r.children, normalizeChildren(v.children), r.dom, null);
    r.vnode = v;
    r.key = v.key;
    return r;
  }
  if (isVHostSlot(v) && r.k === "host") {
    if (r.name === v.name) {
      r.handle?.update(v.props);
      return r;
    }
    const created = createRendered(env, v, parent, firstDomOf(r));
    removeRendered(r);
    return created;
  }
  if (isVChildren(v) && r.k === "list") {
    r.region.update(v.nodes);
    return r;
  }
  if (r.k === "text") {
    const next = String(v);
    if (r.dom.nodeValue !== next) r.dom.nodeValue = next;
  }
  return r;
}

function mountList(env: Env, vnodes: VNode[], parent: Node, ref: Node | null): Rendered[] {
  return vnodes.map((v) => createRendered(env, v, parent, ref));
}

function patchList(env: Env, olds: Rendered[], vnodes: VNode[], parent: Node, ref: Node | null): Rendered[] {
  return isKeyedList(vnodes)
    ? patchListKeyed(env, olds, vnodes, parent, ref)
    : patchListPositional(env, olds, vnodes, parent, ref);
}

/** A list is keyed when every entry is a keyed element (e.g. `items.map(...)`). */
function isKeyedList(vnodes: VNode[]): boolean {
  if (vnodes.length === 0) return false;
  for (const v of vnodes) {
    if (!isVElement(v) || v.key == null) return false;
  }
  return true;
}

// Positional reconcile. Recipe structure is static per node, so index alignment
// holds; conditional branches keep their slot via "none" placeholders. Growth
// and shrink happen at the tail.
function patchListPositional(env: Env, olds: Rendered[], vnodes: VNode[], parent: Node, ref: Node | null): Rendered[] {
  const next: Rendered[] = [];
  const count = Math.max(olds.length, vnodes.length);
  for (let i = 0; i < count; i++) {
    const old = olds[i];
    if (i >= vnodes.length) {
      removeRendered(old);
    } else if (old === undefined) {
      next.push(createRendered(env, vnodes[i], parent, ref));
    } else {
      next.push(patchRendered(env, old, vnodes[i], parent));
    }
  }
  return next;
}

// Keyed reconcile: reuse rendered elements by `key`, patch them in place, then
// reorder the DOM to match the new order. This is what makes `items.map(...)`
// reorder correctly instead of rewriting positions.
function patchListKeyed(env: Env, olds: Rendered[], vnodes: VNode[], parent: Node, ref: Node | null): Rendered[] {
  const byKey = new Map<string | number, Rendered>();
  for (const old of olds) {
    if (old.k === "el" && old.key != null) byKey.set(old.key, old);
  }

  const reused = new Set<Rendered>();
  const next: Rendered[] = vnodes.map((v) => {
    const element = v as VElement;
    const match = byKey.get(element.key!);
    if (match && match.k === "el" && match.dom.tagName.toLowerCase() === element.tag.toLowerCase()) {
      reused.add(match);
      return patchRendered(env, match, v, parent);
    }
    return createRendered(env, v, parent, ref);
  });

  for (const old of olds) {
    if (!reused.has(old)) removeRendered(old);
  }

  // Place nodes in order, each before the running reference (walking backwards).
  let runningRef: Node | null = ref;
  for (let i = next.length - 1; i >= 0; i--) {
    moveRenderedBefore(next[i], parent, runningRef);
    runningRef = firstDomOf(next[i]);
  }

  return next;
}

// --- Props ----------------------------------------------------------------

const EVENT = /^on([A-Z].*)$/;

function eventName(key: string): string | null {
  const match = EVENT.exec(key);
  return match ? match[1].toLowerCase() : null;
}

function applyProps(el: HTMLElement, oldProps: VProps, newProps: VProps, listeners: Map<string, EventListener>): void {
  for (const key in oldProps) {
    if (key === "key" || key === "children") continue;
    if (!(key in newProps)) removeProp(el, key, listeners);
  }
  for (const key in newProps) {
    if (key === "key" || key === "children") continue;
    if (newProps[key] !== oldProps[key]) setProp(el, key, newProps[key], listeners);
  }
}

function setProp(el: HTMLElement, key: string, value: unknown, listeners: Map<string, EventListener>): void {
  const event = eventName(key);
  if (event) {
    const previous = listeners.get(event);
    if (previous) el.removeEventListener(event, previous);
    if (typeof value === "function") {
      const listener = value as EventListener;
      el.addEventListener(event, listener);
      listeners.set(event, listener);
    } else {
      listeners.delete(event);
    }
    return;
  }
  if (key === "class" || key === "className") {
    el.className = value == null ? "" : String(value);
    return;
  }
  if (key === "style") {
    applyStyle(el, value);
    return;
  }
  if (key === "value") {
    (el as HTMLInputElement).value = value == null ? "" : String(value);
    return;
  }
  if (typeof value === "boolean") {
    if (value) el.setAttribute(key, "");
    else el.removeAttribute(key);
    try {
      (el as unknown as Record<string, unknown>)[key] = value;
    } catch {
      /* not a reflected property */
    }
    return;
  }
  if (value == null) {
    el.removeAttribute(key);
    return;
  }
  el.setAttribute(key, String(value));
}

function removeProp(el: HTMLElement, key: string, listeners: Map<string, EventListener>): void {
  const event = eventName(key);
  if (event) {
    const previous = listeners.get(event);
    if (previous) el.removeEventListener(event, previous);
    listeners.delete(event);
    return;
  }
  if (key === "class" || key === "className") {
    el.className = "";
    return;
  }
  if (key === "value") {
    (el as HTMLInputElement).value = "";
    return;
  }
  el.removeAttribute(key);
}

function applyStyle(el: HTMLElement, value: unknown): void {
  if (value == null) {
    el.removeAttribute("style");
    return;
  }
  if (typeof value === "string") {
    el.setAttribute("style", value);
    return;
  }
  if (typeof value === "object") {
    el.removeAttribute("style");
    for (const [key, raw] of Object.entries(value as Record<string, unknown>)) {
      if (raw != null) el.style.setProperty(kebab(key), String(raw));
    }
  }
}

function kebab(key: string): string {
  return key.replace(/[A-Z]/g, (m) => `-${m.toLowerCase()}`);
}

function fallbackHostSlot(el: HTMLElement, props: VProps): HostSlotHandle {
  const render = (p: VProps): void => {
    const children = p.children;
    el.textContent = typeof children === "string" || typeof children === "number" ? String(children) : "";
  };
  render(props);
  return {
    update: render,
    unmount: () => {
      el.textContent = "";
    },
  };
}

// --- Instances ------------------------------------------------------------

class NodeInstance {
  private readonly startAnchor: Comment;
  private readonly endAnchor: Comment;
  private readonly key: string;
  private unsubscribe: (() => void) | null = null;
  private rendered: Rendered[] = [];
  private replacement: NodeInstance | null = null;

  constructor(private readonly env: Env, private node: IrNode) {
    this.key = nodeKey(node);
    this.startAnchor = document.createComment("unode:node");
    this.endAnchor = document.createComment("/unode:node");
  }

  firstNode(): Node {
    return this.startAnchor;
  }

  mountBefore(parent: Node, ref: Node | null): void {
    parent.insertBefore(this.startAnchor, ref);
    parent.insertBefore(this.endAnchor, ref);
    if (this.key) {
      this.unsubscribe = this.env.store.subscribe(this.key, () => this.rerender());
    }
    this.rerender();
  }

  /** Moves this instance's whole anchor range before `ref` (keyed reorder). */
  moveBefore(parent: Node, ref: Node): void {
    let node: Node | null = this.startAnchor;
    const range: Node[] = [];
    while (node) {
      range.push(node);
      if (node === this.endAnchor) break;
      node = node.nextSibling;
    }
    for (const member of range) parent.insertBefore(member, ref);
  }

  update(node: IrNode): void {
    this.node = node;
    this.rerender();
  }

  private rerender(): void {
    const parent = this.endAnchor.parentNode;
    if (!parent) return;

    const snapshot = this.env.store.snapshotOf(this.node);

    // Structural replacement into a differently-keyed node: delegate to a nested
    // instance instead of patching our own recipe output.
    if (snapshot.replacement && nodeKey(snapshot.replacement) !== this.key) {
      this.clearRendered();
      if (this.replacement) {
        this.replacement.update(snapshot.replacement);
      } else {
        this.replacement = new NodeInstance(this.env, snapshot.replacement);
        this.replacement.mountBefore(parent, this.endAnchor);
      }
      return;
    }
    if (this.replacement) {
      this.replacement.destroy();
      this.replacement = null;
    }

    const type = snapshot.type;
    const recipe = this.env.resolve(type);
    const ctx = createRendererRecipeContext<VNode>({
      node: snapshot.node,
      type,
      props: snapshot.props,
      children: { $: "children", nodes: snapshot.children } satisfies VChildren,
      childNodes: snapshot.children,
      dispatch: this.env.onAction,
      renderChildren: (nodes: IrNode[] = []) => ({ $: "children", nodes }) satisfies VChildren,
    });

    const output = normalizeChildren(recipe(ctx));
    this.rendered = patchList(this.env, this.rendered, output, parent, this.endAnchor);
  }

  private clearRendered(): void {
    for (const r of this.rendered) removeRendered(r);
    this.rendered = [];
  }

  destroy(): void {
    this.unsubscribe?.();
    this.unsubscribe = null;
    if (this.replacement) {
      this.replacement.destroy();
      this.replacement = null;
    }
    this.clearRendered();
    this.startAnchor.remove();
    this.endAnchor.remove();
  }
}

class ChildrenRegion {
  readonly startAnchor: Comment;
  readonly endAnchor: Comment;
  private readonly instances = new Map<string, NodeInstance>();

  constructor(private readonly env: Env, private nodes: IrNode[]) {
    this.startAnchor = document.createComment("unode:children");
    this.endAnchor = document.createComment("/unode:children");
  }

  mountInto(parent: Node, ref: Node | null): void {
    parent.insertBefore(this.startAnchor, ref);
    parent.insertBefore(this.endAnchor, ref);
    this.render(this.nodes);
  }

  update(nodes: IrNode[]): void {
    this.render(nodes);
  }

  private keyFor(node: IrNode, index: number): string {
    return nodeKey(node) || `#${index}`;
  }

  private render(nodes: IrNode[]): void {
    const parent = this.endAnchor.parentNode;
    if (!parent) return;
    this.nodes = nodes;

    const order: string[] = [];
    const seen = new Set<string>();
    for (let i = 0; i < nodes.length; i++) {
      const key = this.keyFor(nodes[i], i);
      order.push(key);
      seen.add(key);
      const existing = this.instances.get(key);
      if (existing) {
        existing.update(nodes[i]);
      } else {
        const instance = new NodeInstance(this.env, nodes[i]);
        this.instances.set(key, instance);
        instance.mountBefore(parent, this.endAnchor);
      }
    }

    for (const [key, instance] of this.instances) {
      if (!seen.has(key)) {
        instance.destroy();
        this.instances.delete(key);
      }
    }

    // Reorder DOM to match the new key order (place each before the running ref).
    let ref: Node = this.endAnchor;
    for (let i = order.length - 1; i >= 0; i--) {
      const instance = this.instances.get(order[i]);
      if (!instance) continue;
      instance.moveBefore(parent, ref);
      ref = instance.firstNode();
    }
  }

  destroy(): void {
    for (const instance of this.instances.values()) instance.destroy();
    this.instances.clear();
    this.startAnchor.remove();
    this.endAnchor.remove();
  }
}
