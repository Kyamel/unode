use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use crate::core::ast::CollectionContinuation;
use crate::core::canonical::*;
use crate::core::normalize::is_reactive_expr;
use crate::core::resolver::{DefaultExprResolver, ResolverContext};
use crate::core::state::{MemoryStateStore, StateStore, SubscriptionId};

#[derive(Debug, Clone, Default)]
pub struct BindingSubscriptions {
    pub path_to_nodes: BTreeMap<String, BTreeSet<String>>,
    subscription_ids: Vec<SubscriptionId>,
}

impl BindingSubscriptions {
    pub fn teardown(self, state: &mut MemoryStateStore) {
        for id in self.subscription_ids {
            state.unsubscribe(id);
        }
    }
}

pub fn track_reactive_bindings(
    screen: &CanonicalScreen,
    resolver: &mut DefaultExprResolver,
    ctx: &ResolverContext<'_>,
    state: &mut MemoryStateStore,
    on_patch: impl Fn(BTreeSet<String>) + Send + Sync + 'static,
) -> Result<BindingSubscriptions, String> {
    walk_screen(screen, resolver, ctx);

    let mut path_to_nodes = BTreeMap::new();
    for path in resolver.tracked_paths() {
        let nodes = resolver
            .subscribers_of(&path)
            .into_iter()
            .collect::<BTreeSet<_>>();
        if !nodes.is_empty() {
            path_to_nodes.insert(path, nodes);
        }
    }

    let on_patch = Arc::new(on_patch);
    let resolver_snapshot = resolver.clone();
    let listener = {
        let on_patch = Arc::clone(&on_patch);
        Arc::new(move |_, path: &str| {
            let affected = resolver_snapshot
                .subscribers_of(path)
                .into_iter()
                .collect::<BTreeSet<_>>();

            if !affected.is_empty() {
                on_patch(affected);
            }
        })
    };

    let subscription_id = state.subscribe_prefix("", listener);

    Ok(BindingSubscriptions {
        path_to_nodes,
        subscription_ids: vec![subscription_id],
    })
}

fn walk_screen(screen: &CanonicalScreen, resolver: &mut DefaultExprResolver, ctx: &ResolverContext<'_>) {
    if matches!(screen.meta.subtree_reactivity, NodeReactivity::Static) {
        return;
    }

    resolver.clear_tracking(&screen.meta.key);

    if !matches!(screen.meta.reactivity, NodeReactivity::Static) {
        if let Some(title) = &screen.title {
            if is_reactive_expr(title) {
                resolver.resolve_string(title, ctx, Some(&screen.meta.key));
            }
        }
        if let Some(subtitle) = &screen.subtitle {
            if is_reactive_expr(subtitle) {
                resolver.resolve_string(subtitle, ctx, Some(&screen.meta.key));
            }
        }
    }

    for child in &screen.children {
        walk_node(child, resolver, ctx);
    }
}

fn walk_node(node: &CanonicalUiNode, resolver: &mut DefaultExprResolver, ctx: &ResolverContext<'_>) {
    if matches!(node.meta().subtree_reactivity, NodeReactivity::Static) {
        return;
    }

    resolver.clear_tracking(&node.meta().key);

    if !matches!(node.meta().reactivity, NodeReactivity::Static) {
        resolve_node_expressions(node, resolver, ctx);
    }

    match node {
        CanonicalUiNode::Section(node) => walk_nodes(&node.children, resolver, ctx),
        CanonicalUiNode::Stack(node) => walk_nodes(&node.children, resolver, ctx),
        CanonicalUiNode::Inline(node) => walk_nodes(&node.children, resolver, ctx),
        CanonicalUiNode::Grid(node) => walk_nodes(&node.children, resolver, ctx),
        CanonicalUiNode::Scroll(node) => walk_nodes(&node.children, resolver, ctx),
        CanonicalUiNode::Pressable(node) => walk_node(&node.child, resolver, ctx),
        CanonicalUiNode::Item(node) => {
            walk_nodes(&node.leading, resolver, ctx);
            walk_nodes(&node.primary, resolver, ctx);
            walk_nodes(&node.secondary, resolver, ctx);
            walk_nodes(&node.trailing, resolver, ctx);
        }
        CanonicalUiNode::List(node) => {
            for item in &node.items {
                walk_item(item, resolver, ctx);
            }
        }
        CanonicalUiNode::Actions(node) => {
            for child in &node.children {
                walk_action(child, resolver, ctx);
            }
        }
        CanonicalUiNode::Disclosure(node) => walk_nodes(&node.children, resolver, ctx),
        CanonicalUiNode::Form(node) => walk_nodes(&node.children, resolver, ctx),
        CanonicalUiNode::Status(node) => {
            for child in &node.actions {
                walk_action(child, resolver, ctx);
            }
        }
        CanonicalUiNode::Empty(node) => {
            for child in &node.actions {
                walk_action(child, resolver, ctx);
            }
        }
        CanonicalUiNode::Conditional(node) => {
            walk_node(&node.r#then, resolver, ctx);
            if let Some(else_node) = &node.r#else {
                walk_node(else_node, resolver, ctx);
            }
        }
        CanonicalUiNode::Slot(node) => {
            if let Some(fallback) = &node.fallback {
                walk_node(fallback, resolver, ctx);
            }
        }
        CanonicalUiNode::Text(_)
        | CanonicalUiNode::Value(_)
        | CanonicalUiNode::Icon(_)
        | CanonicalUiNode::Badge(_)
        | CanonicalUiNode::Divider(_)
        | CanonicalUiNode::Media(_)
        | CanonicalUiNode::Action(_)
        | CanonicalUiNode::Menu(_)
        | CanonicalUiNode::Input(_)
        | CanonicalUiNode::Loading(_) => {}
    }
}

fn walk_nodes(nodes: &[CanonicalUiNode], resolver: &mut DefaultExprResolver, ctx: &ResolverContext<'_>) {
    for node in nodes {
        walk_node(node, resolver, ctx);
    }
}

fn walk_item(node: &CanonicalItemNode, resolver: &mut DefaultExprResolver, ctx: &ResolverContext<'_>) {
    if matches!(node.meta.subtree_reactivity, NodeReactivity::Static) {
        return;
    }

    resolver.clear_tracking(&node.meta.key);
    walk_nodes(&node.leading, resolver, ctx);
    walk_nodes(&node.primary, resolver, ctx);
    walk_nodes(&node.secondary, resolver, ctx);
    walk_nodes(&node.trailing, resolver, ctx);
}

fn walk_action(node: &CanonicalActionNode, resolver: &mut DefaultExprResolver, ctx: &ResolverContext<'_>) {
    resolver.clear_tracking(&node.meta.key);
    if is_reactive_expr(&node.label) {
        resolver.resolve_string(&node.label, ctx, Some(&node.meta.key));
    }
    if let Some(disabled) = &node.disabled {
        if is_reactive_expr(disabled) {
            resolver.resolve_bool(disabled, ctx, Some(&node.meta.key));
        }
    }
}

fn resolve_node_expressions(
    node: &CanonicalUiNode,
    resolver: &mut DefaultExprResolver,
    ctx: &ResolverContext<'_>,
) {
    let key = node.meta().key.as_str();

    match node {
        CanonicalUiNode::Section(node) => {
            if let Some(title) = &node.title {
                if is_reactive_expr(title) {
                    resolver.resolve_string(title, ctx, Some(key));
                }
            }
            if let Some(description) = &node.description {
                if is_reactive_expr(description) {
                    resolver.resolve_string(description, ctx, Some(key));
                }
            }
        }
        CanonicalUiNode::Text(node) => {
            if is_reactive_expr(&node.content) {
                resolver.resolve_string(&node.content, ctx, Some(key));
            }
        }
        CanonicalUiNode::Value(node) => {
            if is_reactive_expr(&node.value) {
                resolver.resolve_primitive(&node.value, ctx, Some(key));
            }
        }
        CanonicalUiNode::Badge(node) => {
            if is_reactive_expr(&node.label) {
                resolver.resolve_string(&node.label, ctx, Some(key));
            }
        }
        CanonicalUiNode::Divider(node) => {
            if let Some(label) = &node.label {
                if is_reactive_expr(label) {
                    resolver.resolve_string(label, ctx, Some(key));
                }
            }
        }
        CanonicalUiNode::Action(node) => {
            if is_reactive_expr(&node.label) {
                resolver.resolve_string(&node.label, ctx, Some(key));
            }
            if let Some(disabled) = &node.disabled {
                if is_reactive_expr(disabled) {
                    resolver.resolve_bool(disabled, ctx, Some(key));
                }
            }
        }
        CanonicalUiNode::Input(node) => {
            if is_reactive_expr(&node.label) {
                resolver.resolve_string(&node.label, ctx, Some(key));
            }
            if let Some(value) = &node.value {
                if is_reactive_expr(value) {
                    resolver.resolve_primitive(value, ctx, Some(key));
                }
            }
            if let Some(placeholder) = &node.placeholder {
                if is_reactive_expr(placeholder) {
                    resolver.resolve_string(placeholder, ctx, Some(key));
                }
            }
            if let Some(help_text) = &node.help_text {
                if is_reactive_expr(help_text) {
                    resolver.resolve_string(help_text, ctx, Some(key));
                }
            }
            if let Some(disabled) = &node.disabled {
                if is_reactive_expr(disabled) {
                    resolver.resolve_bool(disabled, ctx, Some(key));
                }
            }
        }
        CanonicalUiNode::Disclosure(node) => {
            if is_reactive_expr(&node.label) {
                resolver.resolve_string(&node.label, ctx, Some(key));
            }
            if let Some(label_expanded) = &node.label_expanded {
                if is_reactive_expr(label_expanded) {
                    resolver.resolve_string(label_expanded, ctx, Some(key));
                }
            }
            resolver.track(key, &node.binding);
        }
        CanonicalUiNode::Conditional(node) => {
            if is_reactive_expr(&node.condition) {
                resolver.resolve_bool(&node.condition, ctx, Some(key));
            }
        }
        CanonicalUiNode::Pressable(node) => {
            if let Some(label) = &node.label {
                if is_reactive_expr(label) {
                    resolver.resolve_string(label, ctx, Some(key));
                }
            }
        }
        CanonicalUiNode::Menu(node) => {
            if is_reactive_expr(&node.label) {
                resolver.resolve_string(&node.label, ctx, Some(key));
            }
            for item in &node.items {
                if is_reactive_expr(&item.label) {
                    resolver.resolve_string(&item.label, ctx, Some(key));
                }
                if let Some(disabled) = &item.disabled {
                    if is_reactive_expr(disabled) {
                        resolver.resolve_bool(disabled, ctx, Some(key));
                    }
                }
            }
        }
        CanonicalUiNode::Status(node) => {
            if let Some(title) = &node.title {
                if is_reactive_expr(title) {
                    resolver.resolve_string(title, ctx, Some(key));
                }
            }
            if is_reactive_expr(&node.message) {
                resolver.resolve_string(&node.message, ctx, Some(key));
            }
        }
        CanonicalUiNode::Empty(node) => {
            if is_reactive_expr(&node.title) {
                resolver.resolve_string(&node.title, ctx, Some(key));
            }
            if let Some(message) = &node.message {
                if is_reactive_expr(message) {
                    resolver.resolve_string(message, ctx, Some(key));
                }
            }
        }
        CanonicalUiNode::Loading(node) => {
            if let Some(label) = &node.label {
                if is_reactive_expr(label) {
                    resolver.resolve_string(label, ctx, Some(key));
                }
            }
            if let Some(progress) = &node.progress {
                if is_reactive_expr(progress) {
                    resolver.resolve_number(progress, ctx, Some(key));
                }
            }
        }
        CanonicalUiNode::Grid(node) => {
            if let Some(continuation) = &node.continuation {
                resolve_continuation(continuation, resolver, ctx, key);
            }
        }
        CanonicalUiNode::List(node) => {
            if let Some(continuation) = &node.continuation {
                resolve_continuation(continuation, resolver, ctx, key);
            }
        }
        CanonicalUiNode::Stack(_)
        | CanonicalUiNode::Inline(_)
        | CanonicalUiNode::Scroll(_)
        | CanonicalUiNode::Item(_)
        | CanonicalUiNode::Actions(_)
        | CanonicalUiNode::Form(_)
        | CanonicalUiNode::Media(_)
        | CanonicalUiNode::Icon(_)
        | CanonicalUiNode::Slot(_) => {}
    }
}

fn resolve_continuation(
    continuation: &CollectionContinuation,
    resolver: &mut DefaultExprResolver,
    ctx: &ResolverContext<'_>,
    key: &str,
) {
    match continuation {
        CollectionContinuation::Incremental(value) => {
            if let Some(label) = &value.label {
                if is_reactive_expr(label) {
                    resolver.resolve_string(label, ctx, Some(key));
                }
            }
        }
        CollectionContinuation::Remote(value) => {
            if let Some(label) = &value.label {
                if is_reactive_expr(label) {
                    resolver.resolve_string(label, ctx, Some(key));
                }
            }
            if let Some(label) = &value.loading_label {
                if is_reactive_expr(label) {
                    resolver.resolve_string(label, ctx, Some(key));
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::core::ast::{
        ActionRef, ActionType, CoreActionType, NodeBase, OneOrExpr, PressableNode, ScreenNode,
        SectionNode, TextNode, UiExpr, UiNode,
    };
    use crate::core::normalize::normalize_screen;
    use crate::core::resolver::{DefaultExprResolver, ResolverContext};
    use crate::core::runtime::ResolvedRoute;
    use crate::core::state::{MemoryStateStore, StateStore};

    use super::track_reactive_bindings;

    #[test]
    fn builds_path_to_nodes_for_reactive_screen() {
        let screen = normalize_screen(ScreenNode {
            base: NodeBase {
                id: Some("screen".to_string()),
                meta: None,
            },
            title: None,
            subtitle: None,
            route_tabs: None,
            initial_focus: None,
            initial_state: None,
            children: vec![UiNode::Text(TextNode {
                base: NodeBase {
                    id: Some("hero-title".to_string()),
                    meta: None,
                },
                content: OneOrExpr::Expr(UiExpr::Binding {
                    path: "work.title".to_string(),
                }),
                role: None,
                tone: None,
                emphasis: None,
                truncate: None,
            })],
        })
        .unwrap();

        let mut state = MemoryStateStore::new(None);
        state.set("work.title", json!("Blue Box"));
        let read_state = MemoryStateStore::new(Some(state.snapshot()));

        let route = ResolvedRoute::default();
        let ctx = ResolverContext {
            state: &read_state,
            route: Some(&route),
            locale: "pt-BR",
        };
        let mut resolver = DefaultExprResolver::default();

        let subscriptions = track_reactive_bindings(&screen, &mut resolver, &ctx, &mut state, |_| {}).unwrap();

        assert_eq!(
            subscriptions.path_to_nodes.get("work.title"),
            Some(&["hero-title".to_string()].into_iter().collect())
        );
    }

    #[test]
    fn tracks_nested_descendant_bindings_in_canonical_tree() {
        let screen = normalize_screen(ScreenNode {
            base: NodeBase {
                id: Some("screen".to_string()),
                meta: None,
            },
            title: None,
            subtitle: None,
            route_tabs: None,
            initial_focus: None,
            initial_state: None,
            children: vec![UiNode::Section(SectionNode {
                base: NodeBase {
                    id: Some("hero".to_string()),
                    meta: None,
                },
                role: None,
                title: None,
                description: None,
                children: vec![UiNode::Pressable(PressableNode {
                    base: NodeBase {
                        id: Some("open-profile".to_string()),
                        meta: None,
                    },
                    child: Box::new(UiNode::Text(TextNode {
                        base: NodeBase {
                            id: Some("profile-name".to_string()),
                            meta: None,
                        },
                        content: OneOrExpr::Expr(UiExpr::Binding {
                            path: "profile.name".to_string(),
                        }),
                        role: None,
                        tone: None,
                        emphasis: None,
                        truncate: None,
                    })),
                    action: ActionRef {
                        r#type: ActionType::Core(CoreActionType::Navigate),
                        params: None,
                        confirm: None,
                    },
                    label: None,
                })],
            })],
        })
        .unwrap();

        let mut state = MemoryStateStore::new(None);
        state.set("profile.name", json!("Alice"));
        let read_state = MemoryStateStore::new(Some(state.snapshot()));

        let route = ResolvedRoute::default();
        let ctx = ResolverContext {
            state: &read_state,
            route: Some(&route),
            locale: "pt-BR",
        };
        let mut resolver = DefaultExprResolver::default();

        let subscriptions =
            track_reactive_bindings(&screen, &mut resolver, &ctx, &mut state, |_| {}).unwrap();

        assert_eq!(
            subscriptions.path_to_nodes.get("profile.name"),
            Some(&["profile-name".to_string()].into_iter().collect())
        );
    }
}
