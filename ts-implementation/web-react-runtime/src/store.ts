// Granular per-node store. This is the piece that turns unode `PatchOp`s into
// React re-renders scoped to a single node key — the web analogue of the TUI
// patching only the affected terminal cells.
//
// Each keyed node component subscribes to its own key via `useSyncExternalStore`.
// A `SetProp` patch bumps only that key's version, so only that one component
// re-renders. `render()` is never re-run; the AST structure is fixed for the
// mount.

import { FIELD_CODE_TO_PROP, IrNode, IrPatchOp, IrScreen, nodeKey } from "./ir";

type Listener = () => void;

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

  // ---- reads (used by components) ----

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
    return () => set!.delete(listener);
  }

  // ---- writes (driven by the host bridge) ----

  applyPatches(ops: IrPatchOp[]): void {
    for (const op of ops) this.applyPatch(op);
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
    this.listeners.get(key)?.forEach((l) => l());
  }
}
