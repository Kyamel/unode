use serde::{Deserialize, Serialize};

use crate::core::ast::ScreenNode;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ScreenRouteTab {
    pub id: String,
    pub label: String,
    pub to: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub badge: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ScreenRouteTabsMeta {
    pub kind: String,
    pub active: String,
    pub tabs: Vec<ScreenRouteTab>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub swipe_enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub swipe_threshold: Option<f64>,
}

impl ScreenRouteTabsMeta {
    pub fn new(active: impl Into<String>, tabs: Vec<ScreenRouteTab>) -> Self {
        Self {
            kind: "route-tabs".to_string(),
            active: active.into(),
            tabs,
            swipe_enabled: None,
            swipe_threshold: None,
        }
    }

    pub fn swipe_enabled(mut self, enabled: bool) -> Self {
        self.swipe_enabled = Some(enabled);
        self
    }

    pub fn swipe_threshold(mut self, threshold: f64) -> Self {
        self.swipe_threshold = Some(threshold);
        self
    }
}

pub fn create_route_tabs_meta(
    active: impl Into<String>,
    tabs: Vec<ScreenRouteTab>,
) -> ScreenRouteTabsMeta {
    ScreenRouteTabsMeta::new(active, tabs)
}

pub fn with_route_tabs(mut screen: ScreenNode, route_tabs: ScreenRouteTabsMeta) -> ScreenNode {
    screen.route_tabs = Some(route_tabs);
    screen
}

pub fn read_route_tabs_meta(screen: &ScreenNode) -> Option<&ScreenRouteTabsMeta> {
    screen
        .route_tabs
        .as_ref()
        .filter(|route_tabs| route_tabs.kind == "route-tabs")
}

#[cfg(test)]
mod tests {
    use super::{ScreenRouteTab, create_route_tabs_meta, read_route_tabs_meta, with_route_tabs};
    use crate::core::ast::{NodeBase, ScreenNode};

    #[test]
    fn writes_and_reads_route_tabs_from_screen_meta() {
        let screen = ScreenNode {
            base: NodeBase {
                id: Some("screen".to_string()),
                meta: None,
            },
            title: None,
            subtitle: None,
            route_tabs: None,
            initial_focus: None,
            initial_state: None,
            children: vec![],
        };

        let screen = with_route_tabs(
            screen,
            create_route_tabs_meta(
                "hot",
                vec![
                    ScreenRouteTab {
                        id: "hot".to_string(),
                        label: "Hot".to_string(),
                        to: "/mangas/hot".to_string(),
                        badge: None,
                    },
                    ScreenRouteTab {
                        id: "recent".to_string(),
                        label: "Recent".to_string(),
                        to: "/mangas/recent".to_string(),
                        badge: Some("2".to_string()),
                    },
                ],
            )
            .swipe_enabled(true)
            .swipe_threshold(60.0),
        );

        let route_tabs = read_route_tabs_meta(&screen).expect("route tabs");
        assert_eq!(route_tabs.active, "hot");
        assert_eq!(route_tabs.tabs.len(), 2);
        assert_eq!(route_tabs.tabs[1].badge.as_deref(), Some("2"));
        assert_eq!(route_tabs.swipe_threshold, Some(60.0));
    }

    #[test]
    fn ignores_invalid_route_tabs_payload() {
        let screen = ScreenNode {
            base: NodeBase::default(),
            title: None,
            subtitle: None,
            route_tabs: Some(super::ScreenRouteTabsMeta {
                kind: "something-else".to_string(),
                active: "hot".to_string(),
                tabs: vec![],
                swipe_enabled: None,
                swipe_threshold: None,
            }),
            initial_focus: None,
            initial_state: None,
            children: vec![],
        };

        assert!(read_route_tabs_meta(&screen).is_none());
    }
}
