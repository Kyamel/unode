use std::collections::BTreeMap;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use thiserror::Error;
use unode::core::ast::{ActionRef, ActionType};
use unode::core::runtime::PluginManifest;

use crate::text::DeferredText;

type ShellAvailability = Arc<dyn Fn(&ShellContext) -> bool + Send + Sync>;
type CommandHandler<Ctx> = Arc<dyn Fn(&Ctx) -> CommandResult + Send + Sync>;
type ActionHandler<Ctx> = Arc<dyn Fn(&ActionRef, &Ctx) -> ActionOutcome + Send + Sync>;
type ActionAvailability<Ctx> = Arc<dyn Fn(&ActionRef, &Ctx) -> bool + Send + Sync>;

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ResolvedRouteInfo {
    pub pathname: String,
    /// The registered pattern that matched (e.g. `/notes/:id`). Plugins that
    /// declare multiple routes branch on this to pick the screen to render.
    #[serde(default)]
    pub pattern: String,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub params: BTreeMap<String, String>,
    pub screen_kind: String,
    pub plugin_id: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ShellContext {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub route: Option<ResolvedRouteInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locale: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub plugin_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub screen_kind: Option<String>,
}

#[derive(Debug, Clone)]
pub struct RegisteredRoute {
    pub plugin_id: String,
    pub pattern: String,
    pub screen_kind: String,
    pub priority: i32,
}

#[derive(Debug, Default, Clone)]
pub struct RouteRegistry {
    routes: Vec<RegisteredRoute>,
}

impl RouteRegistry {
    pub fn register(&mut self, route: RegisteredRoute) {
        self.routes.push(route);
    }

    /// Registers every route declared in a plugin manifest.
    ///
    /// `base_priority` positions the plugin's routes relative to host shell
    /// routes; each declared route's own priority is added on top. Routes
    /// without an explicit `screen_kind` get one derived from the plugin id
    /// and the pattern segments (e.g. `demo.plugin` + `/notes/:id` becomes
    /// `demo.plugin.notes.id`).
    pub fn register_manifest_routes(&mut self, manifest: &PluginManifest, base_priority: i32) {
        for decl in &manifest.routes {
            self.register(RegisteredRoute {
                plugin_id: manifest.id.clone(),
                pattern: decl.pattern.clone(),
                screen_kind: decl
                    .screen_kind
                    .clone()
                    .unwrap_or_else(|| default_screen_kind(&manifest.id, &decl.pattern)),
                priority: base_priority + decl.priority,
            });
        }
    }

    pub fn resolve(&self, pathname: &str) -> Option<ResolvedRouteInfo> {
        let normalized_path = normalize_path(pathname);
        let mut matches = self
            .routes
            .iter()
            .filter_map(|route| {
                match_route_pattern(&route.pattern, &normalized_path).map(|params| {
                    (
                        route.priority,
                        ResolvedRouteInfo {
                            pathname: normalized_path.clone(),
                            pattern: route.pattern.clone(),
                            params,
                            screen_kind: route.screen_kind.clone(),
                            plugin_id: route.plugin_id.clone(),
                        },
                    )
                })
            })
            .collect::<Vec<_>>();

        matches.sort_by(|left, right| right.0.cmp(&left.0));
        matches.into_iter().next().map(|(_, route)| route)
    }
}

#[derive(Clone)]
pub struct RegisteredNavigationItem {
    pub id: String,
    pub plugin_id: String,
    pub label: DeferredText,
    pub short_label: Option<DeferredText>,
    pub to: String,
    pub icon: Option<String>,
    pub section: Option<String>,
    pub priority: i32,
    pub when: Option<ShellAvailability>,
}

impl std::fmt::Debug for RegisteredNavigationItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RegisteredNavigationItem")
            .field("id", &self.id)
            .field("plugin_id", &self.plugin_id)
            .field("to", &self.to)
            .field("priority", &self.priority)
            .finish()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ResolvedNavigationItem {
    pub id: String,
    pub plugin_id: String,
    pub label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub short_label: Option<String>,
    pub to: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub section: Option<String>,
    #[serde(default, skip_serializing_if = "is_zero")]
    pub priority: i32,
}

#[derive(Debug, Default, Clone)]
pub struct NavigationRegistry {
    items: Vec<RegisteredNavigationItem>,
}

impl NavigationRegistry {
    pub fn register(&mut self, item: RegisteredNavigationItem) {
        self.items.push(item);
    }

    pub fn get_available(&self, ctx: &ShellContext) -> Vec<ResolvedNavigationItem> {
        let mut items = self
            .items
            .iter()
            .filter(|item| item.when.as_ref().map(|when| when(ctx)).unwrap_or(true))
            .map(|item| ResolvedNavigationItem {
                id: item.id.clone(),
                plugin_id: item.plugin_id.clone(),
                label: item.label.resolve(),
                short_label: item.short_label.as_ref().map(DeferredText::resolve),
                to: item.to.clone(),
                icon: item.icon.clone(),
                section: item.section.clone(),
                priority: item.priority,
            })
            .collect::<Vec<_>>();

        items.sort_by(|left, right| right.priority.cmp(&left.priority));
        items
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandResult {
    None,
    Navigate(String),
    RefreshCurrentScreen,
    Invalidate(Vec<String>),
}

#[derive(Clone)]
pub struct RegisteredCommand<Ctx> {
    pub id: String,
    pub plugin_id: String,
    pub title: DeferredText,
    pub category: Option<DeferredText>,
    pub keywords: Vec<String>,
    pub when: Option<ShellAvailability>,
    pub run: CommandHandler<Ctx>,
}

impl<Ctx> std::fmt::Debug for RegisteredCommand<Ctx> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RegisteredCommand")
            .field("id", &self.id)
            .field("plugin_id", &self.plugin_id)
            .field("keywords", &self.keywords)
            .finish()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ResolvedCommand {
    pub id: String,
    pub plugin_id: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub keywords: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct CommandRegistry<Ctx> {
    commands: Vec<RegisteredCommand<Ctx>>,
}

impl<Ctx> Default for CommandRegistry<Ctx> {
    fn default() -> Self {
        Self { commands: vec![] }
    }
}

impl<Ctx> CommandRegistry<Ctx> {
    pub fn register(&mut self, command: RegisteredCommand<Ctx>) {
        self.commands.push(command);
    }

    pub fn get_available(&self, ctx: &ShellContext) -> Vec<ResolvedCommand> {
        self.commands
            .iter()
            .filter(|command| command.when.as_ref().map(|when| when(ctx)).unwrap_or(true))
            .map(|command| ResolvedCommand {
                id: command.id.clone(),
                plugin_id: command.plugin_id.clone(),
                title: command.title.resolve(),
                category: command.category.as_ref().map(DeferredText::resolve),
                keywords: command.keywords.clone(),
            })
            .collect()
    }

    pub fn run(
        &self,
        id: &str,
        shell: &ShellContext,
        ctx: &Ctx,
    ) -> Result<CommandResult, ActionRegistryError> {
        let command = self
            .commands
            .iter()
            .find(|command| command.id == id)
            .ok_or_else(|| ActionRegistryError::CommandNotFound(id.to_string()))?;

        if !command
            .when
            .as_ref()
            .map(|when| when(shell))
            .unwrap_or(true)
        {
            return Err(ActionRegistryError::CommandUnavailable(id.to_string()));
        }

        Ok((command.run)(ctx))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActionOutcome {
    Handled,
    RefreshCurrentScreen,
    Navigate(String),
}

#[derive(Clone)]
pub struct RegisteredAction<Ctx> {
    pub id: String,
    pub plugin_id: String,
    pub title: DeferredText,
    pub when: Option<ActionAvailability<Ctx>>,
    pub run: ActionHandler<Ctx>,
}

impl<Ctx> std::fmt::Debug for RegisteredAction<Ctx> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RegisteredAction")
            .field("id", &self.id)
            .field("plugin_id", &self.plugin_id)
            .finish()
    }
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ActionRegistryError {
    #[error("builtin actions are resolved by the host runtime, not the custom action registry")]
    BuiltinAction,
    #[error("custom action not found: {0}")]
    ActionNotFound(String),
    #[error("command not found: {0}")]
    CommandNotFound(String),
    #[error("command unavailable: {0}")]
    CommandUnavailable(String),
    #[error("custom action unavailable: {0}")]
    ActionUnavailable(String),
}

#[derive(Debug, Clone)]
pub struct ActionRegistry<Ctx> {
    actions: Vec<RegisteredAction<Ctx>>,
}

impl<Ctx> Default for ActionRegistry<Ctx> {
    fn default() -> Self {
        Self { actions: vec![] }
    }
}

impl<Ctx> ActionRegistry<Ctx> {
    pub fn register(&mut self, action: RegisteredAction<Ctx>) {
        self.actions.push(action);
    }

    pub fn run(&self, action: &ActionRef, ctx: &Ctx) -> Result<ActionOutcome, ActionRegistryError> {
        let id = match &action.r#type {
            ActionType::Core(_) => return Err(ActionRegistryError::BuiltinAction),
            ActionType::Custom(id) => id,
        };

        let handler = self
            .actions
            .iter()
            .find(|registered| registered.id == *id)
            .ok_or_else(|| ActionRegistryError::ActionNotFound(id.clone()))?;

        if !handler
            .when
            .as_ref()
            .map(|when| when(action, ctx))
            .unwrap_or(true)
        {
            return Err(ActionRegistryError::ActionUnavailable(id.clone()));
        }

        Ok((handler.run)(action, ctx))
    }
}

fn default_screen_kind(plugin_id: &str, pattern: &str) -> String {
    let suffix = pattern
        .split('/')
        .filter(|segment| !segment.is_empty())
        .map(|segment| segment.trim_start_matches(':'))
        .collect::<Vec<_>>()
        .join(".");

    if suffix.is_empty() {
        format!("{plugin_id}.screen")
    } else {
        format!("{plugin_id}.{suffix}")
    }
}

fn normalize_path(pathname: &str) -> String {
    if pathname.len() > 1 && pathname.ends_with('/') {
        pathname[..pathname.len() - 1].to_string()
    } else if pathname.is_empty() {
        "/".to_string()
    } else {
        pathname.to_string()
    }
}

fn match_route_pattern(pattern: &str, pathname: &str) -> Option<BTreeMap<String, String>> {
    let normalized_pattern = normalize_path(pattern);
    let normalized_path = normalize_path(pathname);

    if normalized_pattern == normalized_path {
        return Some(BTreeMap::new());
    }

    let pattern_parts = normalized_pattern
        .split('/')
        .filter(|segment| !segment.is_empty())
        .collect::<Vec<_>>();
    let path_parts = normalized_path
        .split('/')
        .filter(|segment| !segment.is_empty())
        .collect::<Vec<_>>();

    if pattern_parts.len() != path_parts.len() {
        return None;
    }

    let mut params = BTreeMap::new();
    for (pattern_part, path_part) in pattern_parts.iter().zip(path_parts.iter()) {
        if let Some(name) = pattern_part.strip_prefix(':') {
            if name.is_empty() {
                return None;
            }
            params.insert(name.to_string(), path_part.to_string());
            continue;
        }

        if pattern_part != path_part {
            return None;
        }
    }

    Some(params)
}

fn is_zero(value: &i32) -> bool {
    *value == 0
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use unode::core::ast::{ActionRef, ActionType};

    use super::{
        ActionOutcome, ActionRegistry, CommandRegistry, CommandResult, DeferredText,
        NavigationRegistry, RegisteredAction, RegisteredCommand, RegisteredNavigationItem,
        RegisteredRoute, RouteRegistry, ShellContext,
    };

    #[test]
    fn resolves_routes_by_priority_and_params() {
        let mut routes = RouteRegistry::default();
        routes.register(RegisteredRoute {
            plugin_id: "demo.plugin".to_string(),
            pattern: "/works/:id".to_string(),
            screen_kind: "demo.plugin.works.$id".to_string(),
            priority: 10,
        });

        let resolved = routes.resolve("/works/42").expect("route");
        assert_eq!(resolved.plugin_id, "demo.plugin");
        assert_eq!(resolved.pattern, "/works/:id");
        assert_eq!(resolved.params.get("id").map(String::as_str), Some("42"));
    }

    #[test]
    fn registers_manifest_routes_with_derived_screen_kinds() {
        use unode::core::runtime::{PluginManifest, RouteDecl};

        let manifest = PluginManifest {
            id: "demo.plugin".to_string(),
            name: "Demo".to_string(),
            routes: vec![
                RouteDecl {
                    pattern: "/notes".to_string(),
                    screen_kind: None,
                    priority: 0,
                },
                RouteDecl {
                    pattern: "/notes/:id".to_string(),
                    screen_kind: Some("demo.plugin.note-detail".to_string()),
                    priority: 10,
                },
            ],
            ..PluginManifest::default()
        };

        let mut routes = RouteRegistry::default();
        routes.register_manifest_routes(&manifest, 500);

        let list = routes.resolve("/notes").expect("list route");
        assert_eq!(list.plugin_id, "demo.plugin");
        assert_eq!(list.screen_kind, "demo.plugin.notes");

        let detail = routes.resolve("/notes/42").expect("detail route");
        assert_eq!(detail.screen_kind, "demo.plugin.note-detail");
        assert_eq!(detail.pattern, "/notes/:id");
        assert_eq!(detail.params.get("id").map(String::as_str), Some("42"));
    }

    #[test]
    fn resolves_navigation_labels_lazily() {
        let mut registry = NavigationRegistry::default();
        registry.register(RegisteredNavigationItem {
            id: "demo.nav".to_string(),
            plugin_id: "demo.plugin".to_string(),
            label: DeferredText::dynamic(|| "Translated".to_string()),
            short_label: None,
            to: "/demo".to_string(),
            icon: None,
            section: Some("main".to_string()),
            priority: 100,
            when: None,
        });

        let available = registry.get_available(&ShellContext::default());
        assert_eq!(available[0].label, "Translated");
    }

    #[test]
    fn runs_commands_when_available() {
        let mut registry = CommandRegistry::<usize>::default();
        registry.register(RegisteredCommand {
            id: "demo.command".to_string(),
            plugin_id: "demo.plugin".to_string(),
            title: DeferredText::from("Open demo"),
            category: None,
            keywords: vec!["demo".to_string()],
            when: Some(Arc::new(|ctx: &ShellContext| {
                ctx.plugin_id.as_deref() == Some("demo.plugin")
            })),
            run: Arc::new(|ctx| {
                if *ctx == 7 {
                    CommandResult::RefreshCurrentScreen
                } else {
                    CommandResult::None
                }
            }),
        });

        let shell = ShellContext {
            plugin_id: Some("demo.plugin".to_string()),
            ..ShellContext::default()
        };
        assert_eq!(
            registry.run("demo.command", &shell, &7).expect("command"),
            CommandResult::RefreshCurrentScreen
        );
    }

    #[test]
    fn delegates_only_custom_actions_to_registry() {
        let mut registry = ActionRegistry::<usize>::default();
        registry.register(RegisteredAction {
            id: "favorite.toggle".to_string(),
            plugin_id: "demo.plugin".to_string(),
            title: DeferredText::from("Toggle favorite"),
            when: None,
            run: Arc::new(|_, _| ActionOutcome::Handled),
        });

        let action = ActionRef {
            r#type: ActionType::Custom("favorite.toggle".to_string()),
            params: None,
            confirm: None,
        };

        assert_eq!(
            registry.run(&action, &0).expect("action"),
            ActionOutcome::Handled
        );
    }
}
