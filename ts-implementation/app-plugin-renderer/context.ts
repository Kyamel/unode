import { getContext, setContext } from 'svelte';
import type { ActionRef as CoreActionRef, JsonValue, Primitive } from '$lib/unode/core/ast';
import type { StateStore } from '$lib/unode/core/runtime';
import type { RendererConfig } from '$lib/unode/renderer/config';
import { defaultRendererConfig } from '$lib/unode/renderer/config';

export const ACTION_RUNNER_CTX = Symbol('plugin-action-runner');
export const RENDERER_STATE_CTX = Symbol('plugin-renderer-state');
export const RENDERER_CONFIG_CTX = Symbol('plugin-renderer-config');

export type RendererActionRef = CoreActionRef;
export type ActionRunner = (action: RendererActionRef) => void | Promise<void>;

export type RendererStateStore = {
  readonly current: () => StateStore | null;
  get: (path: string) => unknown;
  getPrimitive: (path: string, fallback: Primitive) => Primitive;
  set: (path: string, value: JsonValue) => void;
  toggle: (path: string) => void;
  ensure: (path: string, fallback: JsonValue) => void;
};

export function setActionRunner(runner: ActionRunner) {
  setContext(ACTION_RUNNER_CTX, runner);
}

export function getActionRunner(): ActionRunner {
  return getContext<ActionRunner>(ACTION_RUNNER_CTX);
}

export function setRendererStateStore(store: RendererStateStore) {
  setContext(RENDERER_STATE_CTX, store);
}

export function getRendererStateStore(): RendererStateStore {
  return getContext<RendererStateStore>(RENDERER_STATE_CTX);
}

export function setRendererConfig(config: RendererConfig) {
  setContext(RENDERER_CONFIG_CTX, config);
}

export function getRendererConfig(): RendererConfig {
  return getContext<RendererConfig>(RENDERER_CONFIG_CTX) ?? defaultRendererConfig;
}
