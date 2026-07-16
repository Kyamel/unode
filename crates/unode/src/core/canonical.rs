use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

use crate::core::ast::*;
use crate::core::slot::NodeOrigin;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum NodeReactivity {
    Static,
    Reactive,
    Conditional,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "camelCase")]
pub enum ReactiveField {
    Title,
    Subtitle,
    Description,
    Content,
    Value,
    Label,
    LabelExpanded,
    Disabled,
    Placeholder,
    HelpText,
    Message,
    Progress,
    Condition,
    BindingState,
    Continuation,
    MenuItems,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "camelCase")]
pub enum ShapeReactivity {
    Static,
    Visibility,
    ReplaceChildren,
    ReplaceNode,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum StructuralDependency {
    Binding { path: String },
    Param { name: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CanonicalMetadata {
    pub key: String,
    pub reactivity: NodeReactivity,
    pub subtree_reactivity: NodeReactivity,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reactive_fields: Vec<ReactiveField>,
    pub shape_reactivity: ShapeReactivity,
    pub subtree_shape_reactivity: ShapeReactivity,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub structural_dependencies: Vec<StructuralDependency>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub static_fields: BTreeMap<String, Primitive>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub origin: Option<NodeOrigin>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CanonicalScreen {
    #[serde(flatten)]
    pub base: NodeBase,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<StringOrExpr>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subtitle: Option<StringOrExpr>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub initial_focus: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub initial_state: Option<BTreeMap<String, JsonValue>>,
    pub children: Vec<CanonicalUiNode>,
    #[serde(flatten)]
    pub meta: CanonicalMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum CanonicalUiNode {
    Section(CanonicalSectionNode),
    Stack(CanonicalStackNode),
    Inline(CanonicalInlineNode),
    Grid(CanonicalGridNode),
    Scroll(CanonicalScrollNode),
    Text(CanonicalTextNode),
    Value(CanonicalValueNode),
    Icon(CanonicalIconNode),
    Badge(CanonicalBadgeNode),
    Divider(CanonicalDividerNode),
    Media(CanonicalMediaNode),
    Pressable(CanonicalPressableNode),
    Item(CanonicalItemNode),
    List(CanonicalListNode),
    Action(CanonicalActionNode),
    Actions(CanonicalActionsNode),
    Disclosure(CanonicalDisclosureNode),
    Menu(CanonicalMenuNode),
    Input(CanonicalInputNode),
    Form(CanonicalFormNode),
    Status(CanonicalStatusNode),
    Empty(CanonicalEmptyStateNode),
    Loading(CanonicalLoadingNode),
    Conditional(CanonicalConditionalNode),
    Slot(CanonicalSlotNode),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CanonicalSectionNode {
    #[serde(flatten)]
    pub base: NodeBase,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<ContainerRole>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<StringOrExpr>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<StringOrExpr>,
    pub children: Vec<CanonicalUiNode>,
    #[serde(flatten)]
    pub meta: CanonicalMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CanonicalStackNode {
    #[serde(flatten)]
    pub base: NodeBase,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gap: Option<Gap>,
    pub children: Vec<CanonicalUiNode>,
    #[serde(flatten)]
    pub meta: CanonicalMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CanonicalInlineNode {
    #[serde(flatten)]
    pub base: NodeBase,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gap: Option<Gap>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wrap: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub align: Option<Align>,
    pub children: Vec<CanonicalUiNode>,
    #[serde(flatten)]
    pub meta: CanonicalMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CanonicalGridNode {
    #[serde(flatten)]
    pub base: NodeBase,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_columns: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub columns: Option<ResponsiveGridColumns>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gap: Option<Gap>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub continuation: Option<CollectionContinuation>,
    pub children: Vec<CanonicalUiNode>,
    #[serde(flatten)]
    pub meta: CanonicalMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CanonicalScrollNode {
    #[serde(flatten)]
    pub base: NodeBase,
    pub children: Vec<CanonicalUiNode>,
    #[serde(flatten)]
    pub meta: CanonicalMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CanonicalTextNode {
    #[serde(flatten)]
    pub base: NodeBase,
    pub content: StringOrExpr,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<TextRole>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tone: Option<Tone>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub emphasis: Option<TextEmphasis>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub truncate: Option<bool>,
    #[serde(flatten)]
    pub meta: CanonicalMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CanonicalValueNode {
    #[serde(flatten)]
    pub base: NodeBase,
    pub value: PrimitiveOrExpr,
    pub format: ValueFormat,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub currency_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<TextRole>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tone: Option<Tone>,
    #[serde(flatten)]
    pub meta: CanonicalMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CanonicalIconNode {
    #[serde(flatten)]
    pub base: NodeBase,
    pub name: String,
    pub label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tone: Option<Tone>,
    #[serde(flatten)]
    pub meta: CanonicalMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CanonicalBadgeNode {
    #[serde(flatten)]
    pub base: NodeBase,
    pub label: StringOrExpr,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tone: Option<Tone>,
    #[serde(flatten)]
    pub meta: CanonicalMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CanonicalDividerNode {
    #[serde(flatten)]
    pub base: NodeBase,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<StringOrExpr>,
    #[serde(flatten)]
    pub meta: CanonicalMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CanonicalMediaNode {
    #[serde(flatten)]
    pub base: NodeBase,
    pub r#ref: MediaRef,
    pub media_kind: MediaKind,
    pub alt: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aspect_ratio: Option<AspectRatio>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expandable: Option<bool>,
    #[serde(flatten)]
    pub meta: CanonicalMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CanonicalPressableNode {
    #[serde(flatten)]
    pub base: NodeBase,
    pub child: Box<CanonicalUiNode>,
    pub action: ActionRef,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<StringOrExpr>,
    #[serde(flatten)]
    pub meta: CanonicalMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CanonicalItemNode {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta_map: Option<BTreeMap<String, JsonValue>>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub leading: Vec<CanonicalUiNode>,
    pub primary: Vec<CanonicalUiNode>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub secondary: Vec<CanonicalUiNode>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub trailing: Vec<CanonicalUiNode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action: Option<ActionRef>,
    #[serde(flatten)]
    pub meta: CanonicalMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CanonicalListNode {
    #[serde(flatten)]
    pub base: NodeBase,
    pub items: Vec<CanonicalItemNode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub density: Option<ListDensity>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub continuation: Option<CollectionContinuation>,
    #[serde(flatten)]
    pub meta: CanonicalMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CanonicalActionNode {
    #[serde(flatten)]
    pub base: NodeBase,
    pub label: StringOrExpr,
    pub action: ActionRef,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub intent: Option<ActionIntent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variant: Option<ActionVariant>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub leading_icon: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disabled: Option<BoolOrExpr>,
    #[serde(flatten)]
    pub meta: CanonicalMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CanonicalActionsNode {
    #[serde(flatten)]
    pub base: NodeBase,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub align: Option<Align>,
    pub children: Vec<CanonicalActionNode>,
    #[serde(flatten)]
    pub meta: CanonicalMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CanonicalDisclosureNode {
    #[serde(flatten)]
    pub base: NodeBase,
    pub binding: String,
    pub label: StringOrExpr,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label_expanded: Option<StringOrExpr>,
    pub children: Vec<CanonicalUiNode>,
    #[serde(flatten)]
    pub meta: CanonicalMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CanonicalMenuNode {
    #[serde(flatten)]
    pub base: NodeBase,
    pub label: StringOrExpr,
    pub items: Vec<MenuItem>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub intent: Option<ActionIntent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub align: Option<Align>,
    #[serde(flatten)]
    pub meta: CanonicalMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CanonicalInputNode {
    #[serde(flatten)]
    pub base: NodeBase,
    pub name: String,
    pub input_kind: InputKind,
    pub label: StringOrExpr,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<PrimitiveOrExpr>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub placeholder: Option<StringOrExpr>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub help_text: Option<StringOrExpr>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disabled: Option<BoolOrExpr>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub options: Vec<SelectChoice>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub constraints: Option<InputConstraints>,
    #[serde(flatten)]
    pub meta: CanonicalMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CanonicalFormNode {
    #[serde(flatten)]
    pub base: NodeBase,
    pub name: String,
    pub children: Vec<CanonicalUiNode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub submit: Option<ActionRef>,
    #[serde(flatten)]
    pub meta: CanonicalMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CanonicalStatusNode {
    #[serde(flatten)]
    pub base: NodeBase,
    pub severity: StatusSeverity,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<StringOrExpr>,
    pub message: StringOrExpr,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub actions: Vec<CanonicalActionNode>,
    #[serde(flatten)]
    pub meta: CanonicalMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CanonicalEmptyStateNode {
    #[serde(flatten)]
    pub base: NodeBase,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    pub title: StringOrExpr,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<StringOrExpr>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub actions: Vec<CanonicalActionNode>,
    #[serde(flatten)]
    pub meta: CanonicalMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CanonicalLoadingNode {
    #[serde(flatten)]
    pub base: NodeBase,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<StringOrExpr>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub progress: Option<NumberOrExpr>,
    #[serde(flatten)]
    pub meta: CanonicalMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CanonicalConditionalNode {
    #[serde(flatten)]
    pub base: NodeBase,
    pub condition: BoolOrExpr,
    pub r#then: Box<CanonicalUiNode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#else: Option<Box<CanonicalUiNode>>,
    #[serde(flatten)]
    pub meta: CanonicalMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CanonicalSlotNode {
    #[serde(flatten)]
    pub base: NodeBase,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fallback: Option<Box<CanonicalUiNode>>,
    #[serde(flatten)]
    pub meta: CanonicalMetadata,
}

impl CanonicalScreen {
    pub fn kind(&self) -> &'static str {
        "screen"
    }
}

impl CanonicalUiNode {
    pub fn meta(&self) -> &CanonicalMetadata {
        match self {
            Self::Section(node) => &node.meta,
            Self::Stack(node) => &node.meta,
            Self::Inline(node) => &node.meta,
            Self::Grid(node) => &node.meta,
            Self::Scroll(node) => &node.meta,
            Self::Text(node) => &node.meta,
            Self::Value(node) => &node.meta,
            Self::Icon(node) => &node.meta,
            Self::Badge(node) => &node.meta,
            Self::Divider(node) => &node.meta,
            Self::Media(node) => &node.meta,
            Self::Pressable(node) => &node.meta,
            Self::Item(node) => &node.meta,
            Self::List(node) => &node.meta,
            Self::Action(node) => &node.meta,
            Self::Actions(node) => &node.meta,
            Self::Disclosure(node) => &node.meta,
            Self::Menu(node) => &node.meta,
            Self::Input(node) => &node.meta,
            Self::Form(node) => &node.meta,
            Self::Status(node) => &node.meta,
            Self::Empty(node) => &node.meta,
            Self::Loading(node) => &node.meta,
            Self::Conditional(node) => &node.meta,
            Self::Slot(node) => &node.meta,
        }
    }

    pub fn kind(&self) -> &'static str {
        match self {
            Self::Section(_) => "section",
            Self::Stack(_) => "stack",
            Self::Inline(_) => "inline",
            Self::Grid(_) => "grid",
            Self::Scroll(_) => "scroll",
            Self::Text(_) => "text",
            Self::Value(_) => "value",
            Self::Icon(_) => "icon",
            Self::Badge(_) => "badge",
            Self::Divider(_) => "divider",
            Self::Media(_) => "media",
            Self::Pressable(_) => "pressable",
            Self::Item(_) => "item",
            Self::List(_) => "list",
            Self::Action(_) => "action",
            Self::Actions(_) => "actions",
            Self::Disclosure(_) => "disclosure",
            Self::Menu(_) => "menu",
            Self::Input(_) => "input",
            Self::Form(_) => "form",
            Self::Status(_) => "status",
            Self::Empty(_) => "empty",
            Self::Loading(_) => "loading",
            Self::Conditional(_) => "conditional",
            Self::Slot(_) => "slot",
        }
    }
}
