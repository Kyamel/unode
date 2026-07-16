// Host-side derivation of route tabs from manifest route groups — the TS
// mirror of `unode::core::chrome::route_tabs_view`. Plugins declare grouped
// routes with a `tabs` intent; the host derives the tab set for the matched
// route, resolving dynamic labels/badges against the current state snapshot.

/** A manifest text value: plain string or an expression object. */
export type ManifestTextValue =
  | string
  | { kind: "literal"; value: string }
  | { kind: "binding"; path: string }
  | { kind: "param"; name: string };

export interface ManifestRouteDecl {
  pattern: string;
  screenKind?: string;
  priority?: number;
  label?: ManifestTextValue;
  badge?: ManifestTextValue;
  group?: string;
}

export interface ManifestRouteGroupDecl {
  id: string;
  intent?: "tabs" | "pages";
}

/** The subset of a plugin manifest that route-tab derivation reads. */
export interface RouteTabsManifest {
  routes?: ManifestRouteDecl[];
  routeGroups?: ManifestRouteGroupDecl[];
}

export interface RouteTabView {
  /** The route pattern, doubling as the tab id and navigation target. */
  to: string;
  label: string;
  badge?: string;
}

export interface RouteTabsView {
  group: string;
  /** Pattern of the tab that matches the current route. */
  active: string;
  tabs: RouteTabView[];
}

function resolveText(
  value: ManifestTextValue | undefined,
  state: Record<string, unknown>,
): string | undefined {
  if (value == null) return undefined;
  if (typeof value === "string") return value;
  if (value.kind === "literal") return value.value;
  if (value.kind === "binding") {
    const resolved = state[value.path];
    if (resolved == null) return undefined;
    return typeof resolved === "string" ? resolved : JSON.stringify(resolved);
  }
  return undefined;
}

/**
 * Derives the tab set for `activePattern`, if the matched route belongs to a
 * group declared with a `tabs` intent. Returns `undefined` otherwise — the
 * host then presents the route as a standalone screen.
 */
export function routeTabsView(
  manifest: RouteTabsManifest,
  activePattern: string,
  state: Record<string, unknown> = {},
): RouteTabsView | undefined {
  const activeRoute = manifest.routes?.find((route) => route.pattern === activePattern);
  const groupId = activeRoute?.group;
  if (!groupId) return undefined;
  const group = manifest.routeGroups?.find((candidate) => candidate.id === groupId);
  if (group?.intent !== "tabs") return undefined;

  const tabs = (manifest.routes ?? [])
    .filter((route) => route.group === groupId)
    .map((route) => {
      const badge = resolveText(route.badge, state);
      return {
        to: route.pattern,
        label: resolveText(route.label, state) ?? route.pattern,
        ...(badge != null ? { badge } : {}),
      };
    });

  return { group: groupId, active: activePattern, tabs };
}
