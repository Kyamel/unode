export type Immutable<T> =
	T extends (...args: never[]) => unknown
		? T
		: T extends readonly (infer U)[]
			? readonly Immutable<U>[]
			: T extends object
				? { readonly [K in keyof T]: Immutable<T[K]> }
				: T;

function isObjectLike(value: unknown): value is Record<PropertyKey, unknown> {
	return value !== null && typeof value === 'object';
}

export function deepFreeze<T>(value: T): Immutable<T> {
	if (!isObjectLike(value) || Object.isFrozen(value)) {
		return value as Immutable<T>;
	}

	for (const key of Reflect.ownKeys(value)) {
		const child = value[key];
		if (isObjectLike(child)) {
			deepFreeze(child);
		}
	}

	return Object.freeze(value) as Immutable<T>;
}
