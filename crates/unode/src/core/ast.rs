//! Canonical, serializable Abstract Syntax Tree for uNode UIs.
use crate::core::chrome::ScreenRouteTabsMeta;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::collections::BTreeMap;

pub const UNODE_AST_VERSION: &str = "2.0.0-alpha.1";

// Generic Expression Types

/// Expression form used for route params, local bindings, and literals.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind")]
#[serde(rename_all = "camelCase")]
pub enum UiExpr<T> {
    Literal { value: T },
    Binding { path: String },
    Param { name: String },
}

/// Represents a value that can either be a direct value or a `UiExpr`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum OneOrExpr<T> {
    Value(T),
    Expr(UiExpr<T>),
}

// Type aliases for common OneOrExpr types
pub type StringOrExpr = OneOrExpr<String>;
pub type BoolOrExpr = OneOrExpr<bool>;
pub type NumberOrExpr = OneOrExpr<f64>;
pub type Primitive = Option<JsonValue>; // Represents JSON-safe primitives
pub type PrimitiveOrExpr = OneOrExpr<Primitive>;

// Enumerated Dictionaries

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum Tone {
    Default,
    Muted,
    Info,
    Success,
    Warning,
    Danger,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum Gap {
    None,
    Xs,
    Sm,
    Md,
    Lg,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum TextRole {
    Heading,
    Title,
    Subtitle,
    Body,
    Label,
    Caption,
    Code,
    Hint,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ContainerRole {
    Page,
    Panel,
    Section,
    Group,
    Toolbar,
    Sidebar,
    Dialog,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ValueFormat {
    Number,
    Currency,
    Date,
    Datetime,
    Duration,
    Bytes,
    Percent,
    Raw,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ActionIntent {
    Primary,
    Secondary,
    Ghost,
    Danger,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ActionVariant {
    Button,
    Link,
    IconButton,
    MenuItem,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum InputKind {
    Text,
    Textarea,
    Number,
    Password,
    Email,
    Url,
    Boolean,
    Select,
    Multiselect,
    Date,
    File,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum AspectRatio {
    Square,
    Poster,
    Video,
    Auto,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum Align {
    Start,
    Center,
    End,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum TextEmphasis {
    Normal,
    Strong,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum StatusSeverity {
    Info,
    Success,
    Warning,
    Danger,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ListDensity {
    Compact,
    Normal,
    Comfortable,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum MediaKind {
    Image,
    Video,
    Audio,
    Cover,
    Avatar,
    Thumbnail,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum ActionType {
    Core(CoreActionType),
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum CoreActionType {
    Navigate,
    Submit,
    Dismiss,
    Refresh,
    LoadMore,
}

// Core AST Structures

/// Shared node metadata.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub struct NodeBase {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<BTreeMap<String, JsonValue>>,
}

/// Symbolic action reference interpreted by the runtime or renderer.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ActionRef {
    pub r#type: ActionType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<BTreeMap<String, JsonValue>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confirm: Option<ActionConfirm>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ActionConfirm {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<StringOrExpr>,
    pub message: StringOrExpr,
}

/// Reference to a media resource.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
#[serde(rename_all = "camelCase")]
pub enum MediaRef {
    Url {
        src: String,
    },
    #[serde(rename = "at-blob")]
    AtBlob {
        did: String,
        cid: String,
    },
    Asset {
        name: String,
    },
    Placeholder {
        #[serde(skip_serializing_if = "Option::is_none")]
        kind: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        label: Option<String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub struct ResponsiveGridColumns {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sm: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub md: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lg: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub xl: Option<u8>,
}

// UI Node Definitions

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind")]
#[serde(rename_all = "camelCase")]
pub enum UiNode {
    Section(SectionNode),
    Stack(StackNode),
    Inline(InlineNode),
    Grid(GridNode),
    Scroll(ScrollNode),
    Text(TextNode),
    Value(ValueNode),
    Icon(IconNode),
    Badge(BadgeNode),
    Divider(DividerNode),
    Media(MediaNode),
    Pressable(PressableNode),
    Item(ItemNode),
    List(ListNode),
    Action(ActionNode),
    Actions(ActionsNode),
    Disclosure(DisclosureNode),
    Menu(MenuNode),
    Input(InputNode),
    Form(FormNode),
    Status(StatusNode),
    Empty(EmptyStateNode),
    Loading(LoadingNode),
    Conditional(ConditionalNode),
    Slot(SlotNode),
}

/// Root screen node returned from plugin `render()`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ScreenNode {
    #[serde(flatten)]
    pub base: NodeBase,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<StringOrExpr>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subtitle: Option<StringOrExpr>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub route_tabs: Option<ScreenRouteTabsMeta>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub initial_focus: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub initial_state: Option<BTreeMap<String, JsonValue>>,
    pub children: Vec<UiNode>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SectionNode {
    #[serde(flatten)]
    pub base: NodeBase,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<ContainerRole>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<StringOrExpr>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<StringOrExpr>,
    pub children: Vec<UiNode>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct StackNode {
    #[serde(flatten)]
    pub base: NodeBase,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gap: Option<Gap>,
    pub children: Vec<UiNode>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct InlineNode {
    #[serde(flatten)]
    pub base: NodeBase,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gap: Option<Gap>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wrap: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub align: Option<Align>,
    pub children: Vec<UiNode>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GridNode {
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
    pub children: Vec<UiNode>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ScrollNode {
    #[serde(flatten)]
    pub base: NodeBase,
    pub children: Vec<UiNode>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TextNode {
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
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ValueNode {
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
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct IconNode {
    #[serde(flatten)]
    pub base: NodeBase,
    pub name: String,
    pub label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tone: Option<Tone>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct BadgeNode {
    #[serde(flatten)]
    pub base: NodeBase,
    pub label: StringOrExpr,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tone: Option<Tone>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DividerNode {
    #[serde(flatten)]
    pub base: NodeBase,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<StringOrExpr>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MediaNode {
    #[serde(flatten)]
    pub base: NodeBase,
    pub r#ref: MediaRef,
    pub media_kind: MediaKind,
    pub alt: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aspect_ratio: Option<AspectRatio>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expandable: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PressableNode {
    #[serde(flatten)]
    pub base: NodeBase,
    pub child: Box<UiNode>,
    pub action: ActionRef,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<StringOrExpr>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ItemNode {
    // Note: No `base` field. `id` is mandatory and not part of `NodeBase`.
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<BTreeMap<String, JsonValue>>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub leading: Vec<UiNode>,
    pub primary: Vec<UiNode>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub secondary: Vec<UiNode>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub trailing: Vec<UiNode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action: Option<ActionRef>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind")]
#[serde(rename_all = "camelCase")]
pub enum CollectionContinuation {
    Incremental(IncrementalContinuation),
    Remote(RemoteContinuation),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct IncrementalContinuation {
    pub binding: String,
    pub initial: u32,
    pub step: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<StringOrExpr>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RemoteContinuation {
    pub has_more: bool,
    pub load_more: ActionRef,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<StringOrExpr>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub loading_label: Option<StringOrExpr>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ListNode {
    #[serde(flatten)]
    pub base: NodeBase,
    pub items: Vec<ItemNode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub density: Option<ListDensity>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub continuation: Option<CollectionContinuation>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ActionNode {
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
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ActionsNode {
    #[serde(flatten)]
    pub base: NodeBase,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub align: Option<Align>,
    pub children: Vec<ActionNode>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DisclosureNode {
    #[serde(flatten)]
    pub base: NodeBase,
    pub binding: String,
    pub label: StringOrExpr,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label_expanded: Option<StringOrExpr>,
    pub children: Vec<UiNode>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MenuItem {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub label: StringOrExpr,
    pub action: ActionRef,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selected: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disabled: Option<BoolOrExpr>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MenuNode {
    #[serde(flatten)]
    pub base: NodeBase,
    pub label: StringOrExpr,
    pub items: Vec<MenuItem>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub intent: Option<ActionIntent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub align: Option<Align>, // Exclude "center"
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SelectChoice {
    pub label: String,
    pub value: Primitive,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub struct InputConstraints {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_length: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_length: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pattern: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct InputNode {
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
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct FormNode {
    #[serde(flatten)]
    pub base: NodeBase,
    pub name: String,
    pub children: Vec<UiNode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub submit: Option<ActionRef>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct StatusNode {
    #[serde(flatten)]
    pub base: NodeBase,
    pub severity: StatusSeverity,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<StringOrExpr>,
    pub message: StringOrExpr,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub actions: Vec<ActionNode>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct EmptyStateNode {
    #[serde(flatten)]
    pub base: NodeBase,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    pub title: StringOrExpr,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<StringOrExpr>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub actions: Vec<ActionNode>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct LoadingNode {
    #[serde(flatten)]
    pub base: NodeBase,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<StringOrExpr>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub progress: Option<NumberOrExpr>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ConditionalNode {
    #[serde(flatten)]
    pub base: NodeBase,
    pub condition: BoolOrExpr,
    pub r#then: Box<UiNode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#else: Option<Box<UiNode>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SlotNode {
    #[serde(flatten)]
    pub base: NodeBase,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fallback: Option<Box<UiNode>>,
}
