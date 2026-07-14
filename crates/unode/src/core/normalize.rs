use std::collections::{BTreeMap, BTreeSet};

use serde::Serialize;

use crate::core::ast::*;
use crate::core::ast::{
    BoolOrExpr, NumberOrExpr, Primitive, PrimitiveOrExpr, ScreenNode, StringOrExpr, UiExpr, UiNode,
};
use crate::core::canonical::*;

// ============================================================
// expression helpers
// ============================================================

pub fn is_expr<T>(value: &crate::core::ast::OneOrExpr<T>) -> bool {
    matches!(value, crate::core::ast::OneOrExpr::Expr(_))
}

pub fn is_reactive_expr<T>(value: &crate::core::ast::OneOrExpr<T>) -> bool {
    matches!(
        value,
        crate::core::ast::OneOrExpr::Expr(UiExpr::Binding { .. } | UiExpr::Param { .. })
    )
}

pub fn collapse_string_literal(value: Option<StringOrExpr>) -> Option<StringOrExpr> {
    value.map(|v| match v {
        crate::core::ast::OneOrExpr::Expr(UiExpr::Literal { value }) => {
            crate::core::ast::OneOrExpr::Value(value)
        }
        other => other,
    })
}

pub fn collapse_required_string_literal(value: StringOrExpr) -> StringOrExpr {
    collapse_string_literal(Some(value)).unwrap()
}

pub fn collapse_bool_literal(value: Option<BoolOrExpr>) -> Option<BoolOrExpr> {
    value.map(|v| match v {
        crate::core::ast::OneOrExpr::Expr(UiExpr::Literal { value }) => {
            crate::core::ast::OneOrExpr::Value(value)
        }
        other => other,
    })
}

pub fn collapse_required_bool_literal(value: BoolOrExpr) -> BoolOrExpr {
    collapse_bool_literal(Some(value)).unwrap()
}

pub fn collapse_number_literal(value: Option<NumberOrExpr>) -> Option<NumberOrExpr> {
    value.map(|v| match v {
        crate::core::ast::OneOrExpr::Expr(UiExpr::Literal { value }) => {
            crate::core::ast::OneOrExpr::Value(value)
        }
        other => other,
    })
}

pub fn collapse_required_number_literal(value: NumberOrExpr) -> NumberOrExpr {
    collapse_number_literal(Some(value)).unwrap()
}

pub fn collapse_primitive_literal(value: Option<PrimitiveOrExpr>) -> Option<PrimitiveOrExpr> {
    value.map(|v| match v {
        crate::core::ast::OneOrExpr::Expr(UiExpr::Literal { value }) => {
            crate::core::ast::OneOrExpr::Value(value)
        }
        other => other,
    })
}

pub fn collapse_required_primitive_literal(value: PrimitiveOrExpr) -> PrimitiveOrExpr {
    collapse_primitive_literal(Some(value)).unwrap()
}

// ============================================================
// context
// ============================================================

#[derive(Debug, Clone)]
pub struct NormalizeContext {
    pub path: String,
    pub crumbs: Vec<String>,
}

impl NormalizeContext {
    pub fn root() -> Self {
        Self {
            path: "screen".into(),
            crumbs: vec!["screen".into()],
        }
    }

    pub fn child(&self, segment: &str, index: usize, kind: &str) -> Self {
        Self {
            path: format!("{}.{}[{}]:{}", self.path, segment, index, kind),
            crumbs: self
                .crumbs
                .iter()
                .cloned()
                .chain(std::iter::once(format!("{kind}[{index}]")))
                .collect(),
        }
    }

    pub fn location(&self) -> String {
        format!("\"{}\" ({})", self.crumbs.join(" > "), self.path)
    }
}

// ============================================================
// normalization policy
// ============================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum IdentityRequirement {
    StructuralOk,
    ExplicitStableIdRequired,
}

fn identity_requirement_for(node: &UiNode) -> IdentityRequirement {
    match node {
        // containers / presentational / pure content
        UiNode::Section(_)
        | UiNode::Stack(_)
        | UiNode::Inline(_)
        | UiNode::Grid(_)
        | UiNode::Text(_)
        | UiNode::Value(_)
        | UiNode::Icon(_)
        | UiNode::Badge(_)
        | UiNode::Divider(_)
        | UiNode::Media(_)
        | UiNode::List(_)
        | UiNode::Status(_)
        | UiNode::Empty(_)
        | UiNode::Loading(_)
        | UiNode::Conditional(_) => IdentityRequirement::StructuralOk,

        // stateful / interactive / addressable
        UiNode::Scroll(_)
        | UiNode::Pressable(_)
        | UiNode::Item(_)
        | UiNode::Action(_)
        | UiNode::Actions(_)
        | UiNode::Disclosure(_)
        | UiNode::Menu(_)
        | UiNode::Input(_)
        | UiNode::Form(_)
        | UiNode::Slot(_) => IdentityRequirement::ExplicitStableIdRequired,
    }
}

fn identity_requirement_name(node: &UiNode) -> &'static str {
    kind_of(node)
}

// ============================================================
// normalize helpers
// ============================================================

fn merge_reactivity(values: impl IntoIterator<Item = NodeReactivity>) -> NodeReactivity {
    let mut has_reactive = false;
    for value in values {
        match value {
            NodeReactivity::Conditional => return NodeReactivity::Conditional,
            NodeReactivity::Reactive => has_reactive = true,
            NodeReactivity::Static => {}
        }
    }

    if has_reactive {
        NodeReactivity::Reactive
    } else {
        NodeReactivity::Static
    }
}

fn merge_shape_reactivity(values: impl IntoIterator<Item = ShapeReactivity>) -> ShapeReactivity {
    let mut merged = ShapeReactivity::Static;

    for value in values {
        merged = match (merged, value) {
            (_, ShapeReactivity::ReplaceNode) | (ShapeReactivity::ReplaceNode, _) => {
                ShapeReactivity::ReplaceNode
            }
            (_, ShapeReactivity::ReplaceChildren) | (ShapeReactivity::ReplaceChildren, _) => {
                ShapeReactivity::ReplaceChildren
            }
            (_, ShapeReactivity::Visibility) | (ShapeReactivity::Visibility, _) => {
                ShapeReactivity::Visibility
            }
            _ => ShapeReactivity::Static,
        };
    }

    merged
}

fn combine_reactivity(flags: impl IntoIterator<Item = bool>) -> NodeReactivity {
    if flags.into_iter().any(|v| v) {
        NodeReactivity::Reactive
    } else {
        NodeReactivity::Static
    }
}

fn collect_static_fields_map<T: Serialize>(value: &T) -> BTreeMap<String, Primitive> {
    let mut out = BTreeMap::new();
    let Ok(serde_json::Value::Object(map)) = serde_json::to_value(value) else {
        return out;
    };

    for (key, entry) in map {
        if matches!(
            key.as_str(),
            "id" | "meta"
                | "children"
                | "child"
                | "leading"
                | "primary"
                | "secondary"
                | "trailing"
                | "items"
                | "then"
                | "else"
                | "fallback"
                | "actions"
        ) {
            continue;
        }

        match entry {
            serde_json::Value::Null => {
                out.insert(key, None);
            }
            serde_json::Value::String(_)
            | serde_json::Value::Number(_)
            | serde_json::Value::Bool(_) => {
                out.insert(key, Some(entry));
            }
            serde_json::Value::Array(_) | serde_json::Value::Object(_) => {}
        }
    }

    out
}

fn expr_dependency<T>(value: &crate::core::ast::OneOrExpr<T>) -> Option<StructuralDependency> {
    match value {
        crate::core::ast::OneOrExpr::Expr(UiExpr::Binding { path }) => {
            Some(StructuralDependency::Binding { path: path.clone() })
        }
        crate::core::ast::OneOrExpr::Expr(UiExpr::Param { name }) => {
            Some(StructuralDependency::Param { name: name.clone() })
        }
        _ => None,
    }
}

fn opt_expr_dependency<T>(
    value: &Option<crate::core::ast::OneOrExpr<T>>,
) -> Option<StructuralDependency> {
    value.as_ref().and_then(expr_dependency)
}

fn collect_dependencies(
    values: impl IntoIterator<Item = Option<StructuralDependency>>,
) -> Vec<StructuralDependency> {
    let mut out = BTreeSet::new();
    for value in values.into_iter().flatten() {
        out.insert(value);
    }
    out.into_iter().collect()
}

fn freeze_meta(
    id: Option<&str>,
    self_reactivity: NodeReactivity,
    subtree_reactivity: NodeReactivity,
    reactive_fields: Vec<ReactiveField>,
    shape_reactivity: ShapeReactivity,
    subtree_shape_reactivity: ShapeReactivity,
    structural_dependencies: Vec<StructuralDependency>,
    static_fields: BTreeMap<String, Primitive>,
    ctx: &NormalizeContext,
) -> CanonicalMetadata {
    CanonicalMetadata {
        key: id.unwrap_or(&ctx.path).to_string(),
        reactivity: self_reactivity,
        subtree_reactivity,
        reactive_fields,
        shape_reactivity,
        subtree_shape_reactivity,
        structural_dependencies,
        static_fields,
    }
}

fn register_global_id(
    id: Option<&str>,
    ctx: &NormalizeContext,
    seen_ids: &mut BTreeMap<String, String>,
) -> Result<(), String> {
    if let Some(id) = id {
        if let Some(first) = seen_ids.get(id) {
            return Err(format!(
                "unode duplicate global id \"{}\" at {}; first seen at {}",
                id,
                ctx.location(),
                first
            ));
        }
        seen_ids.insert(id.to_string(), ctx.location());
    }
    Ok(())
}

fn ensure_identity_policy(
    node: &UiNode,
    ctx: &NormalizeContext,
    seen_ids: &mut BTreeMap<String, String>,
) -> Result<(), String> {
    let explicit_id = base_id_of(node);

    if identity_requirement_for(node) == IdentityRequirement::ExplicitStableIdRequired
        && explicit_id.is_none()
    {
        return Err(format!(
            "unode node kind \"{}\" requires an explicit stable id at {}; structural path identity is not enough here",
            identity_requirement_name(node),
            ctx.location(),
        ));
    }

    register_global_id(explicit_id, ctx, seen_ids)
}

fn assert_unique_sibling_keys(
    nodes: &[CanonicalUiNode],
    ctx: &NormalizeContext,
    group: &str,
) -> Result<(), String> {
    let mut seen = BTreeSet::new();

    for (index, node) in nodes.iter().enumerate() {
        if !seen.insert(node.meta().key.clone()) {
            return Err(format!(
                "unode duplicate sibling identity \"{}\" in {} at \"{} > {}[{}]\" ({})",
                node.meta().key,
                group,
                ctx.crumbs.join(" > "),
                node.kind(),
                index,
                ctx.path
            ));
        }
    }

    Ok(())
}

fn normalize_children(
    children: Vec<UiNode>,
    ctx: &NormalizeContext,
    seen_ids: &mut BTreeMap<String, String>,
    segment: &str,
    group: &str,
) -> Result<Vec<CanonicalUiNode>, String> {
    let mut normalized = Vec::with_capacity(children.len());

    for (index, child) in children.into_iter().enumerate() {
        let kind = kind_of(&child);
        normalized.push(normalize_node_with_state(
            child,
            ctx.child(segment, index, kind),
            seen_ids,
        )?);
    }

    assert_unique_sibling_keys(&normalized, ctx, group)?;
    Ok(normalized)
}

fn normalize_action_children(
    children: Vec<ActionNode>,
    ctx: &NormalizeContext,
    seen_ids: &mut BTreeMap<String, String>,
    segment: &str,
    group: &str,
) -> Result<Vec<CanonicalUiNode>, String> {
    let mut normalized = Vec::with_capacity(children.len());

    for (index, child) in children.into_iter().enumerate() {
        normalized.push(normalize_node_with_state(
            UiNode::Action(child),
            ctx.child(segment, index, "action"),
            seen_ids,
        )?);
    }

    assert_unique_sibling_keys(&normalized, ctx, group)?;
    Ok(normalized)
}

fn subtree_from(nodes: &[CanonicalUiNode], self_rx: NodeReactivity) -> NodeReactivity {
    merge_reactivity(
        std::iter::once(self_rx).chain(nodes.iter().map(|n| n.meta().subtree_reactivity)),
    )
}

fn subtree_shape_from(nodes: &[CanonicalUiNode], self_shape: ShapeReactivity) -> ShapeReactivity {
    merge_shape_reactivity(
        std::iter::once(self_shape).chain(nodes.iter().map(|n| n.meta().subtree_shape_reactivity)),
    )
}

fn subtree_from_many<'a>(
    self_rx: NodeReactivity,
    groups: impl IntoIterator<Item = &'a [CanonicalUiNode]>,
) -> NodeReactivity {
    let mut acc = vec![self_rx];
    for group in groups {
        acc.extend(group.iter().map(|n| n.meta().subtree_reactivity));
    }
    merge_reactivity(acc)
}

fn subtree_shape_from_many<'a>(
    self_shape: ShapeReactivity,
    groups: impl IntoIterator<Item = &'a [CanonicalUiNode]>,
) -> ShapeReactivity {
    let mut acc = vec![self_shape];
    for group in groups {
        acc.extend(group.iter().map(|n| n.meta().subtree_shape_reactivity));
    }
    merge_shape_reactivity(acc)
}

// ============================================================
// public entrypoints
// ============================================================

/// Converts a plugin-authored [`ScreenNode`] into a canonical tree.
///
/// Normalization is the first host-owned step after `plugin_render`. It applies
/// defaults, collapses literal expressions, validates global IDs, assigns stable
/// structural keys where allowed, and computes reactivity metadata used by the
/// resolver and patch planner.
///
/// Call this once for each route render before tracking bindings or lowering to
/// IR:
///
/// ```rust
/// use unode::core::dsl as ui;
/// use unode::core::dsl::IntoNode;
/// use unode::core::normalize::normalize_screen;
///
/// let raw = ui::screen()
///     .id("demo")
///     .children([ui::text("Ready").id("demo.ready").into_node()])
///     .build();
/// let canonical = normalize_screen(raw)?;
/// # Ok::<(), String>(())
/// ```
pub fn normalize_screen(mut screen: ScreenNode) -> Result<CanonicalScreen, String> {
    let ctx = NormalizeContext::root();
    let mut seen_ids = BTreeMap::new();

    register_global_id(screen.base.id.as_deref(), &ctx, &mut seen_ids)?;

    screen.title = collapse_string_literal(screen.title);
    screen.subtitle = collapse_string_literal(screen.subtitle);
    let static_fields = collect_static_fields_map(&screen);

    let children = normalize_children(screen.children, &ctx, &mut seen_ids, "c", "children")?;

    let self_rx = combine_reactivity([
        screen.title.as_ref().is_some_and(is_reactive_expr),
        screen.subtitle.as_ref().is_some_and(is_reactive_expr),
    ]);
    let reactive_fields = [
        screen
            .title
            .as_ref()
            .is_some_and(is_reactive_expr)
            .then_some(ReactiveField::Title),
        screen
            .subtitle
            .as_ref()
            .is_some_and(is_reactive_expr)
            .then_some(ReactiveField::Subtitle),
    ]
    .into_iter()
    .flatten()
    .collect();
    let structural_dependencies = collect_dependencies([
        opt_expr_dependency(&screen.title),
        opt_expr_dependency(&screen.subtitle),
    ]);
    let self_shape = ShapeReactivity::Static;

    let subtree_rx = subtree_from(&children, self_rx);
    let subtree_shape = subtree_shape_from(&children, self_shape);

    let screen_id_binding = screen.base.id.clone();
    let screen_id = screen_id_binding.as_deref();

    Ok(CanonicalScreen {
        base: screen.base,
        title: screen.title,
        subtitle: screen.subtitle,
        initial_focus: screen.initial_focus,
        initial_state: screen.initial_state,
        children,
        meta: freeze_meta(
            screen_id,
            self_rx,
            subtree_rx,
            reactive_fields,
            self_shape,
            subtree_shape,
            structural_dependencies,
            static_fields,
            &ctx,
        ),
    })
}

/// Mantida por compatibilidade.
/// Quando usada isoladamente, a verificação de ids globais vale só para a subárvore passada.
pub fn normalize_node(node: UiNode, ctx: NormalizeContext) -> Result<CanonicalUiNode, String> {
    let mut seen_ids = BTreeMap::new();
    normalize_node_with_state(node, ctx, &mut seen_ids)
}

// ============================================================
// internal recursive normalization
// ============================================================

fn normalize_node_with_state(
    node: UiNode,
    ctx: NormalizeContext,
    seen_ids: &mut BTreeMap<String, String>,
) -> Result<CanonicalUiNode, String> {
    match node {
        UiNode::Section(mut node) => {
            let wrapper = UiNode::Section(node.clone());
            ensure_identity_policy(&wrapper, &ctx, seen_ids)?;

            node.title = collapse_string_literal(node.title);
            node.description = collapse_string_literal(node.description);
            node.role.get_or_insert(ContainerRole::Section);
            let static_fields = collect_static_fields_map(&node);
            let node_id = node.base.id.clone();

            let children = normalize_children(node.children, &ctx, seen_ids, "c", "children")?;
            let self_rx = combine_reactivity([
                node.title.as_ref().map(is_reactive_expr).unwrap_or(false),
                node.description
                    .as_ref()
                    .map(is_reactive_expr)
                    .unwrap_or(false),
            ]);
            let subtree_rx = subtree_from(&children, self_rx);
            let reactive_fields = [
                node.title
                    .as_ref()
                    .is_some_and(is_reactive_expr)
                    .then_some(ReactiveField::Title),
                node.description
                    .as_ref()
                    .is_some_and(is_reactive_expr)
                    .then_some(ReactiveField::Description),
            ]
            .into_iter()
            .flatten()
            .collect();
            let self_shape = ShapeReactivity::Static;
            let subtree_shape = subtree_shape_from(&children, self_shape);
            let structural_dependencies = collect_dependencies([
                opt_expr_dependency(&node.title),
                opt_expr_dependency(&node.description),
            ]);

            Ok(CanonicalUiNode::Section(CanonicalSectionNode {
                base: node.base,
                role: node.role,
                title: node.title,
                description: node.description,
                children,
                meta: freeze_meta(
                    node_id.as_deref(),
                    self_rx,
                    subtree_rx,
                    reactive_fields,
                    self_shape,
                    subtree_shape,
                    structural_dependencies,
                    static_fields,
                    &ctx,
                ),
            }))
        }

        UiNode::Text(mut node) => {
            let wrapper = UiNode::Text(node.clone());
            ensure_identity_policy(&wrapper, &ctx, seen_ids)?;

            node.content = collapse_required_string_literal(node.content);
            node.role.get_or_insert(TextRole::Body);
            node.emphasis.get_or_insert(TextEmphasis::Normal);
            let static_fields = collect_static_fields_map(&node);
            let node_id = node.base.id.clone();

            let self_rx = combine_reactivity([is_reactive_expr(&node.content)]);
            let reactive_fields = is_reactive_expr(&node.content)
                .then_some(ReactiveField::Content)
                .into_iter()
                .collect();
            let self_shape = ShapeReactivity::Static;
            let structural_dependencies = collect_dependencies([expr_dependency(&node.content)]);

            Ok(CanonicalUiNode::Text(CanonicalTextNode {
                base: node.base,
                content: node.content,
                role: node.role,
                tone: node.tone,
                emphasis: node.emphasis,
                truncate: node.truncate,
                meta: freeze_meta(
                    node_id.as_deref(),
                    self_rx,
                    self_rx,
                    reactive_fields,
                    self_shape,
                    self_shape,
                    structural_dependencies,
                    static_fields,
                    &ctx,
                ),
            }))
        }

        UiNode::Stack(mut node) => {
            let wrapper = UiNode::Stack(node.clone());
            ensure_identity_policy(&wrapper, &ctx, seen_ids)?;

            node.gap.get_or_insert(Gap::Md);
            let static_fields = collect_static_fields_map(&node);
            let node_id = node.base.id.clone();

            let children = normalize_children(node.children, &ctx, seen_ids, "c", "children")?;
            let self_rx = NodeReactivity::Static;
            let subtree_rx = subtree_from(&children, self_rx);
            let self_shape = ShapeReactivity::Static;
            let subtree_shape = subtree_shape_from(&children, self_shape);

            Ok(CanonicalUiNode::Stack(CanonicalStackNode {
                base: node.base,
                gap: node.gap,
                children,
                meta: freeze_meta(
                    node_id.as_deref(),
                    self_rx,
                    subtree_rx,
                    vec![],
                    self_shape,
                    subtree_shape,
                    vec![],
                    static_fields,
                    &ctx,
                ),
            }))
        }

        UiNode::Inline(mut node) => {
            let wrapper = UiNode::Inline(node.clone());
            ensure_identity_policy(&wrapper, &ctx, seen_ids)?;

            node.gap.get_or_insert(Gap::Sm);
            node.wrap.get_or_insert(false);
            node.align.get_or_insert(Align::Start);
            let static_fields = collect_static_fields_map(&node);
            let node_id = node.base.id.clone();

            let children = normalize_children(node.children, &ctx, seen_ids, "c", "children")?;
            let self_rx = NodeReactivity::Static;
            let subtree_rx = subtree_from(&children, self_rx);
            let self_shape = ShapeReactivity::Static;
            let subtree_shape = subtree_shape_from(&children, self_shape);

            Ok(CanonicalUiNode::Inline(CanonicalInlineNode {
                base: node.base,
                gap: node.gap,
                wrap: node.wrap,
                align: node.align,
                children,
                meta: freeze_meta(
                    node_id.as_deref(),
                    self_rx,
                    subtree_rx,
                    vec![],
                    self_shape,
                    subtree_shape,
                    vec![],
                    static_fields,
                    &ctx,
                ),
            }))
        }

        UiNode::Grid(mut node) => {
            let wrapper = UiNode::Grid(node.clone());
            ensure_identity_policy(&wrapper, &ctx, seen_ids)?;

            node.gap.get_or_insert(Gap::Md);

            if node.columns.is_none() {
                if let Some(max) = node.max_columns {
                    node.columns = Some(ResponsiveGridColumns {
                        base: Some(max),
                        sm: None,
                        md: None,
                        lg: None,
                        xl: None,
                    });
                }
            }

            node.continuation = match node.continuation {
                Some(CollectionContinuation::Incremental(mut continuation)) => {
                    continuation.label = collapse_string_literal(continuation.label);
                    Some(CollectionContinuation::Incremental(continuation))
                }
                Some(CollectionContinuation::Remote(mut continuation)) => {
                    continuation.label = collapse_string_literal(continuation.label);
                    continuation.loading_label =
                        collapse_string_literal(continuation.loading_label);
                    Some(CollectionContinuation::Remote(continuation))
                }
                None => None,
            };
            let static_fields = collect_static_fields_map(&node);
            let node_id = node.base.id.clone();

            let children = normalize_children(node.children, &ctx, seen_ids, "c", "children")?;
            let self_rx = combine_reactivity([
                match &node.continuation {
                    Some(CollectionContinuation::Incremental(value)) => {
                        value.label.as_ref().map(is_reactive_expr).unwrap_or(false)
                    }
                    _ => false,
                },
                match &node.continuation {
                    Some(CollectionContinuation::Remote(value)) => {
                        value.label.as_ref().map(is_reactive_expr).unwrap_or(false)
                    }
                    _ => false,
                },
                match &node.continuation {
                    Some(CollectionContinuation::Remote(value)) => value
                        .loading_label
                        .as_ref()
                        .map(is_reactive_expr)
                        .unwrap_or(false),
                    _ => false,
                },
            ]);
            let subtree_rx = subtree_from(&children, self_rx);
            let reactive_fields = (self_rx != NodeReactivity::Static)
                .then_some(ReactiveField::Continuation)
                .into_iter()
                .collect();
            let self_shape = ShapeReactivity::Static;
            let subtree_shape = subtree_shape_from(&children, self_shape);
            let structural_dependencies = collect_dependencies([
                match &node.continuation {
                    Some(CollectionContinuation::Incremental(value)) => {
                        opt_expr_dependency(&value.label)
                    }
                    _ => None,
                },
                match &node.continuation {
                    Some(CollectionContinuation::Remote(value)) => {
                        opt_expr_dependency(&value.label)
                    }
                    _ => None,
                },
                match &node.continuation {
                    Some(CollectionContinuation::Remote(value)) => {
                        opt_expr_dependency(&value.loading_label)
                    }
                    _ => None,
                },
            ]);

            Ok(CanonicalUiNode::Grid(CanonicalGridNode {
                base: node.base,
                max_columns: node.max_columns,
                columns: node.columns,
                gap: node.gap,
                continuation: node.continuation,
                children,
                meta: freeze_meta(
                    node_id.as_deref(),
                    self_rx,
                    subtree_rx,
                    reactive_fields,
                    self_shape,
                    subtree_shape,
                    structural_dependencies,
                    static_fields,
                    &ctx,
                ),
            }))
        }

        UiNode::Scroll(node) => {
            let wrapper = UiNode::Scroll(node.clone());
            ensure_identity_policy(&wrapper, &ctx, seen_ids)?;
            let static_fields = collect_static_fields_map(&node);
            let node_id = node.base.id.clone();

            let children = normalize_children(node.children, &ctx, seen_ids, "c", "children")?;
            let self_rx = NodeReactivity::Static;
            let subtree_rx = subtree_from(&children, self_rx);
            let self_shape = ShapeReactivity::Static;
            let subtree_shape = subtree_shape_from(&children, self_shape);

            Ok(CanonicalUiNode::Scroll(CanonicalScrollNode {
                base: node.base,
                children,
                meta: freeze_meta(
                    node_id.as_deref(),
                    self_rx,
                    subtree_rx,
                    vec![],
                    self_shape,
                    subtree_shape,
                    vec![],
                    static_fields,
                    &ctx,
                ),
            }))
        }

        UiNode::Value(mut node) => {
            let wrapper = UiNode::Value(node.clone());
            ensure_identity_policy(&wrapper, &ctx, seen_ids)?;

            node.value = collapse_required_primitive_literal(node.value);
            let static_fields = collect_static_fields_map(&node);
            let node_id = node.base.id.clone();
            let self_rx = combine_reactivity([is_reactive_expr(&node.value)]);
            let reactive_fields = is_reactive_expr(&node.value)
                .then_some(ReactiveField::Value)
                .into_iter()
                .collect();
            let self_shape = ShapeReactivity::Static;
            let structural_dependencies = collect_dependencies([expr_dependency(&node.value)]);

            Ok(CanonicalUiNode::Value(CanonicalValueNode {
                base: node.base,
                value: node.value,
                format: node.format,
                currency_code: node.currency_code,
                role: node.role,
                tone: node.tone,
                meta: freeze_meta(
                    node_id.as_deref(),
                    self_rx,
                    self_rx,
                    reactive_fields,
                    self_shape,
                    self_shape,
                    structural_dependencies,
                    static_fields,
                    &ctx,
                ),
            }))
        }

        UiNode::Icon(mut node) => {
            let wrapper = UiNode::Icon(node.clone());
            ensure_identity_policy(&wrapper, &ctx, seen_ids)?;

            node.tone.get_or_insert(Tone::Default);
            let static_fields = collect_static_fields_map(&node);
            let node_id = node.base.id.clone();

            Ok(CanonicalUiNode::Icon(CanonicalIconNode {
                base: node.base,
                name: node.name,
                label: node.label,
                tone: node.tone,
                meta: freeze_meta(
                    node_id.as_deref(),
                    NodeReactivity::Static,
                    NodeReactivity::Static,
                    vec![],
                    ShapeReactivity::Static,
                    ShapeReactivity::Static,
                    vec![],
                    static_fields,
                    &ctx,
                ),
            }))
        }

        UiNode::Badge(mut node) => {
            let wrapper = UiNode::Badge(node.clone());
            ensure_identity_policy(&wrapper, &ctx, seen_ids)?;

            node.label = collapse_required_string_literal(node.label);
            let static_fields = collect_static_fields_map(&node);
            let node_id = node.base.id.clone();
            let self_rx = combine_reactivity([is_reactive_expr(&node.label)]);
            let reactive_fields = is_reactive_expr(&node.label)
                .then_some(ReactiveField::Label)
                .into_iter()
                .collect();
            let self_shape = ShapeReactivity::Static;
            let structural_dependencies = collect_dependencies([expr_dependency(&node.label)]);

            Ok(CanonicalUiNode::Badge(CanonicalBadgeNode {
                base: node.base,
                label: node.label,
                tone: node.tone,
                meta: freeze_meta(
                    node_id.as_deref(),
                    self_rx,
                    self_rx,
                    reactive_fields,
                    self_shape,
                    self_shape,
                    structural_dependencies,
                    static_fields,
                    &ctx,
                ),
            }))
        }

        UiNode::Divider(mut node) => {
            let wrapper = UiNode::Divider(node.clone());
            ensure_identity_policy(&wrapper, &ctx, seen_ids)?;

            node.label = collapse_string_literal(node.label);
            let static_fields = collect_static_fields_map(&node);
            let node_id = node.base.id.clone();
            let self_rx = node.label.as_ref().map(is_reactive_expr).unwrap_or(false);
            let reactive_fields = node
                .label
                .as_ref()
                .is_some_and(is_reactive_expr)
                .then_some(ReactiveField::Label)
                .into_iter()
                .collect();
            let self_shape = ShapeReactivity::Static;
            let structural_dependencies = collect_dependencies([opt_expr_dependency(&node.label)]);

            Ok(CanonicalUiNode::Divider(CanonicalDividerNode {
                base: node.base,
                label: node.label,
                meta: freeze_meta(
                    node_id.as_deref(),
                    combine_reactivity([self_rx]),
                    combine_reactivity([self_rx]),
                    reactive_fields,
                    self_shape,
                    self_shape,
                    structural_dependencies,
                    static_fields,
                    &ctx,
                ),
            }))
        }

        UiNode::Media(mut node) => {
            let wrapper = UiNode::Media(node.clone());
            ensure_identity_policy(&wrapper, &ctx, seen_ids)?;

            node.expandable.get_or_insert(false);
            let static_fields = collect_static_fields_map(&node);
            let node_id = node.base.id.clone();

            Ok(CanonicalUiNode::Media(CanonicalMediaNode {
                base: node.base,
                r#ref: node.r#ref,
                media_kind: node.media_kind,
                alt: node.alt,
                aspect_ratio: node.aspect_ratio,
                expandable: node.expandable,
                meta: freeze_meta(
                    node_id.as_deref(),
                    NodeReactivity::Static,
                    NodeReactivity::Static,
                    vec![],
                    ShapeReactivity::Static,
                    ShapeReactivity::Static,
                    vec![],
                    static_fields,
                    &ctx,
                ),
            }))
        }

        UiNode::Pressable(mut node) => {
            let wrapper = UiNode::Pressable(node.clone());
            ensure_identity_policy(&wrapper, &ctx, seen_ids)?;

            node.label = collapse_string_literal(node.label);
            let static_fields = collect_static_fields_map(&node);
            let node_id = node.base.id.clone();

            let child_kind = kind_of(&node.child);
            let child = normalize_node_with_state(
                *node.child,
                ctx.child("child", 0, child_kind),
                seen_ids,
            )?;

            let self_rx =
                combine_reactivity([node.label.as_ref().map(is_reactive_expr).unwrap_or(false)]);
            let subtree_rx = merge_reactivity(
                std::iter::once(self_rx).chain(std::iter::once(child.meta().subtree_reactivity)),
            );
            let reactive_fields = node
                .label
                .as_ref()
                .is_some_and(is_reactive_expr)
                .then_some(ReactiveField::Label)
                .into_iter()
                .collect();
            let self_shape = ShapeReactivity::Static;
            let subtree_shape = merge_shape_reactivity(
                std::iter::once(self_shape)
                    .chain(std::iter::once(child.meta().subtree_shape_reactivity)),
            );
            let structural_dependencies = collect_dependencies([opt_expr_dependency(&node.label)]);

            Ok(CanonicalUiNode::Pressable(CanonicalPressableNode {
                base: node.base,
                child: Box::new(child),
                action: node.action,
                label: node.label,
                meta: freeze_meta(
                    node_id.as_deref(),
                    self_rx,
                    subtree_rx,
                    reactive_fields,
                    self_shape,
                    subtree_shape,
                    structural_dependencies,
                    static_fields,
                    &ctx,
                ),
            }))
        }

        UiNode::Item(node) => {
            if let Some(first) = seen_ids.get(&node.id) {
                return Err(format!(
                    "unode duplicate global id \"{}\" at {}; first seen at {}",
                    node.id,
                    ctx.location(),
                    first
                ));
            }
            seen_ids.insert(node.id.clone(), ctx.location());
            let static_fields = collect_static_fields_map(&node);
            let node_id = node.id.clone();

            let leading = normalize_children(node.leading, &ctx, seen_ids, "leading", "leading")?;
            let primary = normalize_children(node.primary, &ctx, seen_ids, "primary", "primary")?;
            let secondary =
                normalize_children(node.secondary, &ctx, seen_ids, "secondary", "secondary")?;
            let trailing =
                normalize_children(node.trailing, &ctx, seen_ids, "trailing", "trailing")?;

            let self_rx = NodeReactivity::Static;
            let subtree_rx = subtree_from_many(
                self_rx,
                [
                    leading.as_slice(),
                    primary.as_slice(),
                    secondary.as_slice(),
                    trailing.as_slice(),
                ],
            );
            let self_shape = ShapeReactivity::Static;
            let subtree_shape = subtree_shape_from_many(
                self_shape,
                [
                    leading.as_slice(),
                    primary.as_slice(),
                    secondary.as_slice(),
                    trailing.as_slice(),
                ],
            );

            Ok(CanonicalUiNode::Item(CanonicalItemNode {
                id: node.id,
                meta_map: node.meta,
                leading,
                primary,
                secondary,
                trailing,
                action: node.action,
                meta: freeze_meta(
                    Some(node_id.as_str()),
                    self_rx,
                    subtree_rx,
                    vec![],
                    self_shape,
                    subtree_shape,
                    vec![],
                    static_fields,
                    &ctx,
                ),
            }))
        }

        UiNode::List(mut node) => {
            let wrapper = UiNode::List(node.clone());
            ensure_identity_policy(&wrapper, &ctx, seen_ids)?;

            node.density.get_or_insert(ListDensity::Normal);
            node.continuation = match node.continuation {
                Some(CollectionContinuation::Incremental(mut continuation)) => {
                    continuation.label = collapse_string_literal(continuation.label);
                    Some(CollectionContinuation::Incremental(continuation))
                }
                Some(CollectionContinuation::Remote(mut continuation)) => {
                    continuation.label = collapse_string_literal(continuation.label);
                    continuation.loading_label =
                        collapse_string_literal(continuation.loading_label);
                    Some(CollectionContinuation::Remote(continuation))
                }
                None => None,
            };
            let static_fields = collect_static_fields_map(&node);
            let node_id = node.base.id.clone();

            let mut items = Vec::with_capacity(node.items.len());
            for (index, item) in node.items.into_iter().enumerate() {
                items.push(normalize_node_with_state(
                    UiNode::Item(item),
                    ctx.child("item", index, "item"),
                    seen_ids,
                )?);
            }
            assert_unique_sibling_keys(&items, &ctx, "items")?;

            let self_rx = combine_reactivity([
                match &node.continuation {
                    Some(CollectionContinuation::Incremental(value)) => {
                        value.label.as_ref().map(is_reactive_expr).unwrap_or(false)
                    }
                    _ => false,
                },
                match &node.continuation {
                    Some(CollectionContinuation::Remote(value)) => {
                        value.label.as_ref().map(is_reactive_expr).unwrap_or(false)
                    }
                    _ => false,
                },
                match &node.continuation {
                    Some(CollectionContinuation::Remote(value)) => value
                        .loading_label
                        .as_ref()
                        .map(is_reactive_expr)
                        .unwrap_or(false),
                    _ => false,
                },
            ]);
            let subtree_rx = subtree_from(&items, self_rx);
            let reactive_fields = (self_rx != NodeReactivity::Static)
                .then_some(ReactiveField::Continuation)
                .into_iter()
                .collect();
            let self_shape = ShapeReactivity::Static;
            let subtree_shape = subtree_shape_from(&items, self_shape);
            let structural_dependencies = collect_dependencies([
                match &node.continuation {
                    Some(CollectionContinuation::Incremental(value)) => {
                        opt_expr_dependency(&value.label)
                    }
                    _ => None,
                },
                match &node.continuation {
                    Some(CollectionContinuation::Remote(value)) => {
                        opt_expr_dependency(&value.label)
                    }
                    _ => None,
                },
                match &node.continuation {
                    Some(CollectionContinuation::Remote(value)) => {
                        opt_expr_dependency(&value.loading_label)
                    }
                    _ => None,
                },
            ]);

            Ok(CanonicalUiNode::List(CanonicalListNode {
                base: node.base,
                items: items
                    .into_iter()
                    .map(|n| match n {
                        CanonicalUiNode::Item(item) => item,
                        _ => unreachable!(),
                    })
                    .collect(),
                density: node.density,
                continuation: node.continuation,
                meta: freeze_meta(
                    node_id.as_deref(),
                    self_rx,
                    subtree_rx,
                    reactive_fields,
                    self_shape,
                    subtree_shape,
                    structural_dependencies,
                    static_fields,
                    &ctx,
                ),
            }))
        }

        UiNode::Action(mut node) => {
            let wrapper = UiNode::Action(node.clone());
            ensure_identity_policy(&wrapper, &ctx, seen_ids)?;

            node.label = collapse_required_string_literal(node.label);
            node.intent.get_or_insert(ActionIntent::Primary);
            node.variant.get_or_insert(ActionVariant::Button);
            node.disabled = collapse_bool_literal(node.disabled);
            node.disabled.get_or_insert(OneOrExpr::Value(false));
            let static_fields = collect_static_fields_map(&node);
            let node_id = node.base.id.clone();

            let self_rx = combine_reactivity([
                is_reactive_expr(&node.label),
                node.disabled
                    .as_ref()
                    .map(is_reactive_expr)
                    .unwrap_or(false),
            ]);
            let reactive_fields = [
                is_reactive_expr(&node.label).then_some(ReactiveField::Label),
                node.disabled
                    .as_ref()
                    .is_some_and(is_reactive_expr)
                    .then_some(ReactiveField::Disabled),
            ]
            .into_iter()
            .flatten()
            .collect();
            let self_shape = ShapeReactivity::Static;
            let structural_dependencies = collect_dependencies([
                expr_dependency(&node.label),
                opt_expr_dependency(&node.disabled),
            ]);

            Ok(CanonicalUiNode::Action(CanonicalActionNode {
                base: node.base,
                label: node.label,
                action: node.action,
                intent: node.intent,
                variant: node.variant,
                leading_icon: node.leading_icon,
                disabled: node.disabled,
                meta: freeze_meta(
                    node_id.as_deref(),
                    self_rx,
                    self_rx,
                    reactive_fields,
                    self_shape,
                    self_shape,
                    structural_dependencies,
                    static_fields,
                    &ctx,
                ),
            }))
        }

        UiNode::Actions(mut node) => {
            let wrapper = UiNode::Actions(node.clone());
            ensure_identity_policy(&wrapper, &ctx, seen_ids)?;

            node.align.get_or_insert(Align::Start);
            let static_fields = collect_static_fields_map(&node);
            let node_id = node.base.id.clone();

            let children =
                normalize_action_children(node.children, &ctx, seen_ids, "action", "actions")?;
            let self_rx = NodeReactivity::Static;
            let subtree_rx = subtree_from(&children, self_rx);
            let self_shape = ShapeReactivity::Static;
            let subtree_shape = subtree_shape_from(&children, self_shape);

            Ok(CanonicalUiNode::Actions(CanonicalActionsNode {
                base: node.base,
                align: node.align,
                children: children
                    .into_iter()
                    .map(|n| match n {
                        CanonicalUiNode::Action(action) => action,
                        _ => unreachable!(),
                    })
                    .collect(),
                meta: freeze_meta(
                    node_id.as_deref(),
                    self_rx,
                    subtree_rx,
                    vec![],
                    self_shape,
                    subtree_shape,
                    vec![],
                    static_fields,
                    &ctx,
                ),
            }))
        }

        UiNode::Disclosure(mut node) => {
            let wrapper = UiNode::Disclosure(node.clone());
            ensure_identity_policy(&wrapper, &ctx, seen_ids)?;

            node.label = collapse_required_string_literal(node.label);
            node.label_expanded = collapse_string_literal(node.label_expanded);
            let static_fields = collect_static_fields_map(&node);
            let node_id = node.base.id.clone();

            let children = normalize_children(node.children, &ctx, seen_ids, "c", "children")?;

            let self_rx = combine_reactivity([
                is_reactive_expr(&node.label),
                node.label_expanded
                    .as_ref()
                    .map(is_reactive_expr)
                    .unwrap_or(false),
                true,
            ]);
            let subtree_rx = subtree_from(&children, self_rx);
            let reactive_fields = vec![
                Some(ReactiveField::BindingState),
                is_reactive_expr(&node.label).then_some(ReactiveField::Label),
                node.label_expanded
                    .as_ref()
                    .is_some_and(is_reactive_expr)
                    .then_some(ReactiveField::LabelExpanded),
            ]
            .into_iter()
            .flatten()
            .collect();
            let self_shape = ShapeReactivity::Visibility;
            let subtree_shape = subtree_shape_from(&children, self_shape);
            let structural_dependencies = collect_dependencies([
                Some(StructuralDependency::Binding {
                    path: node.binding.clone(),
                }),
                expr_dependency(&node.label),
                opt_expr_dependency(&node.label_expanded),
            ]);

            Ok(CanonicalUiNode::Disclosure(CanonicalDisclosureNode {
                base: node.base,
                binding: node.binding,
                label: node.label,
                label_expanded: node.label_expanded,
                children,
                meta: freeze_meta(
                    node_id.as_deref(),
                    self_rx,
                    subtree_rx,
                    reactive_fields,
                    self_shape,
                    subtree_shape,
                    structural_dependencies,
                    static_fields,
                    &ctx,
                ),
            }))
        }

        UiNode::Menu(mut node) => {
            let wrapper = UiNode::Menu(node.clone());
            ensure_identity_policy(&wrapper, &ctx, seen_ids)?;

            node.label = collapse_required_string_literal(node.label);
            node.intent.get_or_insert(ActionIntent::Primary);
            node.align.get_or_insert(Align::Start);
            node.items = node
                .items
                .into_iter()
                .map(|mut item| {
                    item.label = collapse_required_string_literal(item.label);
                    item.disabled = collapse_bool_literal(item.disabled);
                    item.selected.get_or_insert(false);
                    item
                })
                .collect();
            let static_fields = collect_static_fields_map(&node);
            let node_id = node.base.id.clone();

            let self_rx = combine_reactivity(std::iter::once(is_reactive_expr(&node.label)).chain(
                node.items.iter().map(|item| {
                    is_reactive_expr(&item.label)
                        || item
                            .disabled
                            .as_ref()
                            .map(is_reactive_expr)
                            .unwrap_or(false)
                }),
            ));
            let reactive_fields = [
                is_reactive_expr(&node.label).then_some(ReactiveField::Label),
                node.items
                    .iter()
                    .any(|item| {
                        is_reactive_expr(&item.label)
                            || item.disabled.as_ref().is_some_and(is_reactive_expr)
                    })
                    .then_some(ReactiveField::MenuItems),
            ]
            .into_iter()
            .flatten()
            .collect();
            let self_shape = ShapeReactivity::Static;
            let structural_dependencies =
                collect_dependencies(std::iter::once(expr_dependency(&node.label)).chain(
                    node.items.iter().flat_map(|item| {
                        [
                            expr_dependency(&item.label),
                            opt_expr_dependency(&item.disabled),
                        ]
                    }),
                ));

            Ok(CanonicalUiNode::Menu(CanonicalMenuNode {
                base: node.base,
                label: node.label,
                items: node.items,
                intent: node.intent,
                align: node.align,
                meta: freeze_meta(
                    node_id.as_deref(),
                    self_rx,
                    self_rx,
                    reactive_fields,
                    self_shape,
                    self_shape,
                    structural_dependencies,
                    static_fields,
                    &ctx,
                ),
            }))
        }

        UiNode::Input(mut node) => {
            let wrapper = UiNode::Input(node.clone());
            ensure_identity_policy(&wrapper, &ctx, seen_ids)?;

            node.label = collapse_required_string_literal(node.label);
            node.placeholder = collapse_string_literal(node.placeholder);
            node.help_text = collapse_string_literal(node.help_text);
            node.required.get_or_insert(false);
            node.disabled = collapse_bool_literal(node.disabled);
            node.disabled.get_or_insert(OneOrExpr::Value(false));
            let static_fields = collect_static_fields_map(&node);
            let node_id = node.base.id.clone();

            let self_rx = combine_reactivity([
                is_reactive_expr(&node.label),
                node.value.as_ref().map(is_reactive_expr).unwrap_or(false),
                node.placeholder
                    .as_ref()
                    .map(is_reactive_expr)
                    .unwrap_or(false),
                node.help_text
                    .as_ref()
                    .map(is_reactive_expr)
                    .unwrap_or(false),
                node.disabled
                    .as_ref()
                    .map(is_reactive_expr)
                    .unwrap_or(false),
            ]);
            let reactive_fields = [
                is_reactive_expr(&node.label).then_some(ReactiveField::Label),
                node.value
                    .as_ref()
                    .is_some_and(is_reactive_expr)
                    .then_some(ReactiveField::Value),
                node.placeholder
                    .as_ref()
                    .is_some_and(is_reactive_expr)
                    .then_some(ReactiveField::Placeholder),
                node.help_text
                    .as_ref()
                    .is_some_and(is_reactive_expr)
                    .then_some(ReactiveField::HelpText),
                node.disabled
                    .as_ref()
                    .is_some_and(is_reactive_expr)
                    .then_some(ReactiveField::Disabled),
            ]
            .into_iter()
            .flatten()
            .collect();
            let self_shape = ShapeReactivity::Static;
            let structural_dependencies = collect_dependencies([
                expr_dependency(&node.label),
                opt_expr_dependency(&node.value),
                opt_expr_dependency(&node.placeholder),
                opt_expr_dependency(&node.help_text),
                opt_expr_dependency(&node.disabled),
            ]);

            Ok(CanonicalUiNode::Input(CanonicalInputNode {
                base: node.base,
                name: node.name,
                input_kind: node.input_kind,
                label: node.label,
                value: node.value,
                placeholder: node.placeholder,
                help_text: node.help_text,
                required: node.required,
                disabled: node.disabled,
                options: node.options,
                constraints: node.constraints,
                meta: freeze_meta(
                    node_id.as_deref(),
                    self_rx,
                    self_rx,
                    reactive_fields,
                    self_shape,
                    self_shape,
                    structural_dependencies,
                    static_fields,
                    &ctx,
                ),
            }))
        }

        UiNode::Form(node) => {
            let wrapper = UiNode::Form(node.clone());
            ensure_identity_policy(&wrapper, &ctx, seen_ids)?;
            let static_fields = collect_static_fields_map(&node);
            let node_id = node.base.id.clone();

            let children = normalize_children(node.children, &ctx, seen_ids, "c", "children")?;
            let self_rx = NodeReactivity::Static;
            let subtree_rx = subtree_from(&children, self_rx);
            let self_shape = ShapeReactivity::Static;
            let subtree_shape = subtree_shape_from(&children, self_shape);

            Ok(CanonicalUiNode::Form(CanonicalFormNode {
                base: node.base,
                name: node.name,
                children,
                submit: node.submit,
                meta: freeze_meta(
                    node_id.as_deref(),
                    self_rx,
                    subtree_rx,
                    vec![],
                    self_shape,
                    subtree_shape,
                    vec![],
                    static_fields,
                    &ctx,
                ),
            }))
        }

        UiNode::Status(mut node) => {
            let wrapper = UiNode::Status(node.clone());
            ensure_identity_policy(&wrapper, &ctx, seen_ids)?;

            node.title = collapse_string_literal(node.title);
            node.title
                .get_or_insert(OneOrExpr::Value("Status".to_string()));
            node.message = collapse_required_string_literal(node.message);
            let static_fields = collect_static_fields_map(&node);
            let node_id = node.base.id.clone();

            let actions =
                normalize_action_children(node.actions, &ctx, seen_ids, "action", "actions")?;

            let self_rx = combine_reactivity([
                node.title.as_ref().map(is_reactive_expr).unwrap_or(false),
                is_reactive_expr(&node.message),
            ]);
            let subtree_rx = subtree_from(&actions, self_rx);
            let reactive_fields = [
                node.title
                    .as_ref()
                    .is_some_and(is_reactive_expr)
                    .then_some(ReactiveField::Title),
                is_reactive_expr(&node.message).then_some(ReactiveField::Message),
            ]
            .into_iter()
            .flatten()
            .collect();
            let self_shape = ShapeReactivity::Static;
            let subtree_shape = subtree_shape_from(&actions, self_shape);
            let structural_dependencies = collect_dependencies([
                opt_expr_dependency(&node.title),
                expr_dependency(&node.message),
            ]);

            Ok(CanonicalUiNode::Status(CanonicalStatusNode {
                base: node.base,
                severity: node.severity,
                title: node.title,
                message: node.message,
                actions: actions
                    .into_iter()
                    .map(|n| match n {
                        CanonicalUiNode::Action(action) => action,
                        _ => unreachable!(),
                    })
                    .collect(),
                meta: freeze_meta(
                    node_id.as_deref(),
                    self_rx,
                    subtree_rx,
                    reactive_fields,
                    self_shape,
                    subtree_shape,
                    structural_dependencies,
                    static_fields,
                    &ctx,
                ),
            }))
        }

        UiNode::Empty(mut node) => {
            let wrapper = UiNode::Empty(node.clone());
            ensure_identity_policy(&wrapper, &ctx, seen_ids)?;

            node.title = collapse_required_string_literal(node.title);
            node.message = collapse_string_literal(node.message);
            let static_fields = collect_static_fields_map(&node);
            let node_id = node.base.id.clone();

            let actions =
                normalize_action_children(node.actions, &ctx, seen_ids, "action", "actions")?;

            let self_rx = combine_reactivity([
                is_reactive_expr(&node.title),
                node.message.as_ref().map(is_reactive_expr).unwrap_or(false),
            ]);
            let subtree_rx = subtree_from(&actions, self_rx);
            let reactive_fields = [
                is_reactive_expr(&node.title).then_some(ReactiveField::Title),
                node.message
                    .as_ref()
                    .is_some_and(is_reactive_expr)
                    .then_some(ReactiveField::Message),
            ]
            .into_iter()
            .flatten()
            .collect();
            let self_shape = ShapeReactivity::Static;
            let subtree_shape = subtree_shape_from(&actions, self_shape);
            let structural_dependencies = collect_dependencies([
                expr_dependency(&node.title),
                opt_expr_dependency(&node.message),
            ]);

            Ok(CanonicalUiNode::Empty(CanonicalEmptyStateNode {
                base: node.base,
                icon: node.icon,
                title: node.title,
                message: node.message,
                actions: actions
                    .into_iter()
                    .map(|n| match n {
                        CanonicalUiNode::Action(action) => action,
                        _ => unreachable!(),
                    })
                    .collect(),
                meta: freeze_meta(
                    node_id.as_deref(),
                    self_rx,
                    subtree_rx,
                    reactive_fields,
                    self_shape,
                    subtree_shape,
                    structural_dependencies,
                    static_fields,
                    &ctx,
                ),
            }))
        }

        UiNode::Loading(mut node) => {
            let wrapper = UiNode::Loading(node.clone());
            ensure_identity_policy(&wrapper, &ctx, seen_ids)?;

            node.label = collapse_string_literal(node.label);
            node.progress = collapse_number_literal(node.progress);
            let static_fields = collect_static_fields_map(&node);
            let node_id = node.base.id.clone();

            let self_rx = combine_reactivity([
                node.label.as_ref().map(is_reactive_expr).unwrap_or(false),
                node.progress
                    .as_ref()
                    .map(is_reactive_expr)
                    .unwrap_or(false),
            ]);
            let reactive_fields = [
                node.label
                    .as_ref()
                    .is_some_and(is_reactive_expr)
                    .then_some(ReactiveField::Label),
                node.progress
                    .as_ref()
                    .is_some_and(is_reactive_expr)
                    .then_some(ReactiveField::Progress),
            ]
            .into_iter()
            .flatten()
            .collect();
            let self_shape = ShapeReactivity::Static;
            let structural_dependencies = collect_dependencies([
                opt_expr_dependency(&node.label),
                opt_expr_dependency(&node.progress),
            ]);

            Ok(CanonicalUiNode::Loading(CanonicalLoadingNode {
                base: node.base,
                label: node.label,
                progress: node.progress,
                meta: freeze_meta(
                    node_id.as_deref(),
                    self_rx,
                    self_rx,
                    reactive_fields,
                    self_shape,
                    self_shape,
                    structural_dependencies,
                    static_fields,
                    &ctx,
                ),
            }))
        }

        UiNode::Conditional(mut node) => {
            let wrapper = UiNode::Conditional(node.clone());
            ensure_identity_policy(&wrapper, &ctx, seen_ids)?;

            node.condition = collapse_required_bool_literal(node.condition);
            let static_fields = collect_static_fields_map(&node);
            let node_id = node.base.id.clone();

            let then_kind = kind_of(&node.r#then);
            let then_node =
                normalize_node_with_state(*node.r#then, ctx.child("then", 0, then_kind), seen_ids)?;

            let else_node = match node.r#else {
                Some(else_node) => {
                    let else_kind = kind_of(&else_node);
                    Some(normalize_node_with_state(
                        *else_node,
                        ctx.child("else", 0, else_kind),
                        seen_ids,
                    )?)
                }
                None => None,
            };

            let self_rx = if is_reactive_expr(&node.condition) {
                NodeReactivity::Conditional
            } else {
                NodeReactivity::Static
            };

            let subtree_rx = merge_reactivity(
                std::iter::once(self_rx)
                    .chain(std::iter::once(then_node.meta().subtree_reactivity))
                    .chain(else_node.iter().map(|n| n.meta().subtree_reactivity)),
            );
            let reactive_fields = (self_rx != NodeReactivity::Static)
                .then_some(ReactiveField::Condition)
                .into_iter()
                .collect();
            let self_shape = ShapeReactivity::ReplaceNode;
            let subtree_shape = merge_shape_reactivity(
                std::iter::once(self_shape)
                    .chain(std::iter::once(then_node.meta().subtree_shape_reactivity))
                    .chain(else_node.iter().map(|n| n.meta().subtree_shape_reactivity)),
            );
            let structural_dependencies = collect_dependencies([expr_dependency(&node.condition)]);

            Ok(CanonicalUiNode::Conditional(CanonicalConditionalNode {
                base: node.base,
                condition: node.condition,
                r#then: Box::new(then_node),
                r#else: else_node.map(Box::new),
                meta: freeze_meta(
                    node_id.as_deref(),
                    self_rx,
                    subtree_rx,
                    reactive_fields,
                    self_shape,
                    subtree_shape,
                    structural_dependencies,
                    static_fields,
                    &ctx,
                ),
            }))
        }

        UiNode::Slot(node) => {
            let wrapper = UiNode::Slot(node.clone());
            ensure_identity_policy(&wrapper, &ctx, seen_ids)?;
            let static_fields = collect_static_fields_map(&node);
            let node_id = node.base.id.clone();

            let fallback_node = match node.fallback {
                Some(fallback) => {
                    let fallback_kind = kind_of(&fallback);
                    Some(normalize_node_with_state(
                        *fallback,
                        ctx.child("fallback", 0, fallback_kind),
                        seen_ids,
                    )?)
                }
                None => None,
            };

            let self_rx = NodeReactivity::Static;
            let subtree_rx = merge_reactivity(
                std::iter::once(self_rx)
                    .chain(fallback_node.iter().map(|n| n.meta().subtree_reactivity)),
            );
            let self_shape = ShapeReactivity::Static;
            let subtree_shape = merge_shape_reactivity(
                std::iter::once(self_shape).chain(
                    fallback_node
                        .iter()
                        .map(|n| n.meta().subtree_shape_reactivity),
                ),
            );

            Ok(CanonicalUiNode::Slot(CanonicalSlotNode {
                base: node.base,
                name: node.name,
                fallback: fallback_node.map(Box::new),
                meta: freeze_meta(
                    node_id.as_deref(),
                    self_rx,
                    subtree_rx,
                    vec![],
                    self_shape,
                    subtree_shape,
                    vec![],
                    static_fields,
                    &ctx,
                ),
            }))
        }
    }
}

// ============================================================
// node metadata helpers
// ============================================================

fn base_id_of(node: &UiNode) -> Option<&str> {
    match node {
        UiNode::Section(n) => n.base.id.as_deref(),
        UiNode::Stack(n) => n.base.id.as_deref(),
        UiNode::Inline(n) => n.base.id.as_deref(),
        UiNode::Grid(n) => n.base.id.as_deref(),
        UiNode::Scroll(n) => n.base.id.as_deref(),
        UiNode::Text(n) => n.base.id.as_deref(),
        UiNode::Value(n) => n.base.id.as_deref(),
        UiNode::Icon(n) => n.base.id.as_deref(),
        UiNode::Badge(n) => n.base.id.as_deref(),
        UiNode::Divider(n) => n.base.id.as_deref(),
        UiNode::Media(n) => n.base.id.as_deref(),
        UiNode::Pressable(n) => n.base.id.as_deref(),
        UiNode::Item(n) => Some(n.id.as_str()),
        UiNode::List(n) => n.base.id.as_deref(),
        UiNode::Action(n) => n.base.id.as_deref(),
        UiNode::Actions(n) => n.base.id.as_deref(),
        UiNode::Disclosure(n) => n.base.id.as_deref(),
        UiNode::Menu(n) => n.base.id.as_deref(),
        UiNode::Input(n) => n.base.id.as_deref(),
        UiNode::Form(n) => n.base.id.as_deref(),
        UiNode::Status(n) => n.base.id.as_deref(),
        UiNode::Empty(n) => n.base.id.as_deref(),
        UiNode::Loading(n) => n.base.id.as_deref(),
        UiNode::Conditional(n) => n.base.id.as_deref(),
        UiNode::Slot(n) => n.base.id.as_deref(),
    }
}

fn kind_of(node: &UiNode) -> &'static str {
    match node {
        UiNode::Section(_) => "section",
        UiNode::Stack(_) => "stack",
        UiNode::Inline(_) => "inline",
        UiNode::Grid(_) => "grid",
        UiNode::Scroll(_) => "scroll",
        UiNode::Text(_) => "text",
        UiNode::Value(_) => "value",
        UiNode::Icon(_) => "icon",
        UiNode::Badge(_) => "badge",
        UiNode::Divider(_) => "divider",
        UiNode::Media(_) => "media",
        UiNode::Pressable(_) => "pressable",
        UiNode::Item(_) => "item",
        UiNode::List(_) => "list",
        UiNode::Action(_) => "action",
        UiNode::Actions(_) => "actions",
        UiNode::Disclosure(_) => "disclosure",
        UiNode::Menu(_) => "menu",
        UiNode::Input(_) => "input",
        UiNode::Form(_) => "form",
        UiNode::Status(_) => "status",
        UiNode::Empty(_) => "empty",
        UiNode::Loading(_) => "loading",
        UiNode::Conditional(_) => "conditional",
        UiNode::Slot(_) => "slot",
    }
}
