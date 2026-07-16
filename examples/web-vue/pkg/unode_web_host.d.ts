/* tslint:disable */
/* eslint-disable */

/**
 * JS-facing handle to a mounted screen. One per active plugin screen.
 */
export class WebSession {
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Apply a flat batch of state writes; returns a JSON array of IR patch
     * ops (`{ o, k, f?, v?, n?, c? }`) for the renderer to re-apply.
     */
    applyWrites(writes_json: string): string;
    /**
     * Initial resolution pass — JSON array of IR patch ops resolving every
     * binding against the seeded state. Apply once right after `mount`.
     */
    initialPatches(): string;
    /**
     * Normalize + track a rendered screen. Returns the IR screen JSON the
     * React adapter mounts.
     *
     * - `screen_json`: the plugin's `render()` output (a raw `ScreenNode`).
     * - `seed_json`: a flat `{ "path": value }` map, or `"{}"`.
     */
    mount(screen_json: string, seed_json: string): string;
    /**
     * Normalize, resolve plugin slot contributions, then track and lower.
     *
     * JS instantiates contributing plugin WASMs and passes their manifest
     * envelopes plus `plugin_render_slot` responses here. The Rust core
     * still owns slot ordering, limits, normalization, and contributor
     * origin annotation.
     */
    mountWithSlots(screen_json: string, seed_json: string, manifests_json: string, slot_responses_json: string): string;
    constructor(locale: string);
    /**
     * Set the active route (JSON `ResolvedRoute`) before `mount`.
     */
    setRoute(route_json: string): void;
    /**
     * Flat snapshot of current state, to feed the plugin as
     * `state_snapshot` on the next dispatch.
     */
    stateSnapshot(): string;
}

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly __wbg_websession_free: (a: number, b: number) => void;
    readonly websession_applyWrites: (a: number, b: number, c: number) => [number, number, number, number];
    readonly websession_initialPatches: (a: number) => [number, number, number, number];
    readonly websession_mount: (a: number, b: number, c: number, d: number, e: number) => [number, number, number, number];
    readonly websession_mountWithSlots: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number) => [number, number, number, number];
    readonly websession_new: (a: number, b: number) => number;
    readonly websession_setRoute: (a: number, b: number, c: number) => [number, number];
    readonly websession_stateSnapshot: (a: number) => [number, number, number, number];
    readonly __wbindgen_externrefs: WebAssembly.Table;
    readonly __wbindgen_malloc: (a: number, b: number) => number;
    readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
    readonly __externref_table_dealloc: (a: number) => void;
    readonly __wbindgen_free: (a: number, b: number, c: number) => void;
    readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;

/**
 * Instantiates the given `module`, which can either be bytes or
 * a precompiled `WebAssembly.Module`.
 *
 * @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
 *
 * @returns {InitOutput}
 */
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
 * If `module_or_path` is {RequestInfo} or {URL}, makes a request and
 * for everything else, calls `WebAssembly.instantiate` directly.
 *
 * @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
 *
 * @returns {Promise<InitOutput>}
 */
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
