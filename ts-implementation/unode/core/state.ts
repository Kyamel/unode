import type { Primitive } from './ast';
import type { StateStore, StateListener, Unsubscribe } from './runtime';

function cloneValue<T>(value: T): T {
	return value === undefined ? value : (structuredClone(value) as T);
}

function isObjectLike(value: unknown): value is Record<string, unknown> {
	return value !== null && typeof value === 'object' && !Array.isArray(value);
}

function splitPath(path: string): string[] {
	return path.split('.').filter(Boolean);
}

function getByPath(root: Record<string, unknown>, path: string): unknown {
	const segments = splitPath(path);
	let current: unknown = root;

	for (const segment of segments) {
		if (current === null || current === undefined) return undefined;
		if (Array.isArray(current)) {
			const index = Number(segment);
			if (!Number.isInteger(index)) return undefined;
			current = current[index];
			continue;
		}
		if (typeof current !== 'object') return undefined;
		current = (current as Record<string, unknown>)[segment];
	}

	return current;
}

function setByPath(root: Record<string, unknown>, path: string, value: unknown): void {
	const segments = splitPath(path);
	if (!segments.length) return;

	let current: Record<string, unknown> | unknown[] = root;
	for (let index = 0; index < segments.length - 1; index += 1) {
		const segment = segments[index];
		const nextSegment = segments[index + 1];

		if (Array.isArray(current)) {
			const currentIndex = Number(segment);
			if (!Number.isInteger(currentIndex)) return;
			const existing = current[currentIndex];
			if (existing && typeof existing === 'object') {
				current = existing as Record<string, unknown> | unknown[];
				continue;
			}
			const nextValue: Record<string, unknown> | unknown[] =
				Number.isInteger(Number(nextSegment)) ? [] : {};
			current[currentIndex] = nextValue;
			current = nextValue;
			continue;
		}

		const existing = current[segment];
		if (existing && typeof existing === 'object') {
			current = existing as Record<string, unknown> | unknown[];
			continue;
		}

		const nextValue: Record<string, unknown> | unknown[] =
			Number.isInteger(Number(nextSegment)) ? [] : {};
		current[segment] = nextValue;
		current = nextValue;
	}

	const last = segments[segments.length - 1];
	if (Array.isArray(current)) {
		const lastIndex = Number(last);
		if (!Number.isInteger(lastIndex)) return;
		current[lastIndex] = value;
		return;
	}
	current[last] = value;
}

function mergeInto(target: Record<string, unknown>, source: Record<string, unknown>): void {
	for (const [key, value] of Object.entries(source)) {
		if (isObjectLike(value) && isObjectLike(target[key])) {
			mergeInto(target[key] as Record<string, unknown>, value);
			continue;
		}
		target[key] = cloneValue(value);
	}
}

function expandFlatObject(obj: Record<string, any>) {
	const result: Record<string, any> = {};

	for (const key in obj) {
		const parts = key.split('.');
		let current = result;

		for (let i = 0; i < parts.length; i++) {
			const part = parts[i];

			if (i === parts.length - 1) {
				current[part] = obj[key];
			} else {
				if (!isObjectLike(current[part])) {
					current[part] = {};
				}
				current = current[part];
			}
		}
	}

	return result;
}

export class MemoryStateStore implements StateStore {
	private readonly initialSeed: Record<string, unknown>;
	private data: Record<string, unknown>;
	private readonly exactListeners = new Map<string, Set<StateListener>>();
	private readonly prefixListeners = new Map<string, Set<StateListener>>();
	private batchDepth = 0;
	private readonly pendingPaths = new Set<string>();

	constructor(seed?: Record<string, unknown>) {
		this.initialSeed = expandFlatObject(cloneValue(seed ?? {}));
		this.data = cloneValue(this.initialSeed);
	}

	get(path: string): unknown {
		return getByPath(this.data, path);
	}

	getPrimitive(path: string, fallback: Primitive): Primitive {
		const value = this.get(path);
		if (
			value === null ||
			typeof value === 'string' ||
			typeof value === 'number' ||
			typeof value === 'boolean'
		) {
			return value;
		}
		return fallback;
	}

	set(path: string, value: unknown): void {
		setByPath(this.data, path, cloneValue(value));
		this.queueNotify(path);
	}

	mergeData(data: Record<string, unknown>): void {
		mergeInto(this.data, data);
		for (const key of Object.keys(data)) {
			this.queueNotify(key);
		}
	}

	batch(fn: () => void): void {
		this.batchDepth += 1;
		try {
			fn();
		} finally {
			this.batchDepth -= 1;
			if (this.batchDepth === 0) {
				this.flush();
			}
		}
	}

	subscribe(path: string, listener: StateListener): Unsubscribe {
		const set = this.exactListeners.get(path) ?? new Set<StateListener>();
		set.add(listener);
		this.exactListeners.set(path, set);
		return () => {
			const current = this.exactListeners.get(path);
			if (!current) return;
			current.delete(listener);
			if (current.size === 0) this.exactListeners.delete(path);
		};
	}

	subscribePrefix(prefix: string, listener: StateListener): Unsubscribe {
		const set = this.prefixListeners.get(prefix) ?? new Set<StateListener>();
		set.add(listener);
		this.prefixListeners.set(prefix, set);
		return () => {
			const current = this.prefixListeners.get(prefix);
			if (!current) return;
			current.delete(listener);
			if (current.size === 0) this.prefixListeners.delete(prefix);
		};
	}

	snapshot(): Record<string, unknown> {
		return cloneValue(this.data);
	}

	reset(): void {
		this.data = cloneValue(this.initialSeed);
		for (const key of Object.keys(this.data)) {
			this.queueNotify(key);
		}
		this.flush();
	}

	private queueNotify(path: string): void {
		this.pendingPaths.add(path);
		if (this.batchDepth === 0) {
			this.flush();
		}
	}

	private flush(): void {
		if (this.batchDepth > 0) return;
		for (const path of this.pendingPaths) {
			const value = this.get(path);
			for (const listener of this.exactListeners.get(path) ?? []) {
				listener(value, path);
			}
			for (const [prefix, listeners] of this.prefixListeners.entries()) {
				if (prefix === '' || path === prefix || path.startsWith(`${prefix}.`)) {
					for (const listener of listeners) {
						listener(value, path);
					}
				}
			}
		}
		this.pendingPaths.clear();
	}
}
