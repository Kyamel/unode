// The renderer-definition API: sane defaults, a fluent builder, and per-node
// override. This is the surface app teams use to map semantic Unode nodes onto
// their design system — the same recipe language on every host.

import { h, type VNode } from "./vnode";
import type { RecipeContext } from "./context";
import { createDomRenderer, type MountOptions, type RendererHandle } from "./dom";
import type { ScreenStore } from "./store";
import type { IrNode } from "./ir";

export type Recipe = (ctx: RecipeContext) => VNode;

export interface RendererSpec {
  /** Recipes keyed by node type; `screen` renders the screen root. */
  nodes: Record<string, Recipe>;
  /** Used for any node type without a recipe. */
  fallback: Recipe;
}

export interface Renderer {
  spec(): RendererSpec;
  /** Mounts a whole screen into `container`. */
  mount(container: Element, store: ScreenStore, options?: MountOptions): RendererHandle;
  /**
   * Mounts a loose list of IR nodes into `container` (e.g. a plugin's
   * contribution to a `shell:header-actions` region), reusing the same store for
   * reactivity.
   */
  mountNodes(container: Element, nodes: IrNode[], store: ScreenStore, options?: MountOptions): RendererHandle;
}

/** Functional, framework-free defaults. Every one is overridable. */
export const defaultRecipes = {
  screen: ({ props, children }) =>
    h(
      "section",
      { class: "unode-screen" },
      props.title != null ? h("h1", { class: "unode-title" }, String(props.title)) : null,
      children,
    ),

  text: ({ content, role }) => h("p", { class: `unode-text unode-text--${role}` }, content),

  actions: ({ children }) => h("div", { class: "unode-actions" }, children),

  action: ({ label, intent, disabled, run }) =>
    h("button", { class: `unode-action unode-action--${intent}`, disabled, onClick: run }, label),

  stack: ({ children }) => h("div", { class: "unode-stack" }, children),

  inline: ({ children }) => h("div", { class: "unode-inline" }, children),

  section: ({ children }) => h("section", { class: "unode-section" }, children),
} satisfies Record<string, Recipe>;

export type BuiltinNodeType = keyof typeof defaultRecipes;

export const defaultFallback: Recipe = ({ children }) => children;

export interface RendererBuilder {
  /** Recipe for the screen root (sugar for `.recipe("screen", ...)`). */
  screen(recipe: Recipe): this;
  /** Override a built-in node type. Use `.custom()` for app-defined node types. */
  recipe(type: BuiltinNodeType, recipe: Recipe): this;
  /** Override many built-in node types at once. */
  recipes(map: Partial<Record<BuiltinNodeType, Recipe>>): this;
  /** Register an app-defined node type. */
  custom(type: string, recipe: Recipe): this;
  /** Register many app-defined node types. */
  customs(map: Record<string, Recipe>): this;
  /** Alias of `recipe`, mirroring the lower-level "node" vocabulary. */
  node(type: BuiltinNodeType, recipe: Recipe): this;
  /** Alias of `recipes`. */
  nodes(map: Partial<Record<BuiltinNodeType, Recipe>>): this;
  /** Recipe used for unknown node types. */
  fallback(recipe: Recipe): this;
  /** The resolved spec (defaults merged with overrides). */
  spec(): RendererSpec;
  /** Materialize a DOM renderer. */
  build(): Renderer;
}

class RendererBuilderImpl implements RendererBuilder {
  private overrides: Record<string, Recipe> = {};
  private fallbackRecipe: Recipe = defaultFallback;

  constructor(base: Partial<RendererSpec> = {}) {
    this.overrides = { ...(base.nodes ?? {}) };
    if (base.fallback) this.fallbackRecipe = base.fallback;
  }

  screen(recipe: Recipe): this {
    return this.recipe("screen", recipe);
  }

  recipe(type: BuiltinNodeType, recipe: Recipe): this {
    this.overrides[type] = recipe;
    return this;
  }

  recipes(map: Partial<Record<BuiltinNodeType, Recipe>>): this {
    Object.assign(this.overrides, map);
    return this;
  }

  custom(type: string, recipe: Recipe): this {
    this.overrides[type] = recipe;
    return this;
  }

  customs(map: Record<string, Recipe>): this {
    Object.assign(this.overrides, map);
    return this;
  }

  node(type: BuiltinNodeType, recipe: Recipe): this {
    return this.recipe(type, recipe);
  }

  nodes(map: Partial<Record<BuiltinNodeType, Recipe>>): this {
    return this.recipes(map);
  }

  fallback(recipe: Recipe): this {
    this.fallbackRecipe = recipe;
    return this;
  }

  spec(): RendererSpec {
    return {
      nodes: { ...defaultRecipes, ...this.overrides },
      fallback: this.fallbackRecipe,
    };
  }

  build(): Renderer {
    return createRenderer(this.spec());
  }
}

/** Entry point: `defineRenderer().recipe("action", ...).custom("app.card", ...).build()`. */
export function defineRenderer(base?: Partial<RendererSpec>): RendererBuilder {
  return new RendererBuilderImpl(base);
}

/** Lower-level: build a renderer straight from a fully-formed spec. */
export function createRenderer(spec: RendererSpec): Renderer {
  const resolved: RendererSpec = {
    nodes: { ...defaultRecipes, ...spec.nodes },
    fallback: spec.fallback ?? defaultFallback,
  };
  const backend = createDomRenderer(resolved);
  return {
    spec: () => resolved,
    mount: backend.mount,
    mountNodes: backend.mountNodes,
  };
}
