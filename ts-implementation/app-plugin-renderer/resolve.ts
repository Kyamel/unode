// TODO: remoove this resolve and use unode tracking dependency system instead

import type { BoolOrExpr, NumberOrExpr, PrimitiveOrExpr, StringOrExpr, UiExpr } from '$lib/unode/core/ast';
import type { RendererStateStore } from '$lib/widgets/app-plugin-renderer/context';

type Primitive = string | number | boolean | null;

function isExpr(value: unknown): value is UiExpr {
	return Boolean(value && typeof value === 'object' && 'kind' in value);
}

function resolveExpr(expr: UiExpr, state: RendererStateStore, fallback: Primitive): Primitive {
	if (expr.kind === 'literal') {
		return expr.value;
	}

	if (expr.kind === 'binding') {
		const resolved = state.get(expr.path);
		return resolved === undefined || resolved === null ? fallback : (resolved as Primitive);
	}

	return fallback;
}

export function resolvePrimitiveValue(
	value: PrimitiveOrExpr | undefined,
	state: RendererStateStore,
	fallback: Primitive
): Primitive {
	if (value === undefined) return fallback;
	if (!isExpr(value)) return value;
	return resolveExpr(value, state, fallback);
}

export function resolveStringValue(
	value: StringOrExpr | undefined,
	state: RendererStateStore,
	fallback = ''
): string {
	const resolved = resolvePrimitiveValue(value, state, fallback);
	return resolved === null || resolved === undefined ? fallback : String(resolved);
}

export function resolveBooleanValue(
	value: BoolOrExpr | undefined,
	state: RendererStateStore,
	fallback = false
): boolean {
	const resolved = resolvePrimitiveValue(value, state, fallback);
	return Boolean(resolved);
}

export function resolveNumberValue(
	value: NumberOrExpr | undefined,
	state: RendererStateStore,
	fallback = 0
): number {
	const resolved = resolvePrimitiveValue(value, state, fallback);
	const parsed = Number(resolved);
	return Number.isFinite(parsed) ? parsed : fallback;
}
