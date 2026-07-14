// core/resolver.ts

import type { Primitive, UiExpr, StringOrExpr, BoolOrExpr, NumberOrExpr } from './ast';
import type { ExprResolver, ResolverContext } from './runtime';

export class DefaultExprResolver implements ExprResolver {
  // nodeKey → paths que esse nó lê
  private readonly nodeToPath = new Map<string, Set<string>>();
  // path → nodeKeys que leem esse path
  private readonly pathToNode = new Map<string, Set<string>>();

  track(nodeKey: string, path: string): void {
    if (!this.nodeToPath.has(nodeKey)) this.nodeToPath.set(nodeKey, new Set());
    this.nodeToPath.get(nodeKey)!.add(path);

    if (!this.pathToNode.has(path)) this.pathToNode.set(path, new Set());
    this.pathToNode.get(path)!.add(nodeKey);
  }

  clearTracking(nodeKey: string): void {
    const paths = this.nodeToPath.get(nodeKey);
    if (!paths) return;
    for (const path of paths) {
      const nodes = this.pathToNode.get(path);
      if (nodes) {
        nodes.delete(nodeKey);
        if (nodes.size === 0) this.pathToNode.delete(path);
      }
    }
    this.nodeToPath.delete(nodeKey);
  }

  dependenciesOf(nodeKey: string): readonly string[] {
    return Array.from(this.nodeToPath.get(nodeKey) ?? []);
  }

  subscribersOf(path: string): readonly string[] {
    // Inclui nós que subscrevem o path exato E qualquer ancestral
    // Ex: mudança em "work.title" acorda nós subscritos a "work" ou "work.title"
    const result = new Set<string>();
    for (const [trackedPath, nodes] of this.pathToNode) {
      if (path === trackedPath || path.startsWith(`${trackedPath}.`)) {
        for (const n of nodes) result.add(n);
      }
    }
    return Array.from(result);
  }

  resolvePrimitive(
    expr: Primitive | UiExpr,
    ctx: ResolverContext,
    nodeKey?: string
  ): Primitive {
    if (expr === null || typeof expr !== 'object') return expr as Primitive;

    switch (expr.kind) {
      case 'literal':
        return expr.value;

      case 'binding': {
        if (nodeKey) this.track(nodeKey, expr.path);
        const val = ctx.state.get(expr.path);
        if (val === null || val === undefined) return null;
        if (typeof val === 'string' || typeof val === 'number' || typeof val === 'boolean') return val;
        return String(val);
      }

      case 'param': {
        const val = ctx.route.params[expr.name] ?? ctx.route.query[expr.name];
        return val ?? null;
      }

      default:
        return null;
    }
  }

  resolveString(expr: StringOrExpr, ctx: ResolverContext, nodeKey?: string): string {
    if (typeof expr === 'string') return expr;
    return String(this.resolvePrimitive(expr, ctx, nodeKey) ?? '');
  }

  resolveBoolean(expr: BoolOrExpr, ctx: ResolverContext, nodeKey?: string): boolean {
    if (typeof expr === 'boolean') return expr;
    return Boolean(this.resolvePrimitive(expr, ctx, nodeKey));
  }

  resolveNumber(expr: NumberOrExpr, ctx: ResolverContext, nodeKey?: string): number {
    if (typeof expr === 'number') return expr;
    const v = this.resolvePrimitive(expr, ctx, nodeKey);
    const n = Number(v);
    return isNaN(n) ? 0 : n;
  }
}