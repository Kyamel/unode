export type ScreenRouteTab = {
  id: string;
  label: string;
  to: string;
  badge?: string;
};

export type ScreenRouteTabsMeta = {
  kind: 'route-tabs';
  active: string;
  tabs: readonly ScreenRouteTab[];
  swipeEnabled?: boolean;
  swipeThreshold?: number;
};

type ScreenMetaRecord = Record<string, unknown>;

function isRecord(value: unknown): value is ScreenMetaRecord {
  return typeof value === 'object' && value !== null && !Array.isArray(value);
}

export function createRouteTabsMeta(input: Omit<ScreenRouteTabsMeta, 'kind'>): ScreenRouteTabsMeta {
  return {
    kind: 'route-tabs',
    ...input
  };
}

export function withRouteTabs<TScreen extends { meta?: Record<string, unknown> }>(
  screen: TScreen,
  routeTabs: ScreenRouteTabsMeta
): TScreen & { meta: Record<string, unknown> } {
  return {
    ...screen,
    meta: {
      ...(screen.meta ?? {}),
      routeTabs
    }
  };
}

export function readRouteTabsMeta(
  meta: Record<string, unknown> | undefined
): ScreenRouteTabsMeta | null {
  if (!isRecord(meta)) return null;
  const candidate = meta.routeTabs;
  if (!isRecord(candidate)) return null;
  if (candidate.kind !== 'route-tabs') return null;
  if (typeof candidate.active !== 'string') return null;
  if (!Array.isArray(candidate.tabs)) return null;

  const tabs = candidate.tabs
    .map((tab) => {
      if (!isRecord(tab)) return null;
      if (typeof tab.id !== 'string') return null;
      if (typeof tab.label !== 'string') return null;
      if (typeof tab.to !== 'string') return null;
      if (tab.badge !== undefined && typeof tab.badge !== 'string') return null;

      return {
        id: tab.id,
        label: tab.label,
        to: tab.to,
        badge: tab.badge
      };
    })
    .filter(Boolean) as ScreenRouteTab[];

  return {
    kind: 'route-tabs',
    active: candidate.active,
    tabs,
    swipeEnabled: typeof candidate.swipeEnabled === 'boolean' ? candidate.swipeEnabled : undefined,
    swipeThreshold:
      typeof candidate.swipeThreshold === 'number' ? candidate.swipeThreshold : undefined
  };
}
