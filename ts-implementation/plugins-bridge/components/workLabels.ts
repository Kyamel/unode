import type { TranslateFn } from '$lib/unode/core/runtime';

export function humanizeWorkToken(value: string): string {
	return value
		.replaceAll(/[_-]+/g, ' ')
		.split(' ')
		.filter(Boolean)
		.map((segment) => segment[0]?.toUpperCase() + segment.slice(1).toLowerCase())
		.join(' ');
}

export function labelForWorkType(
	value: string | null | undefined,
	t: TranslateFn,
	fallbackType: string
): string {
	if (!value) return fallbackType;
	return t(`work_type.${value}`, undefined, humanizeWorkToken(value));
}

export function labelForWorkStatus(value: string | null | undefined, t: TranslateFn): string | null {
	if (!value) return null;
	return t(`work_status.${value}`, undefined, humanizeWorkToken(value));
}
