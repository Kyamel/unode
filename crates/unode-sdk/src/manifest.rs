use unode::core::ast::StringOrExpr;
use unode::core::permissions::PermissionRequest;
use unode::core::runtime::{
    NavIntent, PluginManifest, RouteDecl, RouteGroupDecl, SlotContributionDecl,
    UNODE_CORE_API_VERSION,
};

/// Starts a plugin permission request builder.
///
/// Permission strings are host-defined capabilities such as `http.fetch` or a
/// domain-specific method group. Mark permissions as required when the plugin
/// cannot function without them; optional permissions can be granted later by
/// host policy.
pub fn permission(permission: impl Into<String>) -> PermissionRequestBuilder {
    PermissionRequestBuilder {
        request: PermissionRequest {
            permission: permission.into(),
            required: false,
            reason: None,
            allowed_origins: vec![],
        },
    }
}

/// Starts a plugin manifest builder with the current core API version.
///
/// Plugin authors normally expose the built manifest through the WASM ABI
/// `plugin_manifest` export. Hosts read it before instantiation to validate API
/// compatibility and requested permissions.
pub fn plugin_manifest(id: impl Into<String>, name: impl Into<String>) -> PluginManifestBuilder {
    let mut manifest = PluginManifest::default();
    manifest.id = id.into();
    manifest.name = name.into();
    PluginManifestBuilder { manifest }
}

#[derive(Debug, Clone)]
pub struct PermissionRequestBuilder {
    request: PermissionRequest,
}

impl PermissionRequestBuilder {
    pub fn required(mut self, required: bool) -> Self {
        self.request.required = required;
        self
    }

    pub fn reason(mut self, reason: impl Into<String>) -> Self {
        self.request.reason = Some(reason.into());
        self
    }

    pub fn allow_origin(mut self, origin: impl Into<String>) -> Self {
        self.request.allowed_origins.push(origin.into());
        self
    }

    pub fn allow_origins<I, S>(mut self, origins: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.request
            .allowed_origins
            .extend(origins.into_iter().map(Into::into));
        self
    }

    pub fn build(self) -> PermissionRequest {
        self.request
    }
}

impl From<PermissionRequestBuilder> for PermissionRequest {
    fn from(value: PermissionRequestBuilder) -> Self {
        value.build()
    }
}

/// Starts a route declaration builder.
///
/// Declared routes tell the host which screens this plugin renders. The host
/// registers them at load time and passes the matched route back through
/// `PluginRenderRequest.route`, so the plugin can branch on
/// `route.pattern` to render multiple screens.
pub fn route(pattern: impl Into<String>) -> RouteDeclBuilder {
    RouteDeclBuilder {
        route: RouteDecl {
            pattern: pattern.into(),
            ..RouteDecl::default()
        },
    }
}

#[derive(Debug, Clone)]
pub struct RouteDeclBuilder {
    route: RouteDecl,
}

impl RouteDeclBuilder {
    pub fn screen_kind(mut self, screen_kind: impl Into<String>) -> Self {
        self.route.screen_kind = Some(screen_kind.into());
        self
    }

    pub fn priority(mut self, priority: i32) -> Self {
        self.route.priority = priority;
        self
    }

    /// Display label for navigation entries and route tabs. Accepts a plain
    /// string or an expression (e.g. `expr::binding("path")`) for dynamic
    /// labels.
    pub fn label(mut self, label: impl Into<StringOrExpr>) -> Self {
        self.route.label = Some(label.into());
        self
    }

    /// Optional badge next to the label. Accepts a plain string or an
    /// expression for dynamic badges (e.g. an unread count bound to state).
    pub fn badge(mut self, badge: impl Into<StringOrExpr>) -> Self {
        self.route.badge = Some(badge.into());
        self
    }

    /// Sugar for a label bound to a state path (dynamic label).
    pub fn label_bind(self, path: impl Into<String>) -> Self {
        self.label(unode::core::dsl::expr::binding::<String>(path))
    }

    /// Sugar for a badge bound to a state path (dynamic badge).
    pub fn badge_bind(self, path: impl Into<String>) -> Self {
        self.badge(unode::core::dsl::expr::binding::<String>(path))
    }

    /// Joins a route group declared with `route_group(...)`.
    pub fn group(mut self, group: impl Into<String>) -> Self {
        self.route.group = Some(group.into());
        self
    }

    pub fn build(self) -> RouteDecl {
        self.route
    }
}

impl From<RouteDeclBuilder> for RouteDecl {
    fn from(value: RouteDeclBuilder) -> Self {
        value.build()
    }
}

/// Starts a route group declaration builder.
///
/// Grouped routes form one navigation set; the intent hints at presentation
/// (`tabs()` or `pages()`), but the renderer decides — hosts without tab
/// support treat the members as separate routes.
pub fn route_group(id: impl Into<String>) -> RouteGroupDeclBuilder {
    RouteGroupDeclBuilder {
        group: RouteGroupDecl {
            id: id.into(),
            intent: NavIntent::default(),
        },
    }
}

#[derive(Debug, Clone)]
pub struct RouteGroupDeclBuilder {
    group: RouteGroupDecl,
}

impl RouteGroupDeclBuilder {
    pub fn intent(mut self, intent: NavIntent) -> Self {
        self.group.intent = intent;
        self
    }

    /// Sugar for `.intent(NavIntent::Tabs)`.
    pub fn tabs(self) -> Self {
        self.intent(NavIntent::Tabs)
    }

    /// Sugar for `.intent(NavIntent::Pages)`.
    pub fn pages(self) -> Self {
        self.intent(NavIntent::Pages)
    }

    pub fn build(self) -> RouteGroupDecl {
        self.group
    }
}

impl From<RouteGroupDeclBuilder> for RouteGroupDecl {
    fn from(value: RouteGroupDeclBuilder) -> Self {
        value.build()
    }
}

#[derive(Debug, Clone)]
pub struct PluginManifestBuilder {
    manifest: PluginManifest,
}

impl PluginManifestBuilder {
    pub fn version(mut self, version: impl Into<String>) -> Self {
        self.manifest.version = version.into();
        self
    }

    pub fn api_version(mut self, api_version: impl Into<String>) -> Self {
        self.manifest.api_version = api_version.into();
        self
    }

    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.manifest.description = Some(description.into());
        self
    }

    pub fn author(mut self, author: impl Into<String>) -> Self {
        self.manifest.author = Some(author.into());
        self
    }

    pub fn require(mut self, dependency: impl Into<String>) -> Self {
        self.manifest.requires.push(dependency.into());
        self
    }

    pub fn permission(mut self, permission: impl Into<PermissionRequest>) -> Self {
        self.manifest.permissions.push(permission.into());
        self
    }

    pub fn permissions<I, P>(mut self, permissions: I) -> Self
    where
        I: IntoIterator<Item = P>,
        P: Into<PermissionRequest>,
    {
        self.manifest
            .permissions
            .extend(permissions.into_iter().map(Into::into));
        self
    }

    pub fn host_id(mut self, host_id: impl Into<String>) -> Self {
        self.manifest.host_id = Some(host_id.into());
        self
    }

    pub fn route(mut self, route: impl Into<RouteDecl>) -> Self {
        self.manifest.routes.push(route.into());
        self
    }

    pub fn route_group(mut self, group: impl Into<RouteGroupDecl>) -> Self {
        self.manifest.route_groups.push(group.into());
        self
    }

    pub fn route_groups<I, G>(mut self, groups: I) -> Self
    where
        I: IntoIterator<Item = G>,
        G: Into<RouteGroupDecl>,
    {
        self.manifest
            .route_groups
            .extend(groups.into_iter().map(Into::into));
        self
    }

    pub fn routes<I, R>(mut self, routes: I) -> Self
    where
        I: IntoIterator<Item = R>,
        R: Into<RouteDecl>,
    {
        self.manifest
            .routes
            .extend(routes.into_iter().map(Into::into));
        self
    }

    pub fn slot_contribution(mut self, contribution: SlotContributionDecl) -> Self {
        self.manifest.slot_contributions.push(contribution);
        self
    }

    pub fn slot_contributions<I>(mut self, contributions: I) -> Self
    where
        I: IntoIterator<Item = SlotContributionDecl>,
    {
        self.manifest.slot_contributions.extend(contributions);
        self
    }

    pub fn build(self) -> PluginManifest {
        self.manifest
    }
}

impl Default for PluginManifestBuilder {
    fn default() -> Self {
        Self {
            manifest: PluginManifest {
                api_version: UNODE_CORE_API_VERSION.to_string(),
                ..PluginManifest::default()
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{permission, plugin_manifest, route};

    #[test]
    fn builds_manifest_with_permissions() {
        let manifest = plugin_manifest("demo.plugin", "Demo")
            .version("1.2.3")
            .description("demo plugin")
            .author("Lucas")
            .require("catalog.read")
            .permission(
                permission("http.fetch")
                    .required(true)
                    .reason("load remote data")
                    .allow_origin("https://api.example.com"),
            )
            .build();

        assert_eq!(manifest.id, "demo.plugin");
        assert_eq!(manifest.name, "Demo");
        assert_eq!(manifest.version, "1.2.3");
        assert_eq!(manifest.permissions.len(), 1);
        assert!(manifest.permissions[0].required);
        assert_eq!(
            manifest.permissions[0].allowed_origins,
            vec!["https://api.example.com".to_string()]
        );
        assert_eq!(manifest.requires, vec!["catalog.read".to_string()]);
    }

    #[test]
    fn builds_manifest_with_routes() {
        let manifest = plugin_manifest("demo.plugin", "Demo")
            .route(route("/notes"))
            .routes([
                route("/notes/:id").screen_kind("demo.plugin.note-detail"),
                route("/settings").priority(10),
            ])
            .build();

        assert!(manifest.validate().is_ok());
        assert_eq!(manifest.routes.len(), 3);
        assert_eq!(manifest.routes[0].pattern, "/notes");
        assert_eq!(manifest.routes[0].screen_kind, None);
        assert_eq!(
            manifest.routes[1].screen_kind.as_deref(),
            Some("demo.plugin.note-detail")
        );
        assert_eq!(manifest.routes[2].priority, 10);
    }

    #[test]
    fn builds_manifest_with_tab_route_group() {
        use unode::core::ast::OneOrExpr;
        use unode::core::dsl::expr::binding;
        use unode::core::runtime::NavIntent;

        let manifest = plugin_manifest("demo.plugin", "Demo")
            .route_group(super::route_group("main").tabs())
            .routes([
                route("/notes").group("main").label("Notes"),
                route("/archive")
                    .group("main")
                    .label("Archive")
                    .badge(binding::<String>("archive.count")),
            ])
            .build();

        assert!(manifest.validate().is_ok());
        assert_eq!(manifest.route_groups.len(), 1);
        assert_eq!(manifest.route_groups[0].intent, NavIntent::Tabs);
        assert_eq!(
            manifest.routes[0].label,
            Some(OneOrExpr::Value("Notes".to_string()))
        );
        assert!(matches!(manifest.routes[1].badge, Some(OneOrExpr::Expr(_))));
    }
}
