//! Host-side derivation of navigation chrome from manifest route groups.
//!
//! Plugins declare routes and group them with a [`NavIntent`] in the
//! manifest; they never describe presentation. Hosts that support tabs call
//! [`route_tabs_view`] with the matched route pattern and the current state
//! snapshot to obtain a ready-to-render tab set — the active tab is derived
//! from the route, so it can never drift, and labels/badges may be state
//! bindings for dynamic values.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

use crate::core::ast::{OneOrExpr, StringOrExpr, UiExpr};
use crate::core::runtime::{NavIntent, PluginManifest};

/// One resolved tab: plain strings, ready to render.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RouteTabView {
    /// The route pattern, doubling as the tab id and navigation target.
    pub to: String,
    pub label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub badge: Option<String>,
}

/// A resolved route-tab group for the screen currently on display.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RouteTabsView {
    pub group: String,
    /// Pattern of the tab that matches the current route.
    pub active: String,
    pub tabs: Vec<RouteTabView>,
}

/// Derives the tab set for `active_pattern`, if the matched route belongs to
/// a group declared with [`NavIntent::Tabs`]. Returns `None` otherwise — the
/// host then presents the route as a standalone screen.
pub fn route_tabs_view(
    manifest: &PluginManifest,
    active_pattern: &str,
    state: &BTreeMap<String, JsonValue>,
) -> Option<RouteTabsView> {
    let active_route = manifest
        .routes
        .iter()
        .find(|route| route.pattern == active_pattern)?;
    let group_id = active_route.group.as_deref()?;
    let group = manifest
        .route_groups
        .iter()
        .find(|group| group.id == group_id)?;
    if group.intent != NavIntent::Tabs {
        return None;
    }

    let tabs = manifest
        .routes
        .iter()
        .filter(|route| route.group.as_deref() == Some(group_id))
        .map(|route| RouteTabView {
            to: route.pattern.clone(),
            label: route
                .label
                .as_ref()
                .and_then(|label| resolve_text(label, state))
                .unwrap_or_else(|| route.pattern.clone()),
            badge: route
                .badge
                .as_ref()
                .and_then(|badge| resolve_text(badge, state)),
        })
        .collect();

    Some(RouteTabsView {
        group: group_id.to_string(),
        active: active_pattern.to_string(),
        tabs,
    })
}

/// Resolves a manifest text value against the current state snapshot.
/// Bindings that miss (or non-literal params) resolve to `None` so callers
/// can fall back or omit the value.
fn resolve_text(value: &StringOrExpr, state: &BTreeMap<String, JsonValue>) -> Option<String> {
    match value {
        OneOrExpr::Value(text) => Some(text.clone()),
        OneOrExpr::Expr(UiExpr::Literal { value }) => Some(value.clone()),
        OneOrExpr::Expr(UiExpr::Binding { path }) => state.get(path).map(json_to_string),
        OneOrExpr::Expr(UiExpr::Param { .. }) => None,
    }
}

fn json_to_string(value: &JsonValue) -> String {
    value
        .as_str()
        .map(ToString::to_string)
        .unwrap_or_else(|| value.to_string())
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use serde_json::json;

    use super::route_tabs_view;
    use crate::core::ast::OneOrExpr;
    use crate::core::dsl::expr::binding;
    use crate::core::runtime::{NavIntent, PluginManifest, RouteDecl, RouteGroupDecl};

    fn manifest() -> PluginManifest {
        PluginManifest {
            id: "demo.plugin".to_string(),
            name: "Demo".to_string(),
            route_groups: vec![
                RouteGroupDecl {
                    id: "main".to_string(),
                    intent: NavIntent::Tabs,
                },
                RouteGroupDecl {
                    id: "flat".to_string(),
                    intent: NavIntent::Pages,
                },
            ],
            routes: vec![
                RouteDecl {
                    pattern: "/samples/hot".to_string(),
                    label: Some(OneOrExpr::Value("Hot".to_string())),
                    group: Some("main".to_string()),
                    ..RouteDecl::default()
                },
                RouteDecl {
                    pattern: "/samples/recent".to_string(),
                    label: Some(OneOrExpr::Value("Recent".to_string())),
                    badge: Some(OneOrExpr::Expr(binding("mangas.recentCount"))),
                    group: Some("main".to_string()),
                    ..RouteDecl::default()
                },
                RouteDecl {
                    pattern: "/settings".to_string(),
                    group: Some("flat".to_string()),
                    ..RouteDecl::default()
                },
                RouteDecl {
                    pattern: "/about".to_string(),
                    ..RouteDecl::default()
                },
            ],
            ..PluginManifest::default()
        }
    }

    #[test]
    fn derives_tabs_with_active_from_matched_route() {
        let state = BTreeMap::from([("mangas.recentCount".to_string(), json!(2))]);
        let view = route_tabs_view(&manifest(), "/samples/recent", &state).expect("tabs");

        assert_eq!(view.group, "main");
        assert_eq!(view.active, "/samples/recent");
        assert_eq!(view.tabs.len(), 2);
        assert_eq!(view.tabs[0].label, "Hot");
        assert_eq!(view.tabs[0].to, "/samples/hot");
        assert_eq!(view.tabs[0].badge, None);
        assert_eq!(view.tabs[1].badge.as_deref(), Some("2"));
    }

    #[test]
    fn unresolved_badge_binding_is_omitted() {
        let view = route_tabs_view(&manifest(), "/samples/hot", &BTreeMap::new()).expect("tabs");
        assert_eq!(view.tabs[1].badge, None);
    }

    #[test]
    fn missing_label_falls_back_to_pattern() {
        let mut manifest = manifest();
        manifest.routes[0].label = None;
        let view = route_tabs_view(&manifest, "/samples/hot", &BTreeMap::new()).expect("tabs");
        assert_eq!(view.tabs[0].label, "/samples/hot");
    }

    #[test]
    fn pages_intent_and_ungrouped_routes_yield_no_tabs() {
        assert!(route_tabs_view(&manifest(), "/settings", &BTreeMap::new()).is_none());
        assert!(route_tabs_view(&manifest(), "/about", &BTreeMap::new()).is_none());
        assert!(route_tabs_view(&manifest(), "/unknown", &BTreeMap::new()).is_none());
    }
}
