import type {
  HostEvent,
  MessageCatalog,
  MessageCatalogs,
  MessageValues,
  Translator
} from '$lib/unode/api/host';
import type { MugenHostApi } from './host';
import { catalogWorksRepo } from '$lib/entities/catalog/api/worksRepo';
import { catalogSearchRepo } from '$lib/entities/catalog/api/searchRepo';
import { catalogChaptersRepo } from '$lib/entities/catalog/api/chaptersRepo';
import type {
  ChapterPageRead,
  ChapterRead,
  ChapterReadingPayloadRead,
  WorkRead
} from '$lib/entities/catalog/model/catalog';
import { auth } from '$lib/entities/user/model/authStore';
import { get } from 'svelte/store';
import type { PluginRuntime } from '$lib/unode/runtime/runtime';
import { reader } from '$lib/shared/state/reader';
import { getLocale } from '$lib/shared/i18n/runtime';
import type {
  ChapterDetails,
  ChapterPagesResult,
  ChapterSummary,
  WorkSummary
} from './models';
import { preferences } from '$lib/shared/state/preferences';

const hostListeners = new Map<string, Set<(event: HostEvent) => void | Promise<void>>>();

function emitHostEvent(event: HostEvent) {
  const list = hostListeners.get(event.type);
  if (!list) return;
  for (const handler of list) {
    void handler(event);
  }
}

function onHostEvent<T extends HostEvent['type']>(
  type: T,
  handler: (event: Extract<HostEvent, { type: T }>) => void | Promise<void>
) {
  const list = hostListeners.get(type) ?? new Set();
  list.add(handler as (event: HostEvent) => void | Promise<void>);
  hostListeners.set(type, list);
  return () => {
    const next = hostListeners.get(type);
    if (!next) return;
    next.delete(handler as (event: HostEvent) => void | Promise<void>);
  };
}

function toWorkSummary(work: WorkRead): WorkSummary {
  return {
    id: work.id,
    title: work.title,
    canonical_slug: work.canonical_slug ?? null,
    cover: work.cover ? { url: work.cover.url, locale: work.cover.locale ?? null } : null,
    work_type: work.work_type ?? null,
    format: work.format ?? null,
    status: work.status ?? null,
    year: work.year ?? null
  };
}

function toChapterSummary(chapter: ChapterRead): ChapterSummary {
  return {
    id: chapter.id,
    title: chapter.title ?? null,
    language_code: chapter.language_code ?? null,
    number: chapter.number ?? null,
    number_numeric: chapter.number_numeric ?? null,
    volume: chapter.sort_major ?? null,
    sort_key: chapter.sort_key ?? null,
    source_url: chapter.source_url ?? null
  };
}

function toChapterDetails(chapter: ChapterRead): ChapterDetails {
  return {
    ...toChapterSummary(chapter),
    pages_count: chapter.pages_count ?? null,
    released_at: chapter.published_at ?? null
  };
}

function toChapterPagesResult(input: {
  pages?: ChapterPageRead[] | null;
  reading_payloads?: ChapterReadingPayloadRead[] | null;
}): ChapterPagesResult {
  return {
    pages:
      input.pages?.map((page) => ({
        chapter_id: page.chapter_id,
        page_index: page.page_index,
        asset_id: page.asset_id,
        created_at: page.created_at ?? null,
        updated_at: page.updated_at ?? null
      })) ?? null,
    reading_payloads:
      input.reading_payloads?.map((payload) => ({
        id: payload.id,
        chapter_id: payload.chapter_id,
        provider: payload.provider,
        base_url: payload.base_url ?? null,
        hash: payload.hash ?? null,
        data: payload.data ?? [],
        data_saver: payload.data_saver ?? [],
        created_at: payload.created_at ?? null,
        updated_at: payload.updated_at ?? null
      })) ?? null
  };
}

function normalizeLocaleKey(locale: string): string {
  return locale.trim().toLowerCase();
}

function findCatalogForLocale(catalogs: MessageCatalogs, locale: string): MessageCatalog | null {
  const normalized = normalizeLocaleKey(locale);
  const exact = catalogs[normalized];
  if (exact) return exact;

  const exactMatch = Object.entries(catalogs).find(
    ([key]) => normalizeLocaleKey(key) === normalized
  )?.[1];
  if (exactMatch) return exactMatch;

  const base = normalized.split('-')[0];
  const baseMatch = Object.entries(catalogs).find(([key]) => {
    const normalizedKey = normalizeLocaleKey(key);
    return normalizedKey === base || normalizedKey.startsWith(`${base}-`);
  })?.[1];
  if (baseMatch) return baseMatch;

  return catalogs.en ?? Object.values(catalogs)[0] ?? null;
}

function lookupMessage(catalog: MessageCatalog, key: string): string | null {
  if (key in catalog && typeof catalog[key] === 'string') {
    return catalog[key] as string;
  }

  const segments = key.split('.');
  let current: string | MessageCatalog | undefined = catalog;
  for (const segment of segments) {
    if (!current || typeof current === 'string' || !(segment in current)) {
      return null;
    }
    current = current[segment] as string | MessageCatalog | undefined;
  }

  return typeof current === 'string' ? current : null;
}

function interpolateMessage(template: string, values?: MessageValues): string {
  if (!values) return template;
  return template.replace(/\{(\w+)\}/g, (_match, key: string) => {
    const value = values[key];
    return value === null || value === undefined ? '' : String(value);
  });
}

function createTranslator(catalogs: MessageCatalogs): Translator {
  const translate = (key: string, values?: MessageValues, fallback?: string) => {
    const locale = getLocale();
    const activeCatalog = findCatalogForLocale(catalogs, locale);
    const fallbackCatalog = findCatalogForLocale(catalogs, 'en');
    const template =
      (activeCatalog ? lookupMessage(activeCatalog, key) : null) ??
      (fallbackCatalog ? lookupMessage(fallbackCatalog, key) : null) ??
      fallback ??
      key;
    return interpolateMessage(template, values);
  };

  return {
    t: translate,
    locale: () => getLocale()
  };
}

type PageLike = {
  url: URL;
  params: Record<string, string>;
};

type GotoFn = (href: string | URL, opts?: Record<string, unknown>) => Promise<void>;

type HostApiOptions = {
  goto: GotoFn;
  page: PageLike;
  runtime?: PluginRuntime;
};

export function createHostApi(options: HostApiOptions): MugenHostApi {
  const { goto, page, runtime } = options;

  return {
    catalog: {
      async listWorks(input) {
        const { filter } = get(preferences);
        const page = await catalogWorksRepo.listWorks({
          limit: input?.limit ?? 24,
          lastCursor: input?.cursor ?? null,
          title: input?.filters?.title,
          workType: input?.filters?.workType,
          status: input?.filters?.status,
          format: input?.filters?.format,
          publishedAfter: input?.filters?.publishedAfter,
          contentFilter: filter
        });
        return {
          data: page.data.map((work) => toWorkSummary(work)),
          lastCursor: page.last_cursor,
          meta: { filterKey: JSON.stringify(filter ?? {}) }
        };
      },
      async searchWorks(input) {
        const response = await catalogSearchRepo.search({
          query: input.query,
          limit: 24
        });
        return {
          data: response.data,
          lastCursor: response.last_cursor
        };
      },
      async getWorkById(id) {
        return catalogWorksRepo.getWork(id);
      },
      async getChapterById(id) {
        const chapter = await catalogChaptersRepo.getChapter(id);
        return toChapterDetails(chapter);
      },
      async getChapterPages(chapterId) {
        const response = await catalogChaptersRepo.getChapterPages(chapterId);
        return toChapterPagesResult(response);
      },
      async listChaptersByWork(workId) {
        const response = await catalogWorksRepo.listWorkChapters(workId);
        return response.data.map((chapter) => toChapterSummary(chapter));
      },
      async listWorkStaff(workId) {
        return catalogWorksRepo.listWorkStaff(workId);
      },
      async listWorkRelations(workId) {
        return catalogWorksRepo.listWorkRelations(workId);
      }
    },
    users: {
      async getCurrentUser() {
        return get(auth).user ?? null;
      },
      async getUserById() {
        return null;
      },
      async isLoggedIn() {
        return Boolean(get(auth).accessToken);
      }
    },
    auth: {
      async requireLogin() {
        return Boolean(get(auth).accessToken);
      },
      async getSessionInfo() {
        const state = get(auth);
        if (!state.accessToken || !state.user?.id) return null;
        return { userId: state.user.id };
      }
    },
    navigation: {
      async navigate(to, opts) {
        await goto(to, { replaceState: opts?.replace, state: opts?.state });
      },
      async openExternal(url, options) {
        if (typeof window === 'undefined' || !url) return;
        window.open(url, options?.target ?? '_blank', 'noopener');
      },
      async openScreen(input) {
        if (!runtime) {
          await goto(input.screenKind);
          return;
        }
        const match = runtime.registries.routes
          .resolve(input.screenKind)
          ?.route?.path;
        if (match) {
          await goto(match);
        }
      },
      async getCurrentRoute() {
        if (runtime) {
          const resolved = runtime.registries.routes.resolveRouteInfo(page.url.pathname);
          if (resolved) return resolved;
        }
        return {
          pathname: page.url.pathname,
          params: page.params as Record<string, string>,
          screenKind: 'unknown',
          pluginId: 'host'
        };
      }
    },
    feedback: {
      toast(input) {
        console.info(`[toast] ${input.title}`, input.message ?? '', input.tone ?? 'info');
      },
      async confirm(input) {
        if (typeof window === 'undefined') return false;
        return window.confirm(input.message ? `${input.title}\n${input.message}` : input.title);
      }
    },
    reader: {
      async openChapter() {
        return;
      },
      async setProgress() {
        return;
      },
      async openImages(images, startIndex = 0, key = 'plugin') {
        reader.open(images, startIndex, key);
      }
    },
    storage: {
      async getScoped<T>(scope: string, key: string) {
        if (typeof localStorage === 'undefined') return null;
        const raw = localStorage.getItem(`host:${scope}:${key}`);
        if (!raw) return null;
        return JSON.parse(raw) as T;
      },
      async setScoped<T>(scope: string, key: string, value: T) {
        if (typeof localStorage === 'undefined') return;
        localStorage.setItem(`host:${scope}:${key}`, JSON.stringify(value));
      }
    },
    http: {
      async request(input) {
        const url = new URL(input.url);
        if (input.query) {
          for (const [key, value] of Object.entries(input.query)) {
            url.searchParams.set(key, String(value));
          }
        }
        const response = await fetch(url.toString(), {
          method: input.method ?? 'GET',
          headers: input.headers,
          body: input.body ? JSON.stringify(input.body) : undefined
        });
        const data = (await response.json()) as unknown;
        return { status: response.status, data: data as any };
      }
    },
    i18n: {
      getLocale() {
        return getLocale();
      },
      translate(input) {
        return createTranslator(input.catalogs).t(input.key, input.values, input.fallback);
      },
      getTranslator(catalogs) {
        return createTranslator(catalogs);
      }
    },
    events: {
      emit(event) {
        emitHostEvent(event);
      },
      on(type, handler) {
        return onHostEvent(type, handler);
      }
    },
    system: {
      async getRuntimeInfo() {
        return {
          platform: 'web',
          appVersion: 'dev',
          pluginApiVersion: runtime?.constructor ? '1.0.0' : '1.0.0'
        };
      }
    }
  };
}
