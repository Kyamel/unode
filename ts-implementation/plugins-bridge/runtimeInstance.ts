import { PluginRuntime } from '$lib/unode/runtime/runtime';
import { EsmPluginLoader } from '$lib/unode/runtime/loader';
import type { MugenHostApi } from './host';
import coreWorkPlugin from '$lib/plugins/mangas/work-meta';
import browseHotPlugin from '$lib/plugins/mangas/browse-hot';
import browseRecentPlugin from '$lib/plugins/mangas/browse-recent';
import browseRecommendedPlugin from '$lib/plugins/mangas/browse-recommended';
import browseFriendsPlugin from '$lib/plugins/mangas/browse-friends';
import workChaptersPlugin from '$lib/plugins/mangas/work-chapters';
import workStaffPlugin from '$lib/plugins/mangas/work-staff';
import workRelatedPlugin from '$lib/plugins/mangas/work-related';
import bannerLabPlugin from '$lib/plugins/tests/banner-lab';
import chaptersLabPlugin from '$lib/plugins/tests/chapters-lab';
import { guardMugenHostApi } from './guard';

const RUNTIME_PLUGIN_STORAGE_KEY = 'mugen.runtime.plugin.urls';
const RUNTIME_PLUGIN_REGISTRY_PATH = '/plugins/registry.json';

type RuntimePluginSource = string | { url?: string; enabled?: boolean };

declare global {
  interface Window {
    __MUGEN_PLUGIN_URLS__?: RuntimePluginSource[];
    __MUGEN_PLUGINS__?: RuntimePluginSource[];
  }
}

const builtinPlugins = [
  coreWorkPlugin,
  browseHotPlugin,
  browseRecentPlugin,
  browseRecommendedPlugin,
  browseFriendsPlugin,
  workChaptersPlugin,
  workStaffPlugin,
  workRelatedPlugin,
  bannerLabPlugin
  ,
  chaptersLabPlugin
] as const;

const runtime = new PluginRuntime({
  hostId: 'mugen',
  guardHostApi: guardMugenHostApi,
  loader: new EsmPluginLoader()
});
let builtinPluginsActivated = false;
let activationPromise: Promise<void> | null = null;
const activatedRuntimePluginUrls = new Set<string>();

function normalizeRuntimePluginUrl(source: unknown): string | null {
  if (typeof source === 'string') {
    const url = source.trim();
    return url ? url : null;
  }

  if (!source || typeof source !== 'object') return null;

  const candidate = source as { url?: unknown; enabled?: unknown };
  if (candidate.enabled === false) return null;
  if (typeof candidate.url !== 'string') return null;

  const url = candidate.url.trim();
  return url ? url : null;
}

function collectRuntimePluginUrls(value: unknown): string[] {
  const sources = Array.isArray(value)
    ? value
    : value && typeof value === 'object' && Array.isArray((value as { plugins?: unknown }).plugins)
      ? (value as { plugins: unknown[] }).plugins
      : [];

  return sources
    .map((source) => normalizeRuntimePluginUrl(source))
    .filter((url): url is string => Boolean(url));
}

async function loadRuntimePluginUrls(): Promise<string[]> {
  const urls = new Set<string>();

  if (typeof window !== 'undefined') {
    for (const source of collectRuntimePluginUrls(window.__MUGEN_PLUGIN_URLS__)) {
      urls.add(source);
    }

    for (const source of collectRuntimePluginUrls(window.__MUGEN_PLUGINS__)) {
      urls.add(source);
    }

    const stored = window.localStorage.getItem(RUNTIME_PLUGIN_STORAGE_KEY);
    if (stored) {
      try {
        for (const source of collectRuntimePluginUrls(JSON.parse(stored))) {
          urls.add(source);
        }
      } catch (error) {
        console.warn('[plugins] Ignoring invalid runtime plugin storage payload.', error);
      }
    }

    try {
      const response = await fetch(RUNTIME_PLUGIN_REGISTRY_PATH, { cache: 'no-store' });
      if (response.ok) {
        for (const source of collectRuntimePluginUrls(await response.json())) {
          urls.add(source);
        }
      }
    } catch (error) {
      console.warn('[plugins] Unable to load runtime plugin registry.', error);
    }
  }

  return [...urls];
}

async function activateRuntimePlugins(hostApi: MugenHostApi) {
  const urls = await loadRuntimePluginUrls();
  for (const url of urls) {
    if (activatedRuntimePluginUrls.has(url)) continue;

    try {
      await runtime.activate([url], hostApi);
      activatedRuntimePluginUrls.add(url);
    } catch (error) {
      console.error(`[plugins] Failed to activate runtime plugin: ${url}`, error);
    }
  }
}

export function getPluginRuntime() {
  return runtime;
}

export async function ensurePluginsActivated(hostApi: MugenHostApi) {
  if (activationPromise) {
    await activationPromise;
    return;
  }

  activationPromise = (async () => {
    if (!builtinPluginsActivated) {
      await runtime.activateModules([...builtinPlugins], hostApi);
      builtinPluginsActivated = true;
    }

    await activateRuntimePlugins(hostApi);
  })();

  try {
    await activationPromise;
  } finally {
    activationPromise = null;
  }
}
