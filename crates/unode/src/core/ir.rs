use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::{json, Value as JsonValue};

use crate::core::ast::*;
use crate::core::canonical::*;
use crate::core::patch::{PatchOp, PatchValue};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct IrScreen {
    pub t: String,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub p: BTreeMap<String, JsonValue>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub c: Vec<IrNode>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct IrNode {
    pub t: String,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub p: BTreeMap<String, JsonValue>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub c: Vec<IrNode>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct IrPatchOp {
    pub o: String,
    pub k: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub f: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub v: Option<JsonValue>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub n: Option<IrNode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub c: Option<Vec<IrNode>>,
}

pub fn lower_screen(screen: &CanonicalScreen) -> IrScreen {
    let mut p = BTreeMap::new();

    if let Some(id) = &screen.base.id {
        p.insert("id".into(), json!(id));
    }
    if let Some(title) = &screen.title {
        p.insert("title".into(), lower_string_or_expr(title));
    }
    if let Some(subtitle) = &screen.subtitle {
        p.insert("subtitle".into(), lower_string_or_expr(subtitle));
    }

    inject_meta(&mut p, &screen.meta);

    IrScreen {
        t: "screen".to_string(),
        p,
        c: screen.children.iter().map(lower_node).collect(),
    }
}

pub fn lower_node(node: &CanonicalUiNode) -> IrNode {
    match node {
        CanonicalUiNode::Section(n) => {
            let mut p = BTreeMap::new();
            if let Some(title) = &n.title {
                p.insert("title".into(), lower_string_or_expr(title));
            }
            if let Some(desc) = &n.description {
                p.insert("desc".into(), lower_string_or_expr(desc));
            }
            inject_meta(&mut p, &n.meta);
            IrNode {
                t: "section".into(),
                p,
                c: n.children.iter().map(lower_node).collect(),
            }
        }
        CanonicalUiNode::Stack(n) => {
            let mut p = BTreeMap::new();
            opt_enum(&mut p, "gap", &n.gap);
            inject_meta(&mut p, &n.meta);
            IrNode {
                t: "stack".into(),
                p,
                c: n.children.iter().map(lower_node).collect(),
            }
        }
        CanonicalUiNode::Inline(n) => {
            let mut p = BTreeMap::new();
            opt_enum(&mut p, "gap", &n.gap);
            opt_bool(&mut p, "wrap", n.wrap);
            opt_enum(&mut p, "align", &n.align);
            inject_meta(&mut p, &n.meta);
            IrNode {
                t: "inline".into(),
                p,
                c: n.children.iter().map(lower_node).collect(),
            }
        }
        CanonicalUiNode::Grid(n) => {
            let mut p = BTreeMap::new();
            opt_enum(&mut p, "gap", &n.gap);
            if let Some(cols) = &n.columns {
                p.insert("cols".into(), lower_responsive_cols(cols));
            }
            if let Some(continuation) = &n.continuation {
                p.insert("cont".into(), lower_continuation(continuation));
            }
            inject_meta(&mut p, &n.meta);
            IrNode {
                t: "grid".into(),
                p,
                c: n.children.iter().map(lower_node).collect(),
            }
        }
        CanonicalUiNode::Scroll(n) => {
            let mut p = BTreeMap::new();
            inject_meta(&mut p, &n.meta);
            IrNode {
                t: "scroll".into(),
                p,
                c: n.children.iter().map(lower_node).collect(),
            }
        }
        CanonicalUiNode::Text(n) => {
            let mut p = BTreeMap::new();
            p.insert("content".into(), lower_string_or_expr(&n.content));
            opt_enum(&mut p, "role", &n.role);
            opt_enum(&mut p, "tone", &n.tone);
            opt_enum(&mut p, "em", &n.emphasis);
            opt_bool(&mut p, "tr", n.truncate);
            inject_meta(&mut p, &n.meta);
            IrNode { t: "text".into(), p, c: vec![] }
        }
        CanonicalUiNode::Value(n) => {
            let mut p = BTreeMap::new();
            p.insert("value".into(), lower_primitive_or_expr(&n.value));
            p.insert("fmt".into(), json!(enum_name(&n.format)));
            if let Some(code) = &n.currency_code {
                p.insert("cur".into(), json!(code));
            }
            opt_enum(&mut p, "role", &n.role);
            opt_enum(&mut p, "tone", &n.tone);
            inject_meta(&mut p, &n.meta);
            IrNode { t: "value".into(), p, c: vec![] }
        }
        CanonicalUiNode::Icon(n) => {
            let mut p = BTreeMap::new();
            p.insert("name".into(), json!(n.name));
            p.insert("label".into(), json!(n.label));
            opt_enum(&mut p, "tone", &n.tone);
            inject_meta(&mut p, &n.meta);
            IrNode { t: "icon".into(), p, c: vec![] }
        }
        CanonicalUiNode::Badge(n) => {
            let mut p = BTreeMap::new();
            p.insert("label".into(), lower_string_or_expr(&n.label));
            opt_enum(&mut p, "tone", &n.tone);
            inject_meta(&mut p, &n.meta);
            IrNode { t: "badge".into(), p, c: vec![] }
        }
        CanonicalUiNode::Divider(n) => {
            let mut p = BTreeMap::new();
            opt_string_or_expr(&mut p, "label", &n.label);
            inject_meta(&mut p, &n.meta);
            IrNode { t: "divider".into(), p, c: vec![] }
        }
        CanonicalUiNode::Media(n) => {
            let mut p = BTreeMap::new();
            p.insert("ref".into(), json!(n.r#ref));
            p.insert("kind".into(), json!(enum_name(&n.media_kind)));
            p.insert("alt".into(), json!(n.alt));
            if let Some(aspect_ratio) = &n.aspect_ratio {
                p.insert("ar".into(), json!(enum_name(aspect_ratio)));
            }
            opt_bool(&mut p, "exp", n.expandable);
            inject_meta(&mut p, &n.meta);
            IrNode { t: "media".into(), p, c: vec![] }
        }
        CanonicalUiNode::Pressable(n) => {
            let mut p = BTreeMap::new();
            p.insert("act".into(), lower_action_ref(&n.action));
            opt_string_or_expr(&mut p, "label", &n.label);
            inject_meta(&mut p, &n.meta);
            IrNode {
                t: "pressable".into(),
                p,
                c: vec![lower_node(&n.child)],
            }
        }
        CanonicalUiNode::Item(n) => lower_item(n),
        CanonicalUiNode::List(n) => {
            let mut p = BTreeMap::new();
            opt_enum(&mut p, "density", &n.density);
            if let Some(continuation) = &n.continuation {
                p.insert("cont".into(), lower_continuation(continuation));
            }
            inject_meta(&mut p, &n.meta);
            IrNode {
                t: "list".into(),
                p,
                c: n.items.iter().map(lower_item).collect(),
            }
        }
        CanonicalUiNode::Action(n) => lower_action(n),
        CanonicalUiNode::Actions(n) => {
            let mut p = BTreeMap::new();
            opt_enum(&mut p, "align", &n.align);
            inject_meta(&mut p, &n.meta);
            IrNode {
                t: "actions".into(),
                p,
                c: n.children.iter().map(lower_action).collect(),
            }
        }
        CanonicalUiNode::Disclosure(n) => {
            let mut p = BTreeMap::new();
            p.insert("bind".into(), json!(n.binding));
            p.insert("label".into(), lower_string_or_expr(&n.label));
            opt_string_or_expr(&mut p, "labelExp", &n.label_expanded);
            inject_meta(&mut p, &n.meta);
            IrNode {
                t: "disclosure".into(),
                p,
                c: n.children.iter().map(lower_node).collect(),
            }
        }
        CanonicalUiNode::Menu(n) => {
            let mut p = BTreeMap::new();
            p.insert("label".into(), lower_string_or_expr(&n.label));
            opt_enum(&mut p, "intent", &n.intent);
            opt_enum(&mut p, "align", &n.align);
            inject_meta(&mut p, &n.meta);
            let c = n
                .items
                .iter()
                .map(|item| {
                    let mut p_item = BTreeMap::new();
                    if let Some(id) = &item.id {
                        p_item.insert("id".into(), json!(id));
                    }
                    p_item.insert("label".into(), lower_string_or_expr(&item.label));
                    p_item.insert("do".into(), lower_action_ref(&item.action));
                    if let Some(selected) = item.selected {
                        p_item.insert("sel".into(), json!(selected));
                    }
                    if let Some(disabled) = &item.disabled {
                        p_item.insert("dis".into(), lower_bool_or_expr(disabled));
                    }
                    IrNode {
                        t: "menu-item".into(),
                        p: p_item,
                        c: vec![],
                    }
                })
                .collect();
            IrNode { t: "menu".into(), p, c }
        }
        CanonicalUiNode::Input(n) => {
            let mut p = BTreeMap::new();
            p.insert("name".into(), json!(n.name));
            p.insert("label".into(), lower_string_or_expr(&n.label));
            p.insert("kind".into(), json!(enum_name(&n.input_kind)));
            opt_primitive_or_expr(&mut p, "value", &n.value);
            opt_string_or_expr(&mut p, "ph", &n.placeholder);
            opt_string_or_expr(&mut p, "help", &n.help_text);
            opt_bool(&mut p, "req", n.required);
            opt_bool_or_expr(&mut p, "dis", &n.disabled);
            inject_meta(&mut p, &n.meta);
            IrNode { t: "input".into(), p, c: vec![] }
        }
        CanonicalUiNode::Form(n) => {
            let mut p = BTreeMap::new();
            p.insert("name".into(), json!(n.name));
            if let Some(submit) = &n.submit {
                p.insert("submit".into(), lower_action_ref(submit));
            }
            inject_meta(&mut p, &n.meta);
            IrNode {
                t: "form".into(),
                p,
                c: n.children.iter().map(lower_node).collect(),
            }
        }
        CanonicalUiNode::Status(n) => {
            let mut p = BTreeMap::new();
            p.insert("severity".into(), json!(enum_name(&n.severity)));
            opt_string_or_expr(&mut p, "title", &n.title);
            p.insert("msg".into(), lower_string_or_expr(&n.message));
            inject_meta(&mut p, &n.meta);
            IrNode {
                t: "status".into(),
                p,
                c: n.actions.iter().map(lower_action).collect(),
            }
        }
        CanonicalUiNode::Empty(n) => {
            let mut p = BTreeMap::new();
            if let Some(icon) = &n.icon {
                p.insert("icon".into(), json!(icon));
            }
            p.insert("title".into(), lower_string_or_expr(&n.title));
            opt_string_or_expr(&mut p, "msg", &n.message);
            inject_meta(&mut p, &n.meta);
            IrNode {
                t: "empty".into(),
                p,
                c: n.actions.iter().map(lower_action).collect(),
            }
        }
        CanonicalUiNode::Loading(n) => {
            let mut p = BTreeMap::new();
            opt_string_or_expr(&mut p, "label", &n.label);
            opt_number_or_expr(&mut p, "progress", &n.progress);
            inject_meta(&mut p, &n.meta);
            IrNode { t: "loading".into(), p, c: vec![] }
        }
        CanonicalUiNode::Conditional(n) => {
            let mut p = BTreeMap::new();
            p.insert("if".into(), lower_bool_or_expr(&n.condition));
            inject_meta(&mut p, &n.meta);
            let mut c = vec![lower_node(&n.r#then)];
            if let Some(else_node) = &n.r#else {
                c.push(lower_node(else_node));
            }
            IrNode { t: "if".into(), p, c }
        }
        CanonicalUiNode::Slot(n) => {
            let mut p = BTreeMap::new();
            p.insert("name".into(), json!(n.name));
            inject_meta(&mut p, &n.meta);
            let mut c = vec![];
            if let Some(fallback) = &n.fallback {
                c.push(lower_node(fallback));
            }
            IrNode { t: "slot".into(), p, c }
        }
    }
}

pub fn lower_patch_op(op: &PatchOp) -> IrPatchOp {
    match op {
        PatchOp::SetProp { key, field, value } => IrPatchOp {
            o: "sp".into(),
            k: key.clone(),
            f: Some(reactive_field_code(*field).to_string()),
            v: Some(lower_patch_value(value)),
            n: None,
            c: None,
        },
        PatchOp::ReplaceNode { key, node } => IrPatchOp {
            o: "rn".into(),
            k: key.clone(),
            f: None,
            v: None,
            n: Some(lower_node(node)),
            c: None,
        },
        PatchOp::ReplaceChildren { key, children } => IrPatchOp {
            o: "rc".into(),
            k: key.clone(),
            f: None,
            v: None,
            n: None,
            c: Some(children.iter().map(lower_node).collect()),
        },
    }
}

pub fn lower_patch_ops(ops: &[PatchOp]) -> Vec<IrPatchOp> {
    ops.iter().map(lower_patch_op).collect()
}

fn lower_item(n: &CanonicalItemNode) -> IrNode {
    let mut p = BTreeMap::new();
    p.insert("id".into(), json!(n.id));
    inject_meta(&mut p, &n.meta);

    let mut c = Vec::new();
    c.extend(wrap_group("leading", &n.leading));
    c.extend(wrap_group("primary", &n.primary));
    c.extend(wrap_group("secondary", &n.secondary));
    c.extend(wrap_group("trailing", &n.trailing));

    IrNode { t: "item".into(), p, c }
}

fn lower_action(n: &CanonicalActionNode) -> IrNode {
    let mut p = BTreeMap::new();
    p.insert("label".into(), lower_string_or_expr(&n.label));
    p.insert("do".into(), lower_action_ref(&n.action));
    opt_enum(&mut p, "intent", &n.intent);
    opt_enum(&mut p, "variant", &n.variant);
    if let Some(icon) = &n.leading_icon {
        p.insert("icon".into(), json!(icon));
    }
    opt_bool_or_expr(&mut p, "dis", &n.disabled);
    inject_meta(&mut p, &n.meta);
    IrNode { t: "action".into(), p, c: vec![] }
}

fn inject_meta(p: &mut BTreeMap<String, JsonValue>, meta: &CanonicalMetadata) {
    p.insert("_k".into(), json!(meta.key));
    p.insert("_r".into(), json!(rx_code(meta.reactivity)));
    p.insert("_sr".into(), json!(rx_code(meta.subtree_reactivity)));
    if meta.shape_reactivity != ShapeReactivity::Static {
        p.insert("_h".into(), json!(shape_code(meta.shape_reactivity)));
    }
    if meta.subtree_shape_reactivity != ShapeReactivity::Static {
        p.insert("_hs".into(), json!(shape_code(meta.subtree_shape_reactivity)));
    }
    if !meta.reactive_fields.is_empty() {
        p.insert(
            "rf".into(),
            json!(
                meta.reactive_fields
                    .iter()
                    .map(|field| reactive_field_code(*field))
                    .collect::<Vec<_>>()
            ),
        );
    }
    if !meta.structural_dependencies.is_empty() {
        p.insert(
            "sd".into(),
            JsonValue::Array(
                meta.structural_dependencies
                    .iter()
                    .map(lower_structural_dependency)
                    .collect(),
            ),
        );
    }

    if !meta.static_fields.is_empty() {
        p.insert("sf".into(), lower_static_fields(&meta.static_fields));
    }
}

fn lower_static_fields(fields: &BTreeMap<String, Primitive>) -> JsonValue {
    let mut out = serde_json::Map::new();
    for (k, v) in fields {
        out.insert(k.clone(), lower_primitive(v));
    }
    JsonValue::Object(out)
}

fn wrap_group(group: &str, nodes: &[CanonicalUiNode]) -> Vec<IrNode> {
    if nodes.is_empty() {
        return vec![];
    }

    vec![IrNode {
        t: group.to_string(),
        p: BTreeMap::new(),
        c: nodes.iter().map(lower_node).collect(),
    }]
}

fn lower_continuation(value: &CollectionContinuation) -> JsonValue {
    match value {
        CollectionContinuation::Incremental(v) => {
            let mut m = serde_json::Map::new();
            m.insert("k".into(), json!("inc"));
            m.insert("bind".into(), json!(v.binding));
            m.insert("init".into(), json!(v.initial));
            m.insert("step".into(), json!(v.step));
            if let Some(label) = &v.label {
                m.insert("label".into(), lower_string_or_expr(label));
            }
            JsonValue::Object(m)
        }
        CollectionContinuation::Remote(v) => {
            let mut m = serde_json::Map::new();
            m.insert("k".into(), json!("remote"));
            m.insert("more".into(), json!(v.has_more));
            m.insert("load".into(), lower_action_ref(&v.load_more));
            if let Some(label) = &v.label {
                m.insert("label".into(), lower_string_or_expr(label));
            }
            if let Some(label) = &v.loading_label {
                m.insert("loading".into(), lower_string_or_expr(label));
            }
            JsonValue::Object(m)
        }
    }
}

fn lower_responsive_cols(cols: &ResponsiveGridColumns) -> JsonValue {
    let mut m = serde_json::Map::new();
    if let Some(v) = cols.base {
        m.insert("b".into(), json!(v));
    }
    if let Some(v) = cols.sm {
        m.insert("s".into(), json!(v));
    }
    if let Some(v) = cols.md {
        m.insert("m".into(), json!(v));
    }
    if let Some(v) = cols.lg {
        m.insert("l".into(), json!(v));
    }
    if let Some(v) = cols.xl {
        m.insert("x".into(), json!(v));
    }
    JsonValue::Object(m)
}

fn lower_string_or_expr(value: &StringOrExpr) -> JsonValue {
    match value {
        OneOrExpr::Value(v) => json!({ "v": v }),
        OneOrExpr::Expr(UiExpr::Literal { value }) => json!({ "v": value }),
        OneOrExpr::Expr(UiExpr::Binding { path }) => json!({ "b": path }),
        OneOrExpr::Expr(UiExpr::Param { name }) => json!({ "pa": name }),
    }
}

fn lower_bool_or_expr(value: &BoolOrExpr) -> JsonValue {
    match value {
        OneOrExpr::Value(v) => json!({ "v": v }),
        OneOrExpr::Expr(UiExpr::Literal { value }) => json!({ "v": value }),
        OneOrExpr::Expr(UiExpr::Binding { path }) => json!({ "b": path }),
        OneOrExpr::Expr(UiExpr::Param { name }) => json!({ "pa": name }),
    }
}

fn lower_number_or_expr(value: &NumberOrExpr) -> JsonValue {
    match value {
        OneOrExpr::Value(v) => json!({ "v": v }),
        OneOrExpr::Expr(UiExpr::Literal { value }) => json!({ "v": value }),
        OneOrExpr::Expr(UiExpr::Binding { path }) => json!({ "b": path }),
        OneOrExpr::Expr(UiExpr::Param { name }) => json!({ "pa": name }),
    }
}

fn lower_primitive_or_expr(value: &PrimitiveOrExpr) -> JsonValue {
    match value {
        OneOrExpr::Value(v) => json!({ "v": lower_primitive(v) }),
        OneOrExpr::Expr(UiExpr::Literal { value }) => json!({ "v": lower_primitive(value) }),
        OneOrExpr::Expr(UiExpr::Binding { path }) => json!({ "b": path }),
        OneOrExpr::Expr(UiExpr::Param { name }) => json!({ "pa": name }),
    }
}

fn lower_primitive(value: &Primitive) -> JsonValue {
    match value {
        Some(v) => json!(v),
        _ => JsonValue::Null,
    }
}

fn lower_action_ref(action: &ActionRef) -> JsonValue {
    let mut m = serde_json::Map::new();
    m.insert("t".into(), json!(action_type_name(&action.r#type)));

    if let Some(params) = &action.params {
        m.insert("p".into(), json!(params));
    }

    if let Some(confirm) = &action.confirm {
        let mut cm = serde_json::Map::new();
        if let Some(title) = &confirm.title {
            cm.insert("t".into(), lower_string_or_expr(title));
        }
        cm.insert("m".into(), lower_string_or_expr(&confirm.message));
        m.insert("c".into(), JsonValue::Object(cm));
    }

    JsonValue::Object(m)
}

fn lower_patch_value(value: &PatchValue) -> JsonValue {
    match value {
        PatchValue::String(value) => lower_string_or_expr(value),
        PatchValue::Bool(value) => lower_bool_or_expr(value),
        PatchValue::Number(value) => lower_number_or_expr(value),
        PatchValue::Primitive(value) => lower_primitive_or_expr(value),
        PatchValue::Action(value) => lower_action_ref(value),
        PatchValue::Json(value) => value.clone(),
    }
}

fn lower_structural_dependency(value: &StructuralDependency) -> JsonValue {
    match value {
        StructuralDependency::Binding { path } => json!({ "b": path }),
        StructuralDependency::Param { name } => json!({ "pa": name }),
    }
}

fn action_type_name(action_type: &ActionType) -> String {
    match action_type {
        ActionType::Core(core) => enum_name(core),
        ActionType::Custom(s) => s.clone(),
    }
}

fn rx_code(value: NodeReactivity) -> &'static str {
    match value {
        NodeReactivity::Static => "s",
        NodeReactivity::Reactive => "r",
        NodeReactivity::Conditional => "c",
    }
}

fn shape_code(value: ShapeReactivity) -> &'static str {
    match value {
        ShapeReactivity::Static => "s",
        ShapeReactivity::Visibility => "v",
        ShapeReactivity::ReplaceChildren => "c",
        ShapeReactivity::ReplaceNode => "n",
    }
}

fn reactive_field_code(value: ReactiveField) -> &'static str {
    match value {
        ReactiveField::Title => "ti",
        ReactiveField::Subtitle => "su",
        ReactiveField::Description => "de",
        ReactiveField::Content => "ct",
        ReactiveField::Value => "va",
        ReactiveField::Label => "lb",
        ReactiveField::LabelExpanded => "lx",
        ReactiveField::Disabled => "di",
        ReactiveField::Placeholder => "ph",
        ReactiveField::HelpText => "ht",
        ReactiveField::Message => "ms",
        ReactiveField::Progress => "pg",
        ReactiveField::Condition => "if",
        ReactiveField::BindingState => "bs",
        ReactiveField::Continuation => "co",
        ReactiveField::MenuItems => "mi",
    }
}

fn enum_name<T: std::fmt::Debug>(value: &T) -> String {
    format!("{value:?}").to_lowercase()
}

fn opt_enum<T: std::fmt::Debug>(p: &mut BTreeMap<String, JsonValue>, key: &str, value: &Option<T>) {
    if let Some(v) = value {
        p.insert(key.into(), json!(enum_name(v)));
    }
}

fn opt_bool(p: &mut BTreeMap<String, JsonValue>, key: &str, value: Option<bool>) {
    if let Some(v) = value {
        p.insert(key.into(), json!(v));
    }
}

fn opt_string_or_expr(p: &mut BTreeMap<String, JsonValue>, key: &str, value: &Option<StringOrExpr>) {
    if let Some(v) = value {
        p.insert(key.into(), lower_string_or_expr(v));
    }
}

fn opt_bool_or_expr(p: &mut BTreeMap<String, JsonValue>, key: &str, value: &Option<BoolOrExpr>) {
    if let Some(v) = value {
        p.insert(key.into(), lower_bool_or_expr(v));
    }
}

fn opt_primitive_or_expr(
    p: &mut BTreeMap<String, JsonValue>,
    key: &str,
    value: &Option<PrimitiveOrExpr>,
) {
    if let Some(v) = value {
        p.insert(key.into(), lower_primitive_or_expr(v));
    }
}

fn opt_number_or_expr(p: &mut BTreeMap<String, JsonValue>, key: &str, value: &Option<NumberOrExpr>) {
    if let Some(v) = value {
        p.insert(key.into(), lower_number_or_expr(v));
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use serde_json::json;

    use crate::core::ast::{
        ActionConfirm, ActionNode, ActionRef, ActionType, AspectRatio, BadgeNode, CollectionContinuation,
        CoreActionType, DisclosureNode, EmptyStateNode, FormNode, Gap, GridNode, IconNode,
        IncrementalContinuation, InputKind, InputNode, ListNode, LoadingNode, MediaKind, MediaNode,
        MediaRef, MenuItem, MenuNode, NodeBase, OneOrExpr, PressableNode, PrimitiveOrExpr,
        RemoteContinuation, ResponsiveGridColumns, ScreenNode, SectionNode, SlotNode, StatusNode,
        StatusSeverity, StringOrExpr, TextEmphasis, TextNode, TextRole, Tone, UiExpr, UiNode,
        ValueFormat, ValueNode, ConditionalNode, ItemNode,
    };
    use crate::core::canonical::{
        CanonicalMetadata, NodeReactivity, ReactiveField, ShapeReactivity, StructuralDependency,
    };
    use crate::core::normalize::normalize_screen;
    use crate::core::patch::{PatchOp, PatchValue};

    use super::{
        inject_meta, lower_action_ref, lower_continuation, lower_patch_op, lower_patch_ops,
        lower_screen, lower_static_fields, IrPatchOp,
    };

    fn base(id: &str) -> NodeBase {
        NodeBase {
            id: Some(id.to_string()),
            meta: None,
        }
    }

    fn string_value(value: &str) -> StringOrExpr {
        OneOrExpr::Value(value.to_string())
    }

    fn string_binding(path: &str) -> StringOrExpr {
        OneOrExpr::Expr(UiExpr::Binding {
            path: path.to_string(),
        })
    }

    fn bool_binding(path: &str) -> crate::core::ast::BoolOrExpr {
        OneOrExpr::Expr(UiExpr::Binding {
            path: path.to_string(),
        })
    }

    fn number_binding(path: &str) -> crate::core::ast::NumberOrExpr {
        OneOrExpr::Expr(UiExpr::Binding {
            path: path.to_string(),
        })
    }

    fn primitive_binding(path: &str) -> PrimitiveOrExpr {
        OneOrExpr::Expr(UiExpr::Binding {
            path: path.to_string(),
        })
    }

    fn action(action_type: CoreActionType) -> ActionRef {
        ActionRef {
            r#type: ActionType::Core(action_type),
            params: None,
            confirm: None,
        }
    }

    fn nav_action_with_params(params: &[(&str, serde_json::Value)]) -> ActionRef {
        let mut map = BTreeMap::new();
        for (key, value) in params {
            map.insert((*key).to_string(), value.clone());
        }

        ActionRef {
            r#type: ActionType::Core(CoreActionType::Navigate),
            params: Some(map),
            confirm: None,
        }
    }

    fn mixed_screen_fixture() -> ScreenNode {
        ScreenNode {
            base: base("screen"),
            title: Some(string_binding("screen.title")),
            subtitle: Some(string_value("Library")),
            route_tabs: None,
            initial_focus: None,
            initial_state: None,
            children: vec![
                UiNode::Section(SectionNode {
                    base: base("hero"),
                    role: None,
                    title: Some(string_binding("hero.title")),
                    description: Some(string_value("Featured today")),
                    children: vec![
                        UiNode::Text(TextNode {
                            base: base("hero-text"),
                            content: string_binding("hero.subtitle"),
                            role: Some(TextRole::Subtitle),
                            tone: Some(Tone::Info),
                            emphasis: Some(TextEmphasis::Strong),
                            truncate: Some(true),
                        }),
                        UiNode::Pressable(PressableNode {
                            base: base("hero-open"),
                            child: Box::new(UiNode::Text(TextNode {
                                base: base("hero-open-text"),
                                content: string_value("Open details"),
                                role: None,
                                tone: None,
                                emphasis: None,
                                truncate: None,
                            })),
                            action: nav_action_with_params(&[("to", json!("details"))]),
                            label: Some(string_value("Open")),
                        }),
                    ],
                }),
                UiNode::Grid(GridNode {
                    base: base("gallery"),
                    max_columns: None,
                    columns: Some(ResponsiveGridColumns {
                        base: Some(2),
                        sm: None,
                        md: Some(4),
                        lg: None,
                        xl: None,
                    }),
                    gap: Some(Gap::Lg),
                    continuation: Some(CollectionContinuation::Remote(RemoteContinuation {
                        has_more: true,
                        load_more: action(CoreActionType::LoadMore),
                        label: Some(string_value("More")),
                        loading_label: Some(string_binding("gallery.loading_label")),
                    })),
                    children: vec![
                        UiNode::Badge(BadgeNode {
                            base: base("featured-badge"),
                            label: string_value("Featured"),
                            tone: Some(Tone::Success),
                        }),
                        UiNode::Media(MediaNode {
                            base: base("cover"),
                            r#ref: MediaRef::Asset {
                                name: "cover-art".to_string(),
                            },
                            media_kind: MediaKind::Cover,
                            alt: "Cover image".to_string(),
                            aspect_ratio: Some(AspectRatio::Poster),
                            expandable: Some(true),
                        }),
                    ],
                }),
                UiNode::List(ListNode {
                    base: base("chapters"),
                    items: vec![ItemNode {
                        id: "chapter-1".to_string(),
                        meta: None,
                        leading: vec![UiNode::Icon(IconNode {
                            base: base("chapter-icon"),
                            name: "book".to_string(),
                            label: "Chapter".to_string(),
                            tone: Some(Tone::Muted),
                        })],
                        primary: vec![UiNode::Text(TextNode {
                            base: base("chapter-title"),
                            content: string_binding("chapter.title"),
                            role: Some(TextRole::Body),
                            tone: None,
                            emphasis: None,
                            truncate: None,
                        })],
                        secondary: vec![],
                        trailing: vec![UiNode::Value(ValueNode {
                            base: base("chapter-pages"),
                            value: primitive_binding("chapter.pages"),
                            format: ValueFormat::Number,
                            currency_code: None,
                            role: Some(TextRole::Caption),
                            tone: Some(Tone::Muted),
                        })],
                        action: Some(nav_action_with_params(&[("chapterId", json!(1))])),
                    }],
                    density: None,
                    continuation: Some(CollectionContinuation::Incremental(
                        IncrementalContinuation {
                            binding: "chapters.visible".to_string(),
                            initial: 10,
                            step: 10,
                            label: Some(string_binding("chapters.more_label")),
                        },
                    )),
                }),
                UiNode::Disclosure(DisclosureNode {
                    base: base("filters"),
                    binding: "ui.filters_open".to_string(),
                    label: string_value("Filters"),
                    label_expanded: Some(string_binding("ui.filters_expanded_label")),
                    children: vec![UiNode::Form(FormNode {
                        base: base("search-form"),
                        name: "search".to_string(),
                        children: vec![UiNode::Input(InputNode {
                            base: base("query"),
                            name: "query".to_string(),
                            input_kind: InputKind::Text,
                            label: string_value("Query"),
                            value: Some(primitive_binding("filters.query")),
                            placeholder: Some(string_value("Type to search")),
                            help_text: Some(string_binding("filters.help")),
                            required: Some(false),
                            disabled: Some(bool_binding("filters.disabled")),
                            options: vec![],
                            constraints: None,
                        })],
                        submit: Some(action(CoreActionType::Submit)),
                    })],
                }),
                UiNode::Menu(MenuNode {
                    base: base("sort-menu"),
                    label: string_value("Sort"),
                    items: vec![
                        MenuItem {
                            id: Some("recent".to_string()),
                            label: string_value("Recent"),
                            action: nav_action_with_params(&[("sort", json!("recent"))]),
                            selected: Some(true),
                            disabled: None,
                        },
                        MenuItem {
                            id: Some("popular".to_string()),
                            label: string_binding("menu.popular_label"),
                            action: nav_action_with_params(&[("sort", json!("popular"))]),
                            selected: Some(false),
                            disabled: Some(bool_binding("menu.popular_disabled")),
                        },
                    ],
                    intent: None,
                    align: None,
                }),
                UiNode::Conditional(ConditionalNode {
                    base: base("upsell"),
                    condition: bool_binding("flags.show_upsell"),
                    r#then: Box::new(UiNode::Status(StatusNode {
                        base: base("promo-status"),
                        severity: StatusSeverity::Info,
                        title: Some(string_value("Promo")),
                        message: string_binding("promo.message"),
                        actions: vec![ActionNode {
                            base: base("promo-cta"),
                            label: string_value("View"),
                            action: action(CoreActionType::Navigate),
                            intent: None,
                            variant: None,
                            leading_icon: Some("sparkles".to_string()),
                            disabled: Some(bool_binding("promo.disabled")),
                        }],
                    })),
                    r#else: Some(Box::new(UiNode::Empty(EmptyStateNode {
                        base: base("promo-empty"),
                        icon: Some("sparkles".to_string()),
                        title: string_value("Nothing yet"),
                        message: Some(string_value("Come back soon")),
                        actions: vec![],
                    }))),
                }),
                UiNode::Slot(SlotNode {
                    base: base("footer-slot"),
                    name: "footer".to_string(),
                    fallback: Some(Box::new(UiNode::Loading(LoadingNode {
                        base: base("footer-loading"),
                        label: Some(string_binding("footer.label")),
                        progress: Some(number_binding("footer.progress")),
                    }))),
                }),
            ],
        }
    }

    #[test]
    fn lowers_mixed_screen_to_stable_ir_snapshot() {
        let screen = normalize_screen(mixed_screen_fixture()).unwrap();
        let lowered = lower_screen(&screen);
        let rendered = serde_json::to_string_pretty(&lowered).unwrap();
        let expected = include_str!("testdata/mixed_screen_ir_snapshot.json");

        assert_eq!(rendered, expected.trim());
    }

    #[test]
    fn lowers_action_ref_with_confirm_and_params_compactly() {
        let action = ActionRef {
            r#type: ActionType::Core(CoreActionType::Navigate),
            params: Some(BTreeMap::from([
                ("tab".to_string(), json!("details")),
                ("id".to_string(), json!(42)),
            ])),
            confirm: Some(ActionConfirm {
                title: Some(string_value("Leave page?")),
                message: string_binding("confirm.message"),
            }),
        };

        assert_eq!(
            lower_action_ref(&action),
            json!({
                "t": "navigate",
                "p": {
                    "id": 42,
                    "tab": "details"
                },
                "c": {
                    "t": { "v": "Leave page?" },
                    "m": { "b": "confirm.message" }
                }
            })
        );
    }

    #[test]
    fn lowers_incremental_and_remote_continuations_compactly() {
        let incremental = CollectionContinuation::Incremental(IncrementalContinuation {
            binding: "items.visible".to_string(),
            initial: 20,
            step: 10,
            label: Some(string_binding("items.more_label")),
        });

        let remote = CollectionContinuation::Remote(RemoteContinuation {
            has_more: true,
            load_more: action(CoreActionType::LoadMore),
            label: Some(string_value("Load more")),
            loading_label: Some(string_binding("items.loading_label")),
        });

        assert_eq!(
            lower_continuation(&incremental),
            json!({
                "k": "inc",
                "bind": "items.visible",
                "init": 20,
                "step": 10,
                "label": { "b": "items.more_label" }
            })
        );

        assert_eq!(
            lower_continuation(&remote),
            json!({
                "k": "remote",
                "more": true,
                "load": { "t": "loadmore" },
                "label": { "v": "Load more" },
                "loading": { "b": "items.loading_label" }
            })
        );
    }

    #[test]
    fn lowers_static_fields_and_meta_compactly() {
        let fields = BTreeMap::from([
            ("label".to_string(), Some(json!("Continue"))),
            ("count".to_string(), Some(json!(3))),
            ("enabled".to_string(), Some(json!(true))),
            ("hint".to_string(), None),
        ]);

        assert_eq!(
            lower_static_fields(&fields),
            json!({
                "count": 3,
                "enabled": true,
                "hint": null,
                "label": "Continue"
            })
        );

        let meta = CanonicalMetadata {
            key: "node-1".to_string(),
            reactivity: NodeReactivity::Reactive,
            subtree_reactivity: NodeReactivity::Conditional,
            reactive_fields: vec![ReactiveField::Label],
            shape_reactivity: ShapeReactivity::Visibility,
            subtree_shape_reactivity: ShapeReactivity::ReplaceNode,
            structural_dependencies: vec![
                StructuralDependency::Binding {
                    path: "ui.open".to_string(),
                },
                StructuralDependency::Param {
                    name: "tab".to_string(),
                },
            ],
            static_fields: fields,
        };
        let mut out = BTreeMap::new();

        inject_meta(&mut out, &meta);

        assert_eq!(
            json!(out),
            json!({
                "_k": "node-1",
                "_h": "v",
                "_hs": "n",
                "_r": "r",
                "_sr": "c",
                "rf": ["lb"],
                "sd": [
                    { "b": "ui.open" },
                    { "pa": "tab" }
                ],
                "sf": {
                    "count": 3,
                    "enabled": true,
                    "hint": null,
                    "label": "Continue"
                }
            })
        );
    }

    #[test]
    fn lowers_patch_ops_compactly() {
        let replace_node = normalize_screen(ScreenNode {
            base: base("patch-screen"),
            title: None,
            subtitle: None,
            route_tabs: None,
            initial_focus: None,
            initial_state: None,
            children: vec![UiNode::Text(TextNode {
                base: base("patched-node"),
                content: string_binding("work.name"),
                role: None,
                tone: None,
                emphasis: None,
                truncate: None,
            })],
        })
        .unwrap()
        .children
        .into_iter()
        .next()
        .unwrap();

        let replace_children = normalize_screen(ScreenNode {
            base: base("children-screen"),
            title: None,
            subtitle: None,
            route_tabs: None,
            initial_focus: None,
            initial_state: None,
            children: vec![
                UiNode::Text(TextNode {
                    base: base("child-a"),
                    content: string_value("Alpha"),
                    role: None,
                    tone: None,
                    emphasis: None,
                    truncate: None,
                }),
                UiNode::Text(TextNode {
                    base: base("child-b"),
                    content: string_binding("beta.title"),
                    role: None,
                    tone: None,
                    emphasis: None,
                    truncate: None,
                }),
            ],
        })
        .unwrap()
        .children;

        let ops = vec![
            PatchOp::SetProp {
                key: "hero-text".to_string(),
                field: ReactiveField::Content,
                value: PatchValue::String(string_binding("hero.subtitle")),
            },
            PatchOp::ReplaceNode {
                key: "hero".to_string(),
                node: replace_node,
            },
            PatchOp::ReplaceChildren {
                key: "list".to_string(),
                children: replace_children,
            },
        ];

        assert_eq!(
            lower_patch_op(&ops[0]),
            IrPatchOp {
                o: "sp".into(),
                k: "hero-text".into(),
                f: Some("ct".into()),
                v: Some(json!({ "b": "hero.subtitle" })),
                n: None,
                c: None,
            }
        );

        assert_eq!(
            serde_json::to_value(lower_patch_ops(&ops)).unwrap(),
            json!([
                {
                    "o": "sp",
                    "k": "hero-text",
                    "f": "ct",
                    "v": { "b": "hero.subtitle" }
                },
                {
                    "o": "rn",
                    "k": "hero",
                    "n": {
                        "t": "text",
                        "p": {
                            "_k": "patched-node",
                            "_r": "r",
                            "_sr": "r",
                            "content": { "b": "work.name" },
                            "em": "normal",
                            "rf": ["ct"],
                            "role": "body",
                            "sd": [{ "b": "work.name" }],
                            "sf": {
                                "emphasis": "normal",
                                "role": "body"
                            }
                        }
                    }
                },
                {
                    "o": "rc",
                    "k": "list",
                    "c": [
                        {
                            "t": "text",
                            "p": {
                                "_k": "child-a",
                                "_r": "s",
                                "_sr": "s",
                                "content": { "v": "Alpha" },
                                "em": "normal",
                                "role": "body",
                                "sf": {
                                    "content": "Alpha",
                                    "emphasis": "normal",
                                    "role": "body"
                                }
                            }
                        },
                        {
                            "t": "text",
                            "p": {
                                "_k": "child-b",
                                "_r": "r",
                                "_sr": "r",
                                "content": { "b": "beta.title" },
                                "em": "normal",
                                "rf": ["ct"],
                                "role": "body",
                                "sd": [{ "b": "beta.title" }],
                                "sf": {
                                    "emphasis": "normal",
                                    "role": "body"
                                }
                            }
                        }
                    ]
                }
            ])
        );
    }
}
