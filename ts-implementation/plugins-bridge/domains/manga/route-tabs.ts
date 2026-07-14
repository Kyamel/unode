import type { Translator } from '$lib/unode/api/host';
import {
  createRouteTabsMeta,
  type ScreenRouteTabsMeta
} from '$lib/plugins-bridge/screen-chrome/route-tabs';

export type MangaBrowseRouteTabId = 'hot' | 'recent' | 'recommended' | 'friends';
export type MangaWorkRouteTabId = 'meta' | 'chapters' | 'staff' | 'related';

export function buildMangaBrowseRouteTabs(
  t: Translator['t'],
  active: MangaBrowseRouteTabId
): ScreenRouteTabsMeta {
  return createRouteTabsMeta({
    active,
    tabs: [
      { id: 'hot', label: t('catalog_tab_hot'), to: '/mangas/hot' },
      { id: 'recent', label: t('catalog_tab_recent'), to: '/mangas/recent' },
      {
        id: 'recommended',
        label: t('catalog_tab_recommended'),
        to: '/mangas/recommended'
      },
      { id: 'friends', label: t('catalog_tab_friends'), to: '/mangas/friends' }
    ]
  });
}

export function buildMangaWorkRouteTabs(
  workId: string,
  t: Translator['t'],
  active: MangaWorkRouteTabId
): ScreenRouteTabsMeta {
  return createRouteTabsMeta({
    active,
    tabs: [
      { id: 'meta', label: t('manga_tab_meta'), to: `/mangas/${workId}/meta` },
      {
        id: 'chapters',
        label: t('manga_tab_chapters'),
        to: `/mangas/${workId}/chapters`
      },
      { id: 'staff', label: t('manga_tab_staff'), to: `/mangas/${workId}/staff` },
      { id: 'related', label: t('manga_tab_related'), to: `/mangas/${workId}/related` }
    ]
  });
}
