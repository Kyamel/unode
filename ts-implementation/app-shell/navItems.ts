import type { NavigationItem } from '$lib/unode/api/contracts';
import { createHostApi } from '$lib/plugins-bridge/hostApi';
import { ensurePluginsActivated, getPluginRuntime } from '$lib/plugins-bridge/runtimeInstance';
import { m } from '$lib/shared/i18n/messages';

type PageLike = { url: URL; params: Record<string, string> };
type GotoFn = (href: string | URL, opts?: Record<string, unknown>) => Promise<void>;

export type ShellNavItem = {
  id: string;
  label: string;
  shortLabel: string;
  href: string;
};

export function getStaticNavItems(): ShellNavItem[] {
  return [
    { id: 'universe', label: m.nav_universe(), shortLabel: m.nav_universe_short(), href: '/app/universe' },
    { id: 'groups', label: m.nav_groups(), shortLabel: m.nav_groups_short(), href: '/app/groups' },
    { id: 'encyclopedia', label: m.nav_encyclopedia(), shortLabel: m.nav_encyclopedia_short(), href: '/app/explore' },
    { id: 'network', label: m.nav_network(), shortLabel: m.nav_network_short(), href: '/app/network' }
  ];
}

function mapNavItem(item: NavigationItem): ShellNavItem {
  const shortLabel =
    item.shortLabel ??
    item.label
      .trim()
      .split(/\s+/)[0]
      ?.slice(0, 1)
      ?.toUpperCase() ??
    item.label.slice(0, 1).toUpperCase();

  return {
    id: item.id,
    label: item.label,
    shortLabel,
    href: item.to
  };
}

export async function loadShellNavItems(options: {
  page: PageLike;
  goto: GotoFn;
}): Promise<ShellNavItem[]> {
  const runtime = getPluginRuntime();
  const host = createHostApi({ goto: options.goto, page: options.page, runtime });
  await ensurePluginsActivated(host);
  const route = runtime.registries.routes.resolveRouteInfo(options.page.url.pathname);
  const pluginItems = await runtime.registries.navigation.getAvailable({
    screenKind: route?.screenKind,
    route: route ?? undefined,
    host
  });
  const unique = new Map<string, ShellNavItem>();
  for (const item of pluginItems.map(mapNavItem)) {
    if (!unique.has(item.href)) unique.set(item.href, item);
  }
  for (const item of getStaticNavItems()) {
    if (!unique.has(item.href)) unique.set(item.href, item);
  }
  return [...unique.values()];
}
