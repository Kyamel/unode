use std::collections::{BTreeMap, BTreeSet};

use serde_json::{Number as JsonNumber, Value as JsonValue};

use crate::core::ast::{
    BoolOrExpr, NumberOrExpr, Primitive, PrimitiveOrExpr, StringOrExpr, UiExpr,
};
use crate::core::runtime::ResolvedRoute;
use crate::core::state::{MemoryStateStore, StateStore};

#[derive(Debug, Clone, Copy)]
pub struct ResolverContext<'a> {
    pub state: &'a MemoryStateStore,
    pub route: Option<&'a ResolvedRoute>,
    pub locale: &'a str,
}

#[derive(Debug, Default, Clone)]
pub struct DefaultExprResolver {
    node_to_path: BTreeMap<String, BTreeSet<String>>,
    path_to_node: BTreeMap<String, BTreeSet<String>>,
}

impl DefaultExprResolver {
    /// Records that `node_key` depends on `path`.
    ///
    /// Most callers should not invoke this directly. Use the typed
    /// `resolve_*` methods while walking a canonical tree so dependency
    /// tracking and value resolution stay in sync.
    pub fn track(&mut self, node_key: &str, path: &str) {
        self.node_to_path
            .entry(node_key.to_string())
            .or_default()
            .insert(path.to_string());

        self.path_to_node
            .entry(path.to_string())
            .or_default()
            .insert(node_key.to_string());
    }

    /// Removes all dependencies currently associated with a node key.
    ///
    /// Patch planning calls this before re-resolving a node so dependencies from
    /// an older conditional branch or expression value do not linger.
    pub fn clear_tracking(&mut self, node_key: &str) {
        let Some(paths) = self.node_to_path.remove(node_key) else {
            return;
        };

        for path in paths {
            let mut should_remove = false;

            if let Some(nodes) = self.path_to_node.get_mut(&path) {
                nodes.remove(node_key);
                should_remove = nodes.is_empty();
            }

            if should_remove {
                self.path_to_node.remove(&path);
            }
        }
    }

    /// Returns the state paths read by a node.
    ///
    /// This is mainly useful for diagnostics and tests. Runtime patching usually
    /// asks the inverse question through [`Self::subscribers_of`].
    pub fn dependencies_of(&self, node_key: &str) -> Vec<&str> {
        self.node_to_path
            .get(node_key)
            .into_iter()
            .flat_map(|paths| paths.iter().map(String::as_str))
            .collect()
    }

    /// Returns node keys that should be re-evaluated after a path write.
    ///
    /// Matching is ancestor-aware in both directions: a write to `work.title`
    /// wakes nodes bound to `work.title` and nodes bound to `work`; a write to
    /// `work` wakes nodes bound to `work.title`.
    pub fn subscribers_of(&self, path: &str) -> Vec<String> {
        let mut out = BTreeSet::new();

        for (tracked_path, nodes) in &self.path_to_node {
            if path == tracked_path
                || path.starts_with(&format!("{tracked_path}."))
                || tracked_path.starts_with(&format!("{path}."))
            {
                out.extend(nodes.iter().cloned());
            }
        }

        out.into_iter().collect()
    }

    pub fn tracked_paths(&self) -> Vec<String> {
        self.path_to_node.keys().cloned().collect()
    }

    /// Resolves a primitive expression and optionally records its dependency.
    ///
    /// Pass `Some(node_key)` during tracking or patch planning. Pass `None` when
    /// evaluating outside a node context and no dependency edge should be stored.
    pub fn resolve_primitive(
        &mut self,
        expr: &PrimitiveOrExpr,
        ctx: &ResolverContext<'_>,
        node_key: Option<&str>,
    ) -> Primitive {
        match expr {
            crate::core::ast::OneOrExpr::Value(value) => value.clone(),
            crate::core::ast::OneOrExpr::Expr(expr) => self.resolve_ui_expr(expr, ctx, node_key),
        }
    }

    /// Resolves a string expression from a literal, state binding, or route
    /// parameter.
    ///
    /// Bindings read from the host `MemoryStateStore`; params read from
    /// `ResolvedRoute.params` first and `ResolvedRoute.query` second.
    pub fn resolve_string(
        &mut self,
        expr: &StringOrExpr,
        ctx: &ResolverContext<'_>,
        node_key: Option<&str>,
    ) -> String {
        match expr {
            crate::core::ast::OneOrExpr::Value(value) => value.clone(),
            crate::core::ast::OneOrExpr::Expr(UiExpr::Literal { value }) => value.clone(),
            crate::core::ast::OneOrExpr::Expr(UiExpr::Binding { path }) => {
                if let Some(node_key) = node_key {
                    self.track(node_key, path);
                }
                ctx.state.get(path).map(json_to_string).unwrap_or_default()
            }
            crate::core::ast::OneOrExpr::Expr(UiExpr::Param { name }) => ctx
                .route
                .and_then(|route| route.params.get(name).or_else(|| route.query.get(name)))
                .cloned()
                .unwrap_or_default(),
        }
    }

    pub fn resolve_bool(
        &mut self,
        expr: &BoolOrExpr,
        ctx: &ResolverContext<'_>,
        node_key: Option<&str>,
    ) -> bool {
        match expr {
            crate::core::ast::OneOrExpr::Value(value) => *value,
            crate::core::ast::OneOrExpr::Expr(UiExpr::Literal { value }) => *value,
            crate::core::ast::OneOrExpr::Expr(UiExpr::Binding { path }) => {
                if let Some(node_key) = node_key {
                    self.track(node_key, path);
                }
                ctx.state.get(path).map(json_to_bool).unwrap_or(false)
            }
            crate::core::ast::OneOrExpr::Expr(UiExpr::Param { name }) => ctx
                .route
                .and_then(|route| route.params.get(name).or_else(|| route.query.get(name)))
                .map(|value| matches!(value.as_str(), "1" | "true" | "yes" | "on"))
                .unwrap_or(false),
        }
    }

    pub fn resolve_number(
        &mut self,
        expr: &NumberOrExpr,
        ctx: &ResolverContext<'_>,
        node_key: Option<&str>,
    ) -> f64 {
        match expr {
            crate::core::ast::OneOrExpr::Value(value) => *value,
            crate::core::ast::OneOrExpr::Expr(UiExpr::Literal { value }) => *value,
            crate::core::ast::OneOrExpr::Expr(UiExpr::Binding { path }) => {
                if let Some(node_key) = node_key {
                    self.track(node_key, path);
                }
                ctx.state.get(path).map(json_to_number).unwrap_or_default()
            }
            crate::core::ast::OneOrExpr::Expr(UiExpr::Param { name }) => ctx
                .route
                .and_then(|route| route.params.get(name).or_else(|| route.query.get(name)))
                .and_then(|value| value.parse::<f64>().ok())
                .unwrap_or_default(),
        }
    }

    fn resolve_ui_expr(
        &mut self,
        expr: &UiExpr<Primitive>,
        ctx: &ResolverContext<'_>,
        node_key: Option<&str>,
    ) -> Primitive {
        match expr {
            UiExpr::Literal { value } => value.clone(),
            UiExpr::Binding { path } => {
                if let Some(node_key) = node_key {
                    self.track(node_key, path);
                }

                match ctx.state.get(path) {
                    Some(JsonValue::Null) | None => None,
                    Some(value) => Some(value.clone()),
                }
            }
            UiExpr::Param { name } => ctx
                .route
                .and_then(|route| route.params.get(name).or_else(|| route.query.get(name)))
                .map(|value| JsonValue::String(value.clone())),
        }
    }
}

fn json_to_string(value: &JsonValue) -> String {
    match value {
        JsonValue::String(v) => v.clone(),
        JsonValue::Number(v) => v.to_string(),
        JsonValue::Bool(v) => v.to_string(),
        JsonValue::Null => String::new(),
        JsonValue::Array(_) | JsonValue::Object(_) => value.to_string(),
    }
}

fn json_to_bool(value: &JsonValue) -> bool {
    match value {
        JsonValue::Bool(v) => *v,
        JsonValue::Number(v) => v.as_i64().map(|n| n != 0).unwrap_or(false),
        JsonValue::String(v) => matches!(v.as_str(), "1" | "true" | "yes" | "on"),
        JsonValue::Null => false,
        JsonValue::Array(values) => !values.is_empty(),
        JsonValue::Object(values) => !values.is_empty(),
    }
}

fn json_to_number(value: &JsonValue) -> f64 {
    match value {
        JsonValue::Number(v) => v.as_f64().unwrap_or_default(),
        JsonValue::String(v) => v.parse::<f64>().unwrap_or_default(),
        JsonValue::Bool(true) => 1.0,
        JsonValue::Bool(false) | JsonValue::Null => 0.0,
        JsonValue::Array(_) | JsonValue::Object(_) => 0.0,
    }
}

pub fn primitive_to_json(value: &Primitive) -> JsonValue {
    match value {
        Some(value) => value.clone(),
        None => JsonValue::Null,
    }
}

pub fn string_to_primitive(value: String) -> Primitive {
    Some(JsonValue::String(value))
}

pub fn bool_to_primitive(value: bool) -> Primitive {
    Some(JsonValue::Bool(value))
}

pub fn number_to_primitive(value: f64) -> Primitive {
    JsonNumber::from_f64(value).map(JsonValue::Number)
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use serde_json::json;

    use crate::core::ast::{OneOrExpr, PrimitiveOrExpr, StringOrExpr, UiExpr};
    use crate::core::runtime::ResolvedRoute;
    use crate::core::state::{MemoryStateStore, StateStore};

    use super::{DefaultExprResolver, ResolverContext};

    #[test]
    fn binding_resolution_tracks_dependencies() {
        let mut state = MemoryStateStore::new(None);
        state.set("work.title", json!("Blue Box"));

        let route = ResolvedRoute::default();
        let ctx = ResolverContext {
            state: &state,
            route: Some(&route),
            locale: "pt-BR",
        };
        let mut resolver = DefaultExprResolver::default();

        let expr: StringOrExpr = OneOrExpr::Expr(UiExpr::Binding {
            path: "work.title".to_string(),
        });

        let value = resolver.resolve_string(&expr, &ctx, Some("hero-title"));
        assert_eq!(value, "Blue Box");
        assert_eq!(resolver.dependencies_of("hero-title"), vec!["work.title"]);
        assert_eq!(
            resolver.subscribers_of("work.title"),
            vec!["hero-title".to_string()]
        );
    }

    #[test]
    fn params_can_be_resolved_from_route() {
        let state = MemoryStateStore::new(None);
        let route = ResolvedRoute {
            pattern: "/works/:id".to_string(),
            params: [("id".to_string(), "123".to_string())]
                .into_iter()
                .collect(),
            query: BTreeMap::new(),
        };
        let ctx = ResolverContext {
            state: &state,
            route: Some(&route),
            locale: "pt-BR",
        };

        let mut resolver = DefaultExprResolver::default();
        let expr: PrimitiveOrExpr = OneOrExpr::Expr(UiExpr::Param {
            name: "id".to_string(),
        });

        assert_eq!(
            resolver.resolve_primitive(&expr, &ctx, Some("route-id")),
            Some(json!("123"))
        );
    }
}
