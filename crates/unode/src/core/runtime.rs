use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::core::ast::{BoolOrExpr, StringOrExpr, UNODE_AST_VERSION};
use crate::core::permissions::PermissionRequest;

pub const UNODE_CORE_API_VERSION: &str = UNODE_AST_VERSION;
pub type PluginId = String;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub struct ResolvedRoute {
    pub pattern: String,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub params: BTreeMap<String, String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub query: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PluginManifest {
    pub id: String,
    pub name: String,
    pub version: String,
    pub api_version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub permissions: Vec<PermissionRequest>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub requires: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub host_id: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub slot_contributions: Vec<SlotContributionDecl>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub routes: Vec<RouteDecl>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub route_groups: Vec<RouteGroupDecl>,
}

/// A route (screen) the plugin offers to render. Hosts register declared
/// routes at load time and dispatch matching navigations back to the plugin's
/// `plugin_render` export with the resolved route, so one plugin can own
/// multiple screens.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub struct RouteDecl {
    /// Path pattern such as `/notes` or `/notes/:id`.
    pub pattern: String,
    /// Semantic screen identifier. Hosts derive one from the plugin id and
    /// pattern when omitted.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub screen_kind: Option<String>,
    /// Relative precedence among this plugin's routes when patterns overlap.
    #[serde(default)]
    pub priority: i32,
    /// Display label used when the host lists this route (navigation entries,
    /// route tabs). Supports state bindings for dynamic labels.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<StringOrExpr>,
    /// Optional badge next to the label. Supports state bindings for dynamic
    /// badges (e.g. an unread count).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub badge: Option<StringOrExpr>,
    /// Membership in a declared [`RouteGroupDecl`]. Grouped routes form one
    /// navigation set; the group's intent hints at the presentation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group: Option<String>,
}

/// How the plugin would like a route group presented. This is an intent, not
/// a command: the renderer decides. A host that supports tabs renders the
/// group as route tabs with the active tab derived from the matched route; a
/// host that does not simply treats the members as separate routes.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub enum NavIntent {
    /// Present the group's routes as tabs of one surface.
    Tabs,
    /// Present the group's routes as independent screens.
    #[default]
    Pages,
}

/// A named set of declared routes sharing a navigation intent.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RouteGroupDecl {
    pub id: String,
    #[serde(default)]
    pub intent: NavIntent,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SlotContributionDecl {
    pub id: String,
    pub target: String,
    #[serde(default)]
    pub priority: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub when: Option<BoolOrExpr>,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ManifestValidationError {
    #[error("slot contribution id must not be empty")]
    EmptySlotContributionId,
    #[error("duplicate slot contribution id `{0}`")]
    DuplicateSlotContributionId(String),
    #[error("slot contribution `{id}` has an empty target")]
    EmptySlotContributionTarget { id: String },
    #[error("route pattern must not be empty")]
    EmptyRoutePattern,
    #[error("duplicate route pattern `{0}`")]
    DuplicateRoutePattern(String),
    #[error("route group id must not be empty")]
    EmptyRouteGroupId,
    #[error("duplicate route group id `{0}`")]
    DuplicateRouteGroupId(String),
    #[error("route `{pattern}` references undeclared group `{group}`")]
    UndeclaredRouteGroup { pattern: String, group: String },
}

impl PluginManifest {
    /// Validates every declarative section of the manifest.
    pub fn validate(&self) -> Result<(), ManifestValidationError> {
        self.validate_slot_contributions()?;
        self.validate_routes()
    }

    pub fn validate_routes(&self) -> Result<(), ManifestValidationError> {
        let mut groups = std::collections::BTreeSet::new();
        for group in &self.route_groups {
            if group.id.trim().is_empty() {
                return Err(ManifestValidationError::EmptyRouteGroupId);
            }
            if !groups.insert(group.id.as_str()) {
                return Err(ManifestValidationError::DuplicateRouteGroupId(
                    group.id.clone(),
                ));
            }
        }

        let mut patterns = std::collections::BTreeSet::new();
        for route in &self.routes {
            if route.pattern.trim().is_empty() {
                return Err(ManifestValidationError::EmptyRoutePattern);
            }
            if !patterns.insert(route.pattern.as_str()) {
                return Err(ManifestValidationError::DuplicateRoutePattern(
                    route.pattern.clone(),
                ));
            }
            if let Some(group) = &route.group {
                if !groups.contains(group.as_str()) {
                    return Err(ManifestValidationError::UndeclaredRouteGroup {
                        pattern: route.pattern.clone(),
                        group: group.clone(),
                    });
                }
            }
        }

        Ok(())
    }

    pub fn validate_slot_contributions(&self) -> Result<(), ManifestValidationError> {
        let mut ids = std::collections::BTreeSet::new();

        for contribution in &self.slot_contributions {
            if contribution.id.trim().is_empty() {
                return Err(ManifestValidationError::EmptySlotContributionId);
            }
            if !ids.insert(contribution.id.as_str()) {
                return Err(ManifestValidationError::DuplicateSlotContributionId(
                    contribution.id.clone(),
                ));
            }
            if contribution.target.trim().is_empty() {
                return Err(ManifestValidationError::EmptySlotContributionTarget {
                    id: contribution.id.clone(),
                });
            }
        }

        Ok(())
    }
}

impl Default for PluginManifest {
    fn default() -> Self {
        Self {
            id: String::new(),
            name: String::new(),
            version: "0.1.0".to_string(),
            api_version: UNODE_CORE_API_VERSION.to_string(),
            description: None,
            author: None,
            permissions: vec![],
            requires: vec![],
            host_id: None,
            slot_contributions: vec![],
            routes: vec![],
            route_groups: vec![],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{ManifestValidationError, NavIntent, PluginManifest, RouteDecl, RouteGroupDecl};
    use crate::core::ast::{OneOrExpr, UiExpr};

    fn manifest_with_routes(routes: Vec<RouteDecl>) -> PluginManifest {
        PluginManifest {
            id: "demo.plugin".to_string(),
            name: "Demo".to_string(),
            routes,
            ..PluginManifest::default()
        }
    }

    #[test]
    fn accepts_multiple_distinct_route_patterns() {
        let manifest = manifest_with_routes(vec![
            RouteDecl {
                pattern: "/notes".to_string(),
                ..RouteDecl::default()
            },
            RouteDecl {
                pattern: "/notes/:id".to_string(),
                screen_kind: Some("demo.plugin.note-detail".to_string()),
                priority: 10,
                ..RouteDecl::default()
            },
        ]);

        assert!(manifest.validate().is_ok());
    }

    #[test]
    fn rejects_empty_route_pattern() {
        let manifest = manifest_with_routes(vec![RouteDecl {
            pattern: "  ".to_string(),
            ..RouteDecl::default()
        }]);

        assert_eq!(
            manifest.validate_routes(),
            Err(ManifestValidationError::EmptyRoutePattern)
        );
    }

    #[test]
    fn rejects_duplicate_route_pattern() {
        let manifest = manifest_with_routes(vec![
            RouteDecl {
                pattern: "/notes".to_string(),
                ..RouteDecl::default()
            },
            RouteDecl {
                pattern: "/notes".to_string(),
                priority: 5,
                ..RouteDecl::default()
            },
        ]);

        assert_eq!(
            manifest.validate_routes(),
            Err(ManifestValidationError::DuplicateRoutePattern(
                "/notes".to_string()
            ))
        );
    }

    #[test]
    fn rejects_route_referencing_undeclared_group() {
        let manifest = manifest_with_routes(vec![RouteDecl {
            pattern: "/notes".to_string(),
            group: Some("main".to_string()),
            ..RouteDecl::default()
        }]);

        assert_eq!(
            manifest.validate_routes(),
            Err(ManifestValidationError::UndeclaredRouteGroup {
                pattern: "/notes".to_string(),
                group: "main".to_string(),
            })
        );
    }

    #[test]
    fn rejects_duplicate_route_group_id() {
        let manifest = PluginManifest {
            route_groups: vec![
                RouteGroupDecl {
                    id: "main".to_string(),
                    intent: NavIntent::Tabs,
                },
                RouteGroupDecl {
                    id: "main".to_string(),
                    intent: NavIntent::Pages,
                },
            ],
            ..PluginManifest::default()
        };

        assert_eq!(
            manifest.validate_routes(),
            Err(ManifestValidationError::DuplicateRouteGroupId(
                "main".to_string()
            ))
        );
    }

    #[test]
    fn routes_round_trip_through_json() {
        let manifest = PluginManifest {
            id: "demo.plugin".to_string(),
            name: "Demo".to_string(),
            route_groups: vec![RouteGroupDecl {
                id: "main".to_string(),
                intent: NavIntent::Tabs,
            }],
            routes: vec![RouteDecl {
                pattern: "/notes/:id".to_string(),
                screen_kind: Some("demo.plugin.note-detail".to_string()),
                priority: 10,
                label: Some(OneOrExpr::Value("Notes".to_string())),
                badge: Some(OneOrExpr::Expr(UiExpr::Binding {
                    path: "notes.unread".to_string(),
                })),
                group: Some("main".to_string()),
            }],
            ..PluginManifest::default()
        };
        assert!(manifest.validate().is_ok());

        let json = serde_json::to_value(&manifest).expect("manifest json");
        assert_eq!(json["routes"][0]["pattern"], "/notes/:id");
        assert_eq!(json["routes"][0]["label"], "Notes");
        assert_eq!(json["routes"][0]["badge"]["kind"], "binding");
        assert_eq!(json["routes"][0]["badge"]["path"], "notes.unread");
        assert_eq!(json["routes"][0]["group"], "main");
        assert_eq!(json["routeGroups"][0]["intent"], "tabs");

        let parsed: PluginManifest = serde_json::from_value(json).expect("manifest parse");
        assert_eq!(parsed, manifest);
    }

    #[test]
    fn manifests_without_routes_stay_compatible() {
        let json = serde_json::json!({
            "id": "demo.plugin",
            "name": "Demo",
            "version": "0.1.0",
            "apiVersion": super::UNODE_CORE_API_VERSION,
        });

        let parsed: PluginManifest = serde_json::from_value(json).expect("manifest parse");
        assert!(parsed.routes.is_empty());
        assert!(
            !serde_json::to_string(&parsed)
                .expect("manifest json")
                .contains("\"routes\"")
        );
    }
}
