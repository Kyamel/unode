use std::collections::BTreeSet;

use serde_json::{Value as JsonValue, json};

use crate::core::ast::{BoolOrExpr, CollectionContinuation, OneOrExpr, UiExpr};
use crate::core::canonical::*;
use crate::core::patch::{PatchOp, PatchValue};
use crate::core::resolver::{DefaultExprResolver, ResolverContext};

/// Plans renderer patch operations for dirty canonical node keys.
///
/// Hosts call this after state writes have been applied and the resolver has
/// identified affected keys. Static/property-only nodes produce `SetProp`
/// patches, while structurally reactive nodes can produce `ReplaceNode` or
/// `ReplaceChildren` operations.
///
/// The planner reuses the canonical tree from the current mount; it does not
/// call plugin `render()` again for ordinary local state updates.
pub fn plan_patch_ops(
    screen: &CanonicalScreen,
    dirty_keys: &BTreeSet<String>,
    resolver: &mut DefaultExprResolver,
    ctx: &ResolverContext<'_>,
) -> Vec<PatchOp> {
    let mut out = Vec::new();

    for key in dirty_keys {
        let Some(target) = find_target(screen, key) else {
            continue;
        };

        out.extend(plan_target(target, resolver, ctx));
    }

    out
}

enum CanonicalTarget<'a> {
    Screen(&'a CanonicalScreen),
    Node(&'a CanonicalUiNode),
    Action(&'a CanonicalActionNode),
    Item(&'a CanonicalItemNode),
}

impl CanonicalTarget<'_> {
    fn meta(&self) -> &CanonicalMetadata {
        match self {
            Self::Screen(screen) => &screen.meta,
            Self::Node(node) => node.meta(),
            Self::Action(node) => &node.meta,
            Self::Item(node) => &node.meta,
        }
    }
}

fn find_target<'a>(screen: &'a CanonicalScreen, key: &str) -> Option<CanonicalTarget<'a>> {
    if screen.meta.key == key {
        return Some(CanonicalTarget::Screen(screen));
    }

    screen
        .children
        .iter()
        .find_map(|child| find_in_node(child, key))
}

fn find_in_node<'a>(node: &'a CanonicalUiNode, key: &str) -> Option<CanonicalTarget<'a>> {
    if node.meta().key == key {
        return Some(CanonicalTarget::Node(node));
    }

    match node {
        CanonicalUiNode::Section(node) => node
            .children
            .iter()
            .find_map(|child| find_in_node(child, key)),
        CanonicalUiNode::Stack(node) => node
            .children
            .iter()
            .find_map(|child| find_in_node(child, key)),
        CanonicalUiNode::Inline(node) => node
            .children
            .iter()
            .find_map(|child| find_in_node(child, key)),
        CanonicalUiNode::Grid(node) => node
            .children
            .iter()
            .find_map(|child| find_in_node(child, key)),
        CanonicalUiNode::Scroll(node) => node
            .children
            .iter()
            .find_map(|child| find_in_node(child, key)),
        CanonicalUiNode::Pressable(node) => find_in_node(&node.child, key),
        CanonicalUiNode::Item(node) => find_in_item(node, key),
        CanonicalUiNode::List(node) => node.items.iter().find_map(|item| find_in_item(item, key)),
        CanonicalUiNode::Actions(node) => node
            .children
            .iter()
            .find_map(|action| find_in_action(action, key)),
        CanonicalUiNode::Disclosure(node) => node
            .children
            .iter()
            .find_map(|child| find_in_node(child, key)),
        CanonicalUiNode::Form(node) => node
            .children
            .iter()
            .find_map(|child| find_in_node(child, key)),
        CanonicalUiNode::Status(node) => node
            .actions
            .iter()
            .find_map(|action| find_in_action(action, key)),
        CanonicalUiNode::Empty(node) => node
            .actions
            .iter()
            .find_map(|action| find_in_action(action, key)),
        CanonicalUiNode::Conditional(node) => find_in_node(&node.r#then, key).or_else(|| {
            node.r#else
                .as_ref()
                .and_then(|child| find_in_node(child, key))
        }),
        CanonicalUiNode::Slot(node) => node
            .fallback
            .as_ref()
            .and_then(|fallback| find_in_node(fallback, key)),
        CanonicalUiNode::Text(_)
        | CanonicalUiNode::Value(_)
        | CanonicalUiNode::Icon(_)
        | CanonicalUiNode::Badge(_)
        | CanonicalUiNode::Divider(_)
        | CanonicalUiNode::Media(_)
        | CanonicalUiNode::Action(_)
        | CanonicalUiNode::Menu(_)
        | CanonicalUiNode::Input(_)
        | CanonicalUiNode::Loading(_) => None,
    }
}

fn find_in_item<'a>(node: &'a CanonicalItemNode, key: &str) -> Option<CanonicalTarget<'a>> {
    if node.meta.key == key {
        return Some(CanonicalTarget::Item(node));
    }

    node.leading
        .iter()
        .chain(node.primary.iter())
        .chain(node.secondary.iter())
        .chain(node.trailing.iter())
        .find_map(|child| find_in_node(child, key))
}

fn find_in_action<'a>(node: &'a CanonicalActionNode, key: &str) -> Option<CanonicalTarget<'a>> {
    if node.meta.key == key {
        Some(CanonicalTarget::Action(node))
    } else {
        None
    }
}

fn plan_target(
    target: CanonicalTarget<'_>,
    resolver: &mut DefaultExprResolver,
    ctx: &ResolverContext<'_>,
) -> Vec<PatchOp> {
    let meta = target.meta();

    match meta.shape_reactivity {
        ShapeReactivity::ReplaceNode => clone_as_node(&target)
            .map(|node| {
                vec![PatchOp::ReplaceNode {
                    key: meta.key.clone(),
                    node,
                }]
            })
            .unwrap_or_default(),
        ShapeReactivity::ReplaceChildren => extract_children(&target)
            .map(|children| {
                vec![PatchOp::ReplaceChildren {
                    key: meta.key.clone(),
                    children,
                }]
            })
            .unwrap_or_else(|| {
                clone_as_node(&target)
                    .map(|node| {
                        vec![PatchOp::ReplaceNode {
                            key: meta.key.clone(),
                            node,
                        }]
                    })
                    .unwrap_or_default()
            }),
        ShapeReactivity::Static | ShapeReactivity::Visibility => {
            plan_prop_patches(target, resolver, ctx)
        }
    }
}

fn clone_as_node(target: &CanonicalTarget<'_>) -> Option<CanonicalUiNode> {
    match target {
        CanonicalTarget::Node(node) => Some((*node).clone()),
        CanonicalTarget::Action(node) => Some(CanonicalUiNode::Action((*node).clone())),
        CanonicalTarget::Item(node) => Some(CanonicalUiNode::Item((*node).clone())),
        CanonicalTarget::Screen(_) => None,
    }
}

fn extract_children(target: &CanonicalTarget<'_>) -> Option<Vec<CanonicalUiNode>> {
    match target {
        CanonicalTarget::Screen(screen) => Some(screen.children.clone()),
        CanonicalTarget::Node(node) => match node {
            CanonicalUiNode::Section(node) => Some(node.children.clone()),
            CanonicalUiNode::Stack(node) => Some(node.children.clone()),
            CanonicalUiNode::Inline(node) => Some(node.children.clone()),
            CanonicalUiNode::Grid(node) => Some(node.children.clone()),
            CanonicalUiNode::Scroll(node) => Some(node.children.clone()),
            CanonicalUiNode::Pressable(node) => Some(vec![(*node.child).clone()]),
            CanonicalUiNode::Item(node) => Some(
                node.leading
                    .iter()
                    .chain(node.primary.iter())
                    .chain(node.secondary.iter())
                    .chain(node.trailing.iter())
                    .cloned()
                    .collect(),
            ),
            CanonicalUiNode::List(node) => Some(
                node.items
                    .iter()
                    .cloned()
                    .map(CanonicalUiNode::Item)
                    .collect(),
            ),
            CanonicalUiNode::Actions(node) => Some(
                node.children
                    .iter()
                    .cloned()
                    .map(CanonicalUiNode::Action)
                    .collect(),
            ),
            CanonicalUiNode::Disclosure(node) => Some(node.children.clone()),
            CanonicalUiNode::Form(node) => Some(node.children.clone()),
            CanonicalUiNode::Status(node) => Some(
                node.actions
                    .iter()
                    .cloned()
                    .map(CanonicalUiNode::Action)
                    .collect(),
            ),
            CanonicalUiNode::Empty(node) => Some(
                node.actions
                    .iter()
                    .cloned()
                    .map(CanonicalUiNode::Action)
                    .collect(),
            ),
            CanonicalUiNode::Conditional(node) => {
                let mut out = vec![(*node.r#then).clone()];
                if let Some(else_node) = &node.r#else {
                    out.push((**else_node).clone());
                }
                Some(out)
            }
            CanonicalUiNode::Slot(node) => node
                .fallback
                .as_ref()
                .map(|fallback| vec![(**fallback).clone()])
                .or_else(|| Some(vec![])),
            CanonicalUiNode::Text(_)
            | CanonicalUiNode::Value(_)
            | CanonicalUiNode::Icon(_)
            | CanonicalUiNode::Badge(_)
            | CanonicalUiNode::Divider(_)
            | CanonicalUiNode::Media(_)
            | CanonicalUiNode::Action(_)
            | CanonicalUiNode::Menu(_)
            | CanonicalUiNode::Input(_)
            | CanonicalUiNode::Loading(_) => None,
        },
        CanonicalTarget::Action(_) | CanonicalTarget::Item(_) => None,
    }
}

fn plan_prop_patches(
    target: CanonicalTarget<'_>,
    resolver: &mut DefaultExprResolver,
    ctx: &ResolverContext<'_>,
) -> Vec<PatchOp> {
    let key = target.meta().key.clone();
    let mut out = Vec::new();

    for field in &target.meta().reactive_fields {
        if let Some(op) = field_patch(&target, *field, &key, resolver, ctx) {
            out.push(op);
        }
    }

    out
}

fn field_patch(
    target: &CanonicalTarget<'_>,
    field: ReactiveField,
    key: &str,
    resolver: &mut DefaultExprResolver,
    ctx: &ResolverContext<'_>,
) -> Option<PatchOp> {
    let value = match target {
        CanonicalTarget::Screen(screen) => match field {
            ReactiveField::Title => screen
                .title
                .as_ref()
                .map(|value| resolve_string(value, resolver, ctx, key)),
            ReactiveField::Subtitle => screen
                .subtitle
                .as_ref()
                .map(|value| resolve_string(value, resolver, ctx, key)),
            _ => None,
        },
        CanonicalTarget::Node(node) => match node {
            CanonicalUiNode::Section(node) => match field {
                ReactiveField::Title => node
                    .title
                    .as_ref()
                    .map(|value| resolve_string(value, resolver, ctx, key)),
                ReactiveField::Description => node
                    .description
                    .as_ref()
                    .map(|value| resolve_string(value, resolver, ctx, key)),
                _ => None,
            },
            CanonicalUiNode::Text(node) => match field {
                ReactiveField::Content => Some(resolve_string(&node.content, resolver, ctx, key)),
                _ => None,
            },
            CanonicalUiNode::Value(node) => match field {
                ReactiveField::Value => Some(resolve_primitive(&node.value, resolver, ctx, key)),
                _ => None,
            },
            CanonicalUiNode::Badge(node) => match field {
                ReactiveField::Label => Some(resolve_string(&node.label, resolver, ctx, key)),
                _ => None,
            },
            CanonicalUiNode::Divider(node) => match field {
                ReactiveField::Label => node
                    .label
                    .as_ref()
                    .map(|value| resolve_string(value, resolver, ctx, key)),
                _ => None,
            },
            CanonicalUiNode::Pressable(node) => match field {
                ReactiveField::Label => node
                    .label
                    .as_ref()
                    .map(|value| resolve_string(value, resolver, ctx, key)),
                _ => None,
            },
            CanonicalUiNode::Disclosure(node) => match field {
                ReactiveField::BindingState => {
                    Some(resolve_binding_state(&node.binding, resolver, ctx, key))
                }
                ReactiveField::Label => Some(resolve_string(&node.label, resolver, ctx, key)),
                ReactiveField::LabelExpanded => node
                    .label_expanded
                    .as_ref()
                    .map(|value| resolve_string(value, resolver, ctx, key)),
                _ => None,
            },
            CanonicalUiNode::Menu(node) => match field {
                ReactiveField::Label => Some(resolve_string(&node.label, resolver, ctx, key)),
                ReactiveField::MenuItems => Some(PatchValue::Json(resolve_menu_items_json(
                    node, resolver, ctx, key,
                ))),
                _ => None,
            },
            CanonicalUiNode::Input(node) => match field {
                ReactiveField::Label => Some(resolve_string(&node.label, resolver, ctx, key)),
                ReactiveField::Value => node
                    .value
                    .as_ref()
                    .map(|value| resolve_primitive(value, resolver, ctx, key)),
                ReactiveField::Placeholder => node
                    .placeholder
                    .as_ref()
                    .map(|value| resolve_string(value, resolver, ctx, key)),
                ReactiveField::HelpText => node
                    .help_text
                    .as_ref()
                    .map(|value| resolve_string(value, resolver, ctx, key)),
                ReactiveField::Disabled => node
                    .disabled
                    .as_ref()
                    .map(|value| resolve_bool(value, resolver, ctx, key)),
                _ => None,
            },
            CanonicalUiNode::Status(node) => match field {
                ReactiveField::Title => node
                    .title
                    .as_ref()
                    .map(|value| resolve_string(value, resolver, ctx, key)),
                ReactiveField::Message => Some(resolve_string(&node.message, resolver, ctx, key)),
                _ => None,
            },
            CanonicalUiNode::Empty(node) => match field {
                ReactiveField::Title => Some(resolve_string(&node.title, resolver, ctx, key)),
                ReactiveField::Message => node
                    .message
                    .as_ref()
                    .map(|value| resolve_string(value, resolver, ctx, key)),
                _ => None,
            },
            CanonicalUiNode::Loading(node) => match field {
                ReactiveField::Label => node
                    .label
                    .as_ref()
                    .map(|value| resolve_string(value, resolver, ctx, key)),
                ReactiveField::Progress => node
                    .progress
                    .as_ref()
                    .map(|value| resolve_number(value, resolver, ctx, key)),
                _ => None,
            },
            CanonicalUiNode::Grid(node) => match field {
                ReactiveField::Continuation => node.continuation.as_ref().map(|value| {
                    PatchValue::Json(resolve_continuation_json(value, resolver, ctx, key))
                }),
                _ => None,
            },
            CanonicalUiNode::List(node) => match field {
                ReactiveField::Continuation => node.continuation.as_ref().map(|value| {
                    PatchValue::Json(resolve_continuation_json(value, resolver, ctx, key))
                }),
                _ => None,
            },
            CanonicalUiNode::Conditional(_)
            | CanonicalUiNode::Stack(_)
            | CanonicalUiNode::Inline(_)
            | CanonicalUiNode::Scroll(_)
            | CanonicalUiNode::Icon(_)
            | CanonicalUiNode::Media(_)
            | CanonicalUiNode::Item(_)
            | CanonicalUiNode::Actions(_)
            | CanonicalUiNode::Form(_)
            | CanonicalUiNode::Slot(_) => None,
            CanonicalUiNode::Action(node) => action_field_patch(node, field, resolver, ctx, key),
        },
        CanonicalTarget::Action(node) => action_field_patch(node, field, resolver, ctx, key),
        CanonicalTarget::Item(_) => None,
    }?;

    Some(PatchOp::SetProp {
        key: key.to_string(),
        field,
        value,
    })
}

fn action_field_patch(
    node: &CanonicalActionNode,
    field: ReactiveField,
    resolver: &mut DefaultExprResolver,
    ctx: &ResolverContext<'_>,
    key: &str,
) -> Option<PatchValue> {
    match field {
        ReactiveField::Label => Some(resolve_string(&node.label, resolver, ctx, key)),
        ReactiveField::Disabled => node
            .disabled
            .as_ref()
            .map(|value| resolve_bool(value, resolver, ctx, key)),
        _ => None,
    }
}

fn resolve_string(
    value: &crate::core::ast::StringOrExpr,
    resolver: &mut DefaultExprResolver,
    ctx: &ResolverContext<'_>,
    key: &str,
) -> PatchValue {
    PatchValue::String(OneOrExpr::Value(resolver.resolve_string(
        value,
        ctx,
        Some(key),
    )))
}

fn resolve_bool(
    value: &crate::core::ast::BoolOrExpr,
    resolver: &mut DefaultExprResolver,
    ctx: &ResolverContext<'_>,
    key: &str,
) -> PatchValue {
    PatchValue::Bool(OneOrExpr::Value(resolver.resolve_bool(
        value,
        ctx,
        Some(key),
    )))
}

fn resolve_number(
    value: &crate::core::ast::NumberOrExpr,
    resolver: &mut DefaultExprResolver,
    ctx: &ResolverContext<'_>,
    key: &str,
) -> PatchValue {
    PatchValue::Number(OneOrExpr::Value(resolver.resolve_number(
        value,
        ctx,
        Some(key),
    )))
}

fn resolve_primitive(
    value: &crate::core::ast::PrimitiveOrExpr,
    resolver: &mut DefaultExprResolver,
    ctx: &ResolverContext<'_>,
    key: &str,
) -> PatchValue {
    PatchValue::Primitive(OneOrExpr::Value(resolver.resolve_primitive(
        value,
        ctx,
        Some(key),
    )))
}

fn resolve_binding_state(
    binding: &str,
    resolver: &mut DefaultExprResolver,
    ctx: &ResolverContext<'_>,
    key: &str,
) -> PatchValue {
    let expr = BoolOrExpr::Expr(UiExpr::Binding {
        path: binding.to_string(),
    });
    resolve_bool(&expr, resolver, ctx, key)
}

fn resolve_menu_items_json(
    node: &CanonicalMenuNode,
    resolver: &mut DefaultExprResolver,
    ctx: &ResolverContext<'_>,
    key: &str,
) -> JsonValue {
    JsonValue::Array(
        node.items
            .iter()
            .map(|item| {
                let mut out = serde_json::Map::new();
                if let Some(id) = &item.id {
                    out.insert("id".into(), json!(id));
                }
                out.insert(
                    "label".into(),
                    json!(resolver.resolve_string(&item.label, ctx, Some(key))),
                );
                if let Some(disabled) = &item.disabled {
                    out.insert(
                        "disabled".into(),
                        json!(resolver.resolve_bool(disabled, ctx, Some(key))),
                    );
                }
                if let Some(selected) = item.selected {
                    out.insert("selected".into(), json!(selected));
                }
                out.insert(
                    "action".into(),
                    serde_json::to_value(&item.action).unwrap_or(JsonValue::Null),
                );
                JsonValue::Object(out)
            })
            .collect(),
    )
}

fn resolve_continuation_json(
    value: &CollectionContinuation,
    resolver: &mut DefaultExprResolver,
    ctx: &ResolverContext<'_>,
    key: &str,
) -> JsonValue {
    match value {
        CollectionContinuation::Incremental(value) => {
            let mut out = serde_json::Map::new();
            out.insert("kind".into(), json!("incremental"));
            out.insert("binding".into(), json!(value.binding));
            out.insert("initial".into(), json!(value.initial));
            out.insert("step".into(), json!(value.step));
            if let Some(label) = &value.label {
                out.insert(
                    "label".into(),
                    json!(resolver.resolve_string(label, ctx, Some(key))),
                );
            }
            JsonValue::Object(out)
        }
        CollectionContinuation::Remote(value) => {
            let mut out = serde_json::Map::new();
            out.insert("kind".into(), json!("remote"));
            out.insert("hasMore".into(), json!(value.has_more));
            out.insert(
                "loadMore".into(),
                serde_json::to_value(&value.load_more).unwrap_or(JsonValue::Null),
            );
            if let Some(label) = &value.label {
                out.insert(
                    "label".into(),
                    json!(resolver.resolve_string(label, ctx, Some(key))),
                );
            }
            if let Some(loading_label) = &value.loading_label {
                out.insert(
                    "loadingLabel".into(),
                    json!(resolver.resolve_string(loading_label, ctx, Some(key))),
                );
            }
            JsonValue::Object(out)
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use serde_json::json;

    use crate::core::ast::{
        ConditionalNode, NodeBase, OneOrExpr, ScreenNode, TextNode, UiExpr, UiNode,
    };
    use crate::core::ir::lower_patch_ops;
    use crate::core::normalize::normalize_screen;
    use crate::core::resolver::{DefaultExprResolver, ResolverContext};
    use crate::core::runtime::ResolvedRoute;
    use crate::core::state::{MemoryStateStore, StateStore};

    use super::plan_patch_ops;

    #[test]
    fn plans_set_prop_for_reactive_text() {
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

        let route = ResolvedRoute::default();
        let ctx = ResolverContext {
            state: &state,
            route: Some(&route),
            locale: "pt-BR",
        };
        let mut resolver = DefaultExprResolver::default();

        let dirty = ["hero-title".to_string()]
            .into_iter()
            .collect::<BTreeSet<_>>();
        let planned = plan_patch_ops(&screen, &dirty, &mut resolver, &ctx);

        assert_eq!(
            serde_json::to_value(lower_patch_ops(&planned)).unwrap(),
            json!([
                {
                    "o": "sp",
                    "k": "hero-title",
                    "f": "ct",
                    "v": { "v": "Blue Box" }
                }
            ])
        );
    }

    #[test]
    fn plans_replace_node_for_conditional_shape_changes() {
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
            children: vec![UiNode::Conditional(ConditionalNode {
                base: NodeBase {
                    id: Some("upsell".to_string()),
                    meta: None,
                },
                condition: OneOrExpr::Expr(UiExpr::Binding {
                    path: "flags.show_upsell".to_string(),
                }),
                r#then: Box::new(UiNode::Text(TextNode {
                    base: NodeBase {
                        id: Some("upsell-on".to_string()),
                        meta: None,
                    },
                    content: OneOrExpr::Value("On".to_string()),
                    role: None,
                    tone: None,
                    emphasis: None,
                    truncate: None,
                })),
                r#else: Some(Box::new(UiNode::Text(TextNode {
                    base: NodeBase {
                        id: Some("upsell-off".to_string()),
                        meta: None,
                    },
                    content: OneOrExpr::Value("Off".to_string()),
                    role: None,
                    tone: None,
                    emphasis: None,
                    truncate: None,
                }))),
            })],
        })
        .unwrap();

        let mut state = MemoryStateStore::new(None);
        state.set("flags.show_upsell", json!(true));

        let route = ResolvedRoute::default();
        let ctx = ResolverContext {
            state: &state,
            route: Some(&route),
            locale: "pt-BR",
        };
        let mut resolver = DefaultExprResolver::default();
        let dirty = ["upsell".to_string()].into_iter().collect::<BTreeSet<_>>();

        let planned = plan_patch_ops(&screen, &dirty, &mut resolver, &ctx);

        assert_eq!(
            serde_json::to_value(lower_patch_ops(&planned)).unwrap(),
            json!([
                {
                    "o": "rn",
                    "k": "upsell",
                    "n": {
                        "t": "if",
                        "p": {
                            "_h": "n",
                            "_hs": "n",
                            "_k": "upsell",
                            "_r": "c",
                            "_sr": "c",
                            "if": { "b": "flags.show_upsell" },
                            "rf": ["if"],
                            "sd": [{ "b": "flags.show_upsell" }]
                        },
                        "c": [
                            {
                                "t": "text",
                                "p": {
                                    "_k": "upsell-on",
                                    "_r": "s",
                                    "_sr": "s",
                                    "content": { "v": "On" },
                                    "em": "normal",
                                    "role": "body",
                                    "sf": {
                                        "content": "On",
                                        "emphasis": "normal",
                                        "role": "body"
                                    }
                                }
                            },
                            {
                                "t": "text",
                                "p": {
                                    "_k": "upsell-off",
                                    "_r": "s",
                                    "_sr": "s",
                                    "content": { "v": "Off" },
                                    "em": "normal",
                                    "role": "body",
                                    "sf": {
                                        "content": "Off",
                                        "emphasis": "normal",
                                        "role": "body"
                                    }
                                }
                            }
                        ]
                    }
                }
            ])
        );
    }
}
