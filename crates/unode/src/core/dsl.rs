//! Fluent builder API (DSL) for creating uNode ASTs.
use super::ast::*;
use serde_json::Value as JsonValue;
use std::collections::BTreeMap;

// --- Core Traits ---

/// A trait for types that can be converted into a `UiNode`.
pub trait IntoNode {
    fn into_node(self) -> UiNode;
}

/// A trait for types that can be converted into a collection of `UiNode`s.
pub trait IntoChildren {
    fn into_children(self) -> Vec<UiNode>;
}

/// A trait for types that can be converted into an `ActionNode`.
pub trait IntoAction {
    fn into_action(self) -> ActionNode;
}

/// A trait for types that can be converted into a collection of `ActionNode`s.
pub trait IntoActions {
    fn into_actions(self) -> Vec<ActionNode>;
}

/// A trait for types that can be converted into an `ItemNode`.
pub trait IntoItem {
    fn into_item(self) -> ItemNode;
}

/// A trait for types that can be converted into a collection of `ItemNode`s.
pub trait IntoItems {
    fn into_items(self) -> Vec<ItemNode>;
}

/// A trait for types that can be converted into a `MenuItem`.
pub trait IntoMenuItem {
    fn into_menu_item(self) -> MenuItem;
}

/// A trait for types that can be converted into a collection of `MenuItem`s.
pub trait IntoMenuItems {
    fn into_menu_items(self) -> Vec<MenuItem>;
}

/// A local conversion trait for the AST's `Primitive = Option<JsonValue>`.
pub trait IntoPrimitive {
    fn into_primitive(self) -> Primitive;
}

/// A local conversion trait for `PrimitiveOrExpr`.
pub trait IntoPrimitiveOrExpr {
    fn into_primitive_or_expr(self) -> PrimitiveOrExpr;
}

// --- Small Wrapper Types ---

/// Wrapper for iterator-based children.
pub struct Children(pub Vec<UiNode>);

/// Wrapper for iterator-based actions.
pub struct Actions(pub Vec<ActionNode>);

/// Wrapper for iterator-based items.
pub struct Items(pub Vec<ItemNode>);

/// Wrapper for iterator-based menu items.
pub struct MenuItems(pub Vec<MenuItem>);

// --- Trait Implementations: IntoNode for canonical node structs ---

impl IntoNode for UiNode {
    fn into_node(self) -> UiNode {
        self
    }
}

macro_rules! impl_into_node_for_ast_nodes {
    ($($ty:ty => $variant:ident),* $(,)?) => {
        $(
            impl IntoNode for $ty {
                fn into_node(self) -> UiNode {
                    UiNode::$variant(self)
                }
            }
        )*
    };
}

impl_into_node_for_ast_nodes!(
    SectionNode => Section,
    StackNode => Stack,
    InlineNode => Inline,
    GridNode => Grid,
    ScrollNode => Scroll,
    TextNode => Text,
    ValueNode => Value,
    IconNode => Icon,
    BadgeNode => Badge,
    DividerNode => Divider,
    MediaNode => Media,
    PressableNode => Pressable,
    ItemNode => Item,
    ListNode => List,
    ActionNode => Action,
    ActionsNode => Actions,
    DisclosureNode => Disclosure,
    MenuNode => Menu,
    InputNode => Input,
    FormNode => Form,
    StatusNode => Status,
    EmptyStateNode => Empty,
    LoadingNode => Loading,
    ConditionalNode => Conditional,
    SlotNode => Slot,
);

// --- Trait Implementations: IntoChildren ---

impl IntoChildren for UiNode {
    fn into_children(self) -> Vec<UiNode> {
        vec![self]
    }
}

impl<T: IntoNode> IntoChildren for Option<T> {
    fn into_children(self) -> Vec<UiNode> {
        self.into_iter().map(IntoNode::into_node).collect()
    }
}

impl<T: IntoNode, const N: usize> IntoChildren for [T; N] {
    fn into_children(self) -> Vec<UiNode> {
        self.into_iter().map(IntoNode::into_node).collect()
    }
}

impl<T: IntoNode> IntoChildren for Vec<T> {
    fn into_children(self) -> Vec<UiNode> {
        self.into_iter().map(IntoNode::into_node).collect()
    }
}

impl IntoChildren for Children {
    fn into_children(self) -> Vec<UiNode> {
        self.0
    }
}

// --- Trait Implementations: IntoAction / IntoActions ---

impl IntoAction for ActionNode {
    fn into_action(self) -> ActionNode {
        self
    }
}

impl<T: IntoAction> IntoActions for T {
    fn into_actions(self) -> Vec<ActionNode> {
        vec![self.into_action()]
    }
}

impl<T: IntoAction> IntoActions for Option<T> {
    fn into_actions(self) -> Vec<ActionNode> {
        self.into_iter().map(IntoAction::into_action).collect()
    }
}

impl<T: IntoAction, const N: usize> IntoActions for [T; N] {
    fn into_actions(self) -> Vec<ActionNode> {
        self.into_iter().map(IntoAction::into_action).collect()
    }
}

impl<T: IntoAction> IntoActions for Vec<T> {
    fn into_actions(self) -> Vec<ActionNode> {
        self.into_iter().map(IntoAction::into_action).collect()
    }
}

impl IntoActions for Actions {
    fn into_actions(self) -> Vec<ActionNode> {
        self.0
    }
}

// --- Trait Implementations: IntoItem / IntoItems ---

impl IntoItem for ItemNode {
    fn into_item(self) -> ItemNode {
        self
    }
}

impl<T: IntoItem> IntoItems for T {
    fn into_items(self) -> Vec<ItemNode> {
        vec![self.into_item()]
    }
}

impl<T: IntoItem> IntoItems for Option<T> {
    fn into_items(self) -> Vec<ItemNode> {
        self.into_iter().map(IntoItem::into_item).collect()
    }
}

impl<T: IntoItem, const N: usize> IntoItems for [T; N] {
    fn into_items(self) -> Vec<ItemNode> {
        self.into_iter().map(IntoItem::into_item).collect()
    }
}

impl<T: IntoItem> IntoItems for Vec<T> {
    fn into_items(self) -> Vec<ItemNode> {
        self.into_iter().map(IntoItem::into_item).collect()
    }
}

impl IntoItems for Items {
    fn into_items(self) -> Vec<ItemNode> {
        self.0
    }
}

// --- Trait Implementations: IntoMenuItem / IntoMenuItems ---

impl IntoMenuItem for MenuItem {
    fn into_menu_item(self) -> MenuItem {
        self
    }
}

impl<T: IntoMenuItem> IntoMenuItems for T {
    fn into_menu_items(self) -> Vec<MenuItem> {
        vec![self.into_menu_item()]
    }
}

impl<T: IntoMenuItem> IntoMenuItems for Option<T> {
    fn into_menu_items(self) -> Vec<MenuItem> {
        self.into_iter().map(IntoMenuItem::into_menu_item).collect()
    }
}

impl<T: IntoMenuItem, const N: usize> IntoMenuItems for [T; N] {
    fn into_menu_items(self) -> Vec<MenuItem> {
        self.into_iter().map(IntoMenuItem::into_menu_item).collect()
    }
}

impl<T: IntoMenuItem> IntoMenuItems for Vec<T> {
    fn into_menu_items(self) -> Vec<MenuItem> {
        self.into_iter().map(IntoMenuItem::into_menu_item).collect()
    }
}

impl IntoMenuItems for MenuItems {
    fn into_menu_items(self) -> Vec<MenuItem> {
        self.0
    }
}

// --- Trait Implementations: Primitive conversions ---

impl IntoPrimitive for Primitive {
    fn into_primitive(self) -> Primitive {
        self
    }
}

impl IntoPrimitive for JsonValue {
    fn into_primitive(self) -> Primitive {
        Some(self)
    }
}

impl IntoPrimitive for &str {
    fn into_primitive(self) -> Primitive {
        Some(JsonValue::String(self.to_owned()))
    }
}

impl IntoPrimitive for String {
    fn into_primitive(self) -> Primitive {
        Some(JsonValue::String(self))
    }
}

impl IntoPrimitive for bool {
    fn into_primitive(self) -> Primitive {
        Some(JsonValue::Bool(self))
    }
}

macro_rules! impl_into_primitive_number {
    ($($ty:ty),* $(,)?) => {
        $(
            impl IntoPrimitive for $ty {
                fn into_primitive(self) -> Primitive {
                    serde_json::Number::from_f64(self as f64)
                        .map(JsonValue::Number)
                        .map(Some)
                        .unwrap_or(None)
                }
            }
        )*
    };
}

impl_into_primitive_number!(i8, i16, i32, i64, isize, u8, u16, u32, u64, usize, f32, f64);

impl IntoPrimitiveOrExpr for PrimitiveOrExpr {
    fn into_primitive_or_expr(self) -> PrimitiveOrExpr {
        self
    }
}

impl IntoPrimitiveOrExpr for Primitive {
    fn into_primitive_or_expr(self) -> PrimitiveOrExpr {
        OneOrExpr::Value(self)
    }
}

impl IntoPrimitiveOrExpr for JsonValue {
    fn into_primitive_or_expr(self) -> PrimitiveOrExpr {
        OneOrExpr::Value(Some(self))
    }
}

impl IntoPrimitiveOrExpr for &str {
    fn into_primitive_or_expr(self) -> PrimitiveOrExpr {
        OneOrExpr::Value(self.into_primitive())
    }
}

impl IntoPrimitiveOrExpr for String {
    fn into_primitive_or_expr(self) -> PrimitiveOrExpr {
        OneOrExpr::Value(self.into_primitive())
    }
}

impl IntoPrimitiveOrExpr for bool {
    fn into_primitive_or_expr(self) -> PrimitiveOrExpr {
        OneOrExpr::Value(self.into_primitive())
    }
}

macro_rules! impl_into_primitive_or_expr_number {
    ($($ty:ty),* $(,)?) => {
        $(
            impl IntoPrimitiveOrExpr for $ty {
                fn into_primitive_or_expr(self) -> PrimitiveOrExpr {
                    OneOrExpr::Value(self.into_primitive())
                }
            }
        )*
    };
}

impl_into_primitive_or_expr_number!(i8, i16, i32, i64, isize, u8, u16, u32, u64, usize, f32, f64);

impl IntoPrimitiveOrExpr for UiExpr<Primitive> {
    fn into_primitive_or_expr(self) -> PrimitiveOrExpr {
        OneOrExpr::Expr(self)
    }
}

// --- Helper Constructors for iterator-based collections ---

pub fn children<I, T>(iter: I) -> Children
where
    I: IntoIterator<Item = T>,
    T: IntoNode,
{
    Children(iter.into_iter().map(IntoNode::into_node).collect())
}

pub fn action_children<I, T>(iter: I) -> Actions
where
    I: IntoIterator<Item = T>,
    T: IntoAction,
{
    Actions(iter.into_iter().map(IntoAction::into_action).collect())
}

pub fn items<I, T>(iter: I) -> Items
where
    I: IntoIterator<Item = T>,
    T: IntoItem,
{
    Items(iter.into_iter().map(IntoItem::into_item).collect())
}

pub fn menu_items<I, T>(iter: I) -> MenuItems
where
    I: IntoIterator<Item = T>,
    T: IntoMenuItem,
{
    MenuItems(iter.into_iter().map(IntoMenuItem::into_menu_item).collect())
}

// --- Conditional Helpers ---

/// Renders a node only if the condition is true.
pub fn when(cond: bool, node: impl IntoNode) -> Option<UiNode> {
    cond.then(|| node.into_node())
}

/// Renders a node from the value of an `Option`.
pub fn when_some<T, F, O>(value: Option<T>, f: F) -> Option<UiNode>
where
    F: FnOnce(T) -> O,
    O: IntoNode,
{
    value.map(|v| f(v).into_node())
}

/// Creates a conditional subtree that evaluates at runtime.
pub fn conditional(
    condition: impl Into<BoolOrExpr>,
    then_node: impl IntoNode,
    else_node: Option<impl IntoNode>,
) -> UiNode {
    UiNode::Conditional(ConditionalNode {
        base: NodeBase::default(),
        condition: condition.into(),
        r#then: Box::new(then_node.into_node()),
        r#else: else_node.map(|n| Box::new(n.into_node())),
    })
}

// --- Expression Helpers ---

pub mod expr {
    use super::UiExpr;

    /// Creates a literal expression.
    ///
    /// Literal expressions are serialized in the same expression shape as
    /// bindings and route params, then collapsed during normalization where the
    /// target field accepts plain values. Use this when generated UI code wants
    /// to keep one uniform expression path even for constant values.
    pub fn literal<T>(value: T) -> UiExpr<T> {
        UiExpr::Literal { value }
    }

    /// Creates a reactive binding to a host state path.
    ///
    /// The path uses dot notation such as `"ui.count"` or `"items.0.title"`.
    /// During `track_reactive_bindings`, Unode records which node key reads this
    /// path. Later writes to the path wake only those nodes and produce targeted
    /// patch ops instead of re-rendering the whole screen.
    pub fn binding<T>(path: impl Into<String>) -> UiExpr<T> {
        UiExpr::Binding { path: path.into() }
    }

    /// Creates an expression backed by the current resolved route.
    ///
    /// Param expressions first read named route params, then query values. They
    /// are useful for screens whose labels, filters, or selected resources are
    /// derived from route state rather than local screen state.
    pub fn param<T>(name: impl Into<String>) -> UiExpr<T> {
        UiExpr::Param { name: name.into() }
    }
}

// --- OneOrExpr From Implementations ---

impl<T> From<T> for OneOrExpr<T> {
    fn from(value: T) -> Self {
        OneOrExpr::Value(value)
    }
}

impl From<&str> for OneOrExpr<String> {
    fn from(value: &str) -> Self {
        OneOrExpr::Value(value.to_string())
    }
}

impl From<&String> for OneOrExpr<String> {
    fn from(value: &String) -> Self {
        OneOrExpr::Value(value.clone())
    }
}

impl<T> From<UiExpr<T>> for OneOrExpr<T> {
    fn from(expr: UiExpr<T>) -> Self {
        OneOrExpr::Expr(expr)
    }
}

// --- AST Convenience Inherent Impl ---

impl ResponsiveGridColumns {
    pub fn base(mut self, value: u8) -> Self {
        self.base = Some(value);
        self
    }

    pub fn sm(mut self, value: u8) -> Self {
        self.sm = Some(value);
        self
    }

    pub fn md(mut self, value: u8) -> Self {
        self.md = Some(value);
        self
    }

    pub fn lg(mut self, value: u8) -> Self {
        self.lg = Some(value);
        self
    }

    pub fn xl(mut self, value: u8) -> Self {
        self.xl = Some(value);
        self
    }
}

// --- Helper Macros for repetitive impls ---

macro_rules! impl_into_children_via_node {
    ($($ty:ty),* $(,)?) => {
        $(
            impl IntoChildren for $ty {
                fn into_children(self) -> Vec<UiNode> {
                    vec![self.into_node()]
                }
            }
        )*
    };
}

impl_into_children_via_node!(
    SectionNode,
    StackNode,
    InlineNode,
    GridNode,
    ScrollNode,
    TextNode,
    ValueNode,
    IconNode,
    BadgeNode,
    DividerNode,
    MediaNode,
    PressableNode,
    ItemNode,
    ListNode,
    ActionNode,
    ActionsNode,
    DisclosureNode,
    MenuNode,
    InputNode,
    FormNode,
    StatusNode,
    EmptyStateNode,
    LoadingNode,
    ConditionalNode,
    SlotNode,
);

// --- Builders ---

/// Starts a root screen builder.
///
/// A screen is what a plugin returns from `plugin_render`: semantic children,
/// optional title/subtitle metadata, initial focus, and an optional flat
/// `initial_state` map. The host normalizes the finished `ScreenNode`, seeds its
/// `MemoryStateStore`, and lowers it to renderer IR.
///
/// ```rust
/// use unode::core::dsl as ui;
/// use unode::core::dsl::IntoNode;
///
/// let screen = ui::screen()
///     .id("demo.screen")
///     .title("Demo")
///     .children([ui::text("Hello").id("demo.greeting").into_node()])
///     .build();
/// ```
pub fn screen() -> ScreenBuilder {
    ScreenBuilder::default()
}

#[derive(Default)]
pub struct ScreenBuilder {
    id: Option<String>,
    title: Option<StringOrExpr>,
    subtitle: Option<StringOrExpr>,
    initial_focus: Option<String>,
    initial_state: Option<BTreeMap<String, JsonValue>>,
    children: Vec<UiNode>,
}

impl ScreenBuilder {
    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    pub fn title(mut self, title: impl Into<StringOrExpr>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn subtitle(mut self, subtitle: impl Into<StringOrExpr>) -> Self {
        self.subtitle = Some(subtitle.into());
        self
    }

    pub fn initial_focus(mut self, focus: impl Into<String>) -> Self {
        self.initial_focus = Some(focus.into());
        self
    }

    pub fn initial_state(mut self, state: BTreeMap<String, JsonValue>) -> Self {
        self.initial_state = Some(state);
        self
    }

    pub fn children(mut self, children: impl IntoChildren) -> Self {
        self.children.extend(children.into_children());
        self
    }

    pub fn child(mut self, child: impl IntoNode) -> Self {
        self.children.push(child.into_node());
        self
    }

    pub fn build(self) -> ScreenNode {
        ScreenNode {
            base: NodeBase {
                id: self.id,
                meta: None,
            },
            title: self.title,
            subtitle: self.subtitle,
            route_tabs: None,
            initial_focus: self.initial_focus,
            initial_state: self.initial_state,
            children: self.children,
        }
    }
}

/// Creates a vertical stack layout container.
pub fn stack() -> StackBuilder {
    StackBuilder::default()
}

#[derive(Default)]
pub struct StackBuilder {
    id: Option<String>,
    gap: Option<Gap>,
    children: Vec<UiNode>,
}

impl StackBuilder {
    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    pub fn gap(mut self, gap: Gap) -> Self {
        self.gap = Some(gap);
        self
    }

    pub fn children(mut self, children: impl IntoChildren) -> Self {
        self.children.extend(children.into_children());
        self
    }

    pub fn child(mut self, child: impl IntoNode) -> Self {
        self.children.push(child.into_node());
        self
    }
}

impl IntoNode for StackBuilder {
    fn into_node(self) -> UiNode {
        UiNode::Stack(StackNode {
            base: NodeBase {
                id: self.id,
                meta: None,
            },
            gap: self.gap,
            children: self.children,
        })
    }
}

impl IntoChildren for StackBuilder {
    fn into_children(self) -> Vec<UiNode> {
        vec![self.into_node()]
    }
}

/// Creates a semantic section container.
pub fn section() -> SectionBuilder {
    SectionBuilder::default()
}

#[derive(Default)]
pub struct SectionBuilder {
    id: Option<String>,
    role: Option<ContainerRole>,
    title: Option<StringOrExpr>,
    description: Option<StringOrExpr>,
    children: Vec<UiNode>,
}

impl SectionBuilder {
    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    pub fn role(mut self, role: ContainerRole) -> Self {
        self.role = Some(role);
        self
    }

    pub fn title(mut self, title: impl Into<StringOrExpr>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn description(mut self, description: impl Into<StringOrExpr>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn children(mut self, children: impl IntoChildren) -> Self {
        self.children.extend(children.into_children());
        self
    }

    pub fn child(mut self, child: impl IntoNode) -> Self {
        self.children.push(child.into_node());
        self
    }
}

impl IntoNode for SectionBuilder {
    fn into_node(self) -> UiNode {
        UiNode::Section(SectionNode {
            base: NodeBase {
                id: self.id,
                meta: None,
            },
            role: self.role,
            title: self.title,
            description: self.description,
            children: self.children,
        })
    }
}

impl IntoChildren for SectionBuilder {
    fn into_children(self) -> Vec<UiNode> {
        vec![self.into_node()]
    }
}

/// Creates an inline layout container.
pub fn inline() -> InlineBuilder {
    InlineBuilder::default()
}

#[derive(Default)]
pub struct InlineBuilder {
    id: Option<String>,
    gap: Option<Gap>,
    wrap: Option<bool>,
    align: Option<Align>,
    children: Vec<UiNode>,
}

impl InlineBuilder {
    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    pub fn gap(mut self, gap: Gap) -> Self {
        self.gap = Some(gap);
        self
    }

    pub fn wrap(mut self, wrap: bool) -> Self {
        self.wrap = Some(wrap);
        self
    }

    pub fn align(mut self, align: Align) -> Self {
        self.align = Some(align);
        self
    }

    pub fn children(mut self, children: impl IntoChildren) -> Self {
        self.children.extend(children.into_children());
        self
    }

    pub fn child(mut self, child: impl IntoNode) -> Self {
        self.children.push(child.into_node());
        self
    }
}

impl IntoNode for InlineBuilder {
    fn into_node(self) -> UiNode {
        UiNode::Inline(InlineNode {
            base: NodeBase {
                id: self.id,
                meta: None,
            },
            gap: self.gap,
            wrap: self.wrap,
            align: self.align,
            children: self.children,
        })
    }
}

impl IntoChildren for InlineBuilder {
    fn into_children(self) -> Vec<UiNode> {
        vec![self.into_node()]
    }
}

/// Creates a scrollable container.
pub fn scroll() -> ScrollBuilder {
    ScrollBuilder::default()
}

#[derive(Default)]
pub struct ScrollBuilder {
    id: Option<String>,
    children: Vec<UiNode>,
}

impl ScrollBuilder {
    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    pub fn children(mut self, children: impl IntoChildren) -> Self {
        self.children.extend(children.into_children());
        self
    }

    pub fn child(mut self, child: impl IntoNode) -> Self {
        self.children.push(child.into_node());
        self
    }
}

impl IntoNode for ScrollBuilder {
    fn into_node(self) -> UiNode {
        UiNode::Scroll(ScrollNode {
            base: NodeBase {
                id: self.id,
                meta: None,
            },
            children: self.children,
        })
    }
}

impl IntoChildren for ScrollBuilder {
    fn into_children(self) -> Vec<UiNode> {
        vec![self.into_node()]
    }
}

/// Helper to create responsive grid columns.
pub fn cols() -> ResponsiveGridColumns {
    ResponsiveGridColumns::default()
}

/// Starts a semantic text node builder.
///
/// Text content can be a plain string or a `StringOrExpr`, including
/// `expr::binding::<String>("path")`. Give text nodes stable IDs when they are
/// reactive or when a renderer/plugin anchor needs to address them directly.
pub fn text(content: impl Into<StringOrExpr>) -> TextBuilder {
    TextBuilder {
        content: content.into(),
        id: None,
        role: None,
        tone: None,
        emphasis: None,
        truncate: None,
    }
}

pub struct TextBuilder {
    id: Option<String>,
    content: StringOrExpr,
    role: Option<TextRole>,
    tone: Option<Tone>,
    emphasis: Option<TextEmphasis>,
    truncate: Option<bool>,
}

impl TextBuilder {
    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    pub fn role(mut self, role: TextRole) -> Self {
        self.role = Some(role);
        self
    }

    pub fn tone(mut self, tone: Tone) -> Self {
        self.tone = Some(tone);
        self
    }

    pub fn emphasis(mut self, emphasis: TextEmphasis) -> Self {
        self.emphasis = Some(emphasis);
        self
    }

    pub fn truncate(mut self, truncate: bool) -> Self {
        self.truncate = Some(truncate);
        self
    }
}

impl IntoNode for TextBuilder {
    fn into_node(self) -> UiNode {
        UiNode::Text(TextNode {
            base: NodeBase {
                id: self.id,
                meta: None,
            },
            content: self.content,
            role: self.role,
            tone: self.tone,
            emphasis: self.emphasis,
            truncate: self.truncate,
        })
    }
}

impl IntoChildren for TextBuilder {
    fn into_children(self) -> Vec<UiNode> {
        vec![self.into_node()]
    }
}

/// Creates a responsive grid container.
pub fn grid() -> GridBuilder {
    GridBuilder::default()
}

#[derive(Default)]
pub struct GridBuilder {
    id: Option<String>,
    max_columns: Option<u8>,
    columns: Option<ResponsiveGridColumns>,
    gap: Option<Gap>,
    continuation: Option<CollectionContinuation>,
    children: Vec<UiNode>,
}

impl GridBuilder {
    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    pub fn max_columns(mut self, max: u8) -> Self {
        self.max_columns = Some(max);
        self
    }

    pub fn columns(mut self, columns: ResponsiveGridColumns) -> Self {
        self.columns = Some(columns);
        self
    }

    pub fn gap(mut self, gap: Gap) -> Self {
        self.gap = Some(gap);
        self
    }

    pub fn continuation(mut self, continuation: CollectionContinuation) -> Self {
        self.continuation = Some(continuation);
        self
    }

    pub fn children(mut self, children: impl IntoChildren) -> Self {
        self.children.extend(children.into_children());
        self
    }

    pub fn child(mut self, child: impl IntoNode) -> Self {
        self.children.push(child.into_node());
        self
    }
}

impl IntoNode for GridBuilder {
    fn into_node(self) -> UiNode {
        UiNode::Grid(GridNode {
            base: NodeBase {
                id: self.id,
                meta: None,
            },
            max_columns: self.max_columns,
            columns: self.columns,
            gap: self.gap,
            continuation: self.continuation,
            children: self.children,
        })
    }
}

impl IntoChildren for GridBuilder {
    fn into_children(self) -> Vec<UiNode> {
        vec![self.into_node()]
    }
}

/// Creates a formatted scalar value node.
pub fn value(current: impl IntoPrimitiveOrExpr, format: ValueFormat) -> ValueBuilder {
    ValueBuilder {
        id: None,
        current: current.into_primitive_or_expr(),
        format,
        currency_code: None,
        role: None,
        tone: None,
    }
}

pub struct ValueBuilder {
    id: Option<String>,
    current: PrimitiveOrExpr,
    format: ValueFormat,
    currency_code: Option<String>,
    role: Option<TextRole>,
    tone: Option<Tone>,
}

impl ValueBuilder {
    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    pub fn currency_code(mut self, code: impl Into<String>) -> Self {
        self.currency_code = Some(code.into());
        self
    }

    pub fn role(mut self, role: TextRole) -> Self {
        self.role = Some(role);
        self
    }

    pub fn tone(mut self, tone: Tone) -> Self {
        self.tone = Some(tone);
        self
    }
}

impl IntoNode for ValueBuilder {
    fn into_node(self) -> UiNode {
        UiNode::Value(ValueNode {
            base: NodeBase {
                id: self.id,
                meta: None,
            },
            value: self.current,
            format: self.format,
            currency_code: self.currency_code,
            role: self.role,
            tone: self.tone,
        })
    }
}

impl IntoChildren for ValueBuilder {
    fn into_children(self) -> Vec<UiNode> {
        vec![self.into_node()]
    }
}

/// Creates a semantic icon node.
pub fn icon(name: impl Into<String>, label: impl Into<String>) -> IconBuilder {
    IconBuilder {
        id: None,
        name: name.into(),
        label: label.into(),
        tone: None,
    }
}

pub struct IconBuilder {
    id: Option<String>,
    name: String,
    label: String,
    tone: Option<Tone>,
}

impl IconBuilder {
    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    pub fn tone(mut self, tone: Tone) -> Self {
        self.tone = Some(tone);
        self
    }
}

impl IntoNode for IconBuilder {
    fn into_node(self) -> UiNode {
        UiNode::Icon(IconNode {
            base: NodeBase {
                id: self.id,
                meta: None,
            },
            name: self.name,
            label: self.label,
            tone: self.tone,
        })
    }
}

impl IntoChildren for IconBuilder {
    fn into_children(self) -> Vec<UiNode> {
        vec![self.into_node()]
    }
}

/// Creates a compact badge/chip node.
pub fn badge(label: impl Into<StringOrExpr>) -> BadgeBuilder {
    BadgeBuilder {
        id: None,
        label: label.into(),
        tone: None,
    }
}

pub struct BadgeBuilder {
    id: Option<String>,
    label: StringOrExpr,
    tone: Option<Tone>,
}

impl BadgeBuilder {
    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    pub fn tone(mut self, tone: Tone) -> Self {
        self.tone = Some(tone);
        self
    }
}

impl IntoNode for BadgeBuilder {
    fn into_node(self) -> UiNode {
        UiNode::Badge(BadgeNode {
            base: NodeBase {
                id: self.id,
                meta: None,
            },
            label: self.label,
            tone: self.tone,
        })
    }
}

impl IntoChildren for BadgeBuilder {
    fn into_children(self) -> Vec<UiNode> {
        vec![self.into_node()]
    }
}

/// Creates a semantic divider.
pub fn divider() -> DividerBuilder {
    DividerBuilder {
        id: None,
        label: None,
    }
}

pub struct DividerBuilder {
    id: Option<String>,
    label: Option<StringOrExpr>,
}

impl DividerBuilder {
    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    pub fn label(mut self, label: impl Into<StringOrExpr>) -> Self {
        self.label = Some(label.into());
        self
    }
}

impl IntoNode for DividerBuilder {
    fn into_node(self) -> UiNode {
        UiNode::Divider(DividerNode {
            base: NodeBase {
                id: self.id,
                meta: None,
            },
            label: self.label,
        })
    }
}

impl IntoChildren for DividerBuilder {
    fn into_children(self) -> Vec<UiNode> {
        vec![self.into_node()]
    }
}

/// Creates a media node that delegates rendering details to the platform renderer.
pub fn media(r#ref: MediaRef, media_kind: MediaKind, alt: impl Into<String>) -> MediaBuilder {
    MediaBuilder {
        id: None,
        r#ref,
        media_kind: media_kind.into(),
        alt: alt.into(),
        aspect_ratio: None,
        expandable: None,
    }
}

pub struct MediaBuilder {
    id: Option<String>,
    r#ref: MediaRef,
    media_kind: MediaKind,
    alt: String,
    aspect_ratio: Option<AspectRatio>,
    expandable: Option<bool>,
}

impl MediaBuilder {
    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    pub fn aspect_ratio(mut self, aspect_ratio: AspectRatio) -> Self {
        self.aspect_ratio = Some(aspect_ratio);
        self
    }

    pub fn expandable(mut self, expandable: bool) -> Self {
        self.expandable = Some(expandable);
        self
    }
}

impl IntoNode for MediaBuilder {
    fn into_node(self) -> UiNode {
        UiNode::Media(MediaNode {
            base: NodeBase {
                id: self.id,
                meta: None,
            },
            r#ref: self.r#ref,
            media_kind: self.media_kind,
            alt: self.alt,
            aspect_ratio: self.aspect_ratio,
            expandable: self.expandable,
        })
    }
}

impl IntoChildren for MediaBuilder {
    fn into_children(self) -> Vec<UiNode> {
        vec![self.into_node()]
    }
}

/// Makes a single child node behave as one semantic pressable region.
pub fn pressable(child: impl IntoNode, action: impl Into<ActionRef>) -> PressableBuilder {
    PressableBuilder {
        id: None,
        child: child.into_node(),
        action: action.into(),
        label: None,
    }
}

pub struct PressableBuilder {
    id: Option<String>,
    child: UiNode,
    action: ActionRef,
    label: Option<StringOrExpr>,
}

impl PressableBuilder {
    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    pub fn label(mut self, label: impl Into<StringOrExpr>) -> Self {
        self.label = Some(label.into());
        self
    }
}

impl IntoNode for PressableBuilder {
    fn into_node(self) -> UiNode {
        UiNode::Pressable(PressableNode {
            base: NodeBase {
                id: self.id,
                meta: None,
            },
            child: Box::new(self.child),
            action: self.action,
            label: self.label,
        })
    }
}

impl IntoChildren for PressableBuilder {
    fn into_children(self) -> Vec<UiNode> {
        vec![self.into_node()]
    }
}

/// Creates a reusable item row suitable for lists and other collections.
pub fn item(id: impl Into<String>, primary: impl IntoChildren) -> ItemBuilder {
    ItemBuilder {
        id: id.into(),
        meta: None,
        leading: Vec::new(),
        primary: primary.into_children(),
        secondary: Vec::new(),
        trailing: Vec::new(),
        action: None,
    }
}

pub struct ItemBuilder {
    id: String,
    meta: Option<BTreeMap<String, JsonValue>>,
    leading: Vec<UiNode>,
    primary: Vec<UiNode>,
    secondary: Vec<UiNode>,
    trailing: Vec<UiNode>,
    action: Option<ActionRef>,
}

impl ItemBuilder {
    pub fn meta(mut self, meta: BTreeMap<String, JsonValue>) -> Self {
        self.meta = Some(meta);
        self
    }

    pub fn leading(mut self, nodes: impl IntoChildren) -> Self {
        self.leading.extend(nodes.into_children());
        self
    }

    pub fn leading_child(mut self, node: impl IntoNode) -> Self {
        self.leading.push(node.into_node());
        self
    }

    pub fn primary(mut self, nodes: impl IntoChildren) -> Self {
        self.primary.extend(nodes.into_children());
        self
    }

    pub fn primary_child(mut self, node: impl IntoNode) -> Self {
        self.primary.push(node.into_node());
        self
    }

    pub fn secondary(mut self, nodes: impl IntoChildren) -> Self {
        self.secondary.extend(nodes.into_children());
        self
    }

    pub fn secondary_child(mut self, node: impl IntoNode) -> Self {
        self.secondary.push(node.into_node());
        self
    }

    pub fn trailing(mut self, nodes: impl IntoChildren) -> Self {
        self.trailing.extend(nodes.into_children());
        self
    }

    pub fn trailing_child(mut self, node: impl IntoNode) -> Self {
        self.trailing.push(node.into_node());
        self
    }

    pub fn action(mut self, action: impl Into<ActionRef>) -> Self {
        self.action = Some(action.into());
        self
    }

    pub fn build(self) -> ItemNode {
        ItemNode {
            id: self.id,
            meta: self.meta,
            leading: self.leading,
            primary: self.primary,
            secondary: self.secondary,
            trailing: self.trailing,
            action: self.action,
        }
    }
}

impl IntoItem for ItemBuilder {
    fn into_item(self) -> ItemNode {
        self.build()
    }
}

impl IntoNode for ItemBuilder {
    fn into_node(self) -> UiNode {
        UiNode::Item(self.build())
    }
}

impl IntoChildren for ItemBuilder {
    fn into_children(self) -> Vec<UiNode> {
        vec![self.into_node()]
    }
}

/// Creates a list of semantic collection items.
pub fn list(items: impl IntoItems) -> ListBuilder {
    ListBuilder {
        id: None,
        items: items.into_items(),
        density: None,
        continuation: None,
    }
}

pub struct ListBuilder {
    id: Option<String>,
    items: Vec<ItemNode>,
    density: Option<ListDensity>,
    continuation: Option<CollectionContinuation>,
}

impl ListBuilder {
    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    pub fn density(mut self, density: ListDensity) -> Self {
        self.density = Some(density);
        self
    }

    pub fn continuation(mut self, continuation: CollectionContinuation) -> Self {
        self.continuation = Some(continuation);
        self
    }

    pub fn items(mut self, items: impl IntoItems) -> Self {
        self.items.extend(items.into_items());
        self
    }

    pub fn item(mut self, item: impl IntoItem) -> Self {
        self.items.push(item.into_item());
        self
    }
}

impl IntoNode for ListBuilder {
    fn into_node(self) -> UiNode {
        UiNode::List(ListNode {
            base: NodeBase {
                id: self.id,
                meta: None,
            },
            items: self.items,
            density: self.density,
            continuation: self.continuation,
        })
    }
}

impl IntoChildren for ListBuilder {
    fn into_children(self) -> Vec<UiNode> {
        vec![self.into_node()]
    }
}

/// Starts a user-invokable action node builder.
///
/// Actions describe intent; renderers decide whether they become buttons, menu
/// items, key bindings, or terminal commands. Core action types may be handled by
/// the host, while custom action strings are dispatched back to the plugin.
pub fn action(label: impl Into<StringOrExpr>, action: impl Into<ActionRef>) -> ActionBuilder {
    ActionBuilder {
        id: None,
        label: label.into(),
        action: action.into(),
        intent: None,
        variant: None,
        leading_icon: None,
        disabled: None,
    }
}

pub struct ActionBuilder {
    id: Option<String>,
    label: StringOrExpr,
    action: ActionRef,
    intent: Option<ActionIntent>,
    variant: Option<ActionVariant>,
    leading_icon: Option<String>,
    disabled: Option<BoolOrExpr>,
}

impl ActionBuilder {
    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    pub fn intent(mut self, intent: ActionIntent) -> Self {
        self.intent = Some(intent);
        self
    }

    pub fn variant(mut self, variant: ActionVariant) -> Self {
        self.variant = Some(variant);
        self
    }

    pub fn leading_icon(mut self, leading_icon: impl Into<String>) -> Self {
        self.leading_icon = Some(leading_icon.into());
        self
    }

    pub fn disabled(mut self, disabled: impl Into<BoolOrExpr>) -> Self {
        self.disabled = Some(disabled.into());
        self
    }

    pub fn build(self) -> ActionNode {
        ActionNode {
            base: NodeBase {
                id: self.id,
                meta: None,
            },
            label: self.label,
            action: self.action,
            intent: self.intent,
            variant: self.variant,
            leading_icon: self.leading_icon,
            disabled: self.disabled,
        }
    }
}

impl IntoAction for ActionBuilder {
    fn into_action(self) -> ActionNode {
        self.build()
    }
}

impl IntoNode for ActionBuilder {
    fn into_node(self) -> UiNode {
        UiNode::Action(self.build())
    }
}

impl IntoChildren for ActionBuilder {
    fn into_children(self) -> Vec<UiNode> {
        vec![self.into_node()]
    }
}

/// Creates a container for grouped actions.
pub fn actions() -> ActionsBuilder {
    ActionsBuilder {
        id: None,
        align: None,
        children: Vec::new(),
    }
}

pub struct ActionsBuilder {
    id: Option<String>,
    align: Option<Align>,
    children: Vec<ActionNode>,
}

impl ActionsBuilder {
    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    pub fn align(mut self, align: Align) -> Self {
        self.align = Some(align);
        self
    }

    pub fn children(mut self, children: impl IntoActions) -> Self {
        self.children.extend(children.into_actions());
        self
    }

    pub fn child(mut self, child: impl IntoAction) -> Self {
        self.children.push(child.into_action());
        self
    }
}

impl IntoNode for ActionsBuilder {
    fn into_node(self) -> UiNode {
        UiNode::Actions(ActionsNode {
            base: NodeBase {
                id: self.id,
                meta: None,
            },
            align: self.align,
            children: self.children,
        })
    }
}

impl IntoChildren for ActionsBuilder {
    fn into_children(self) -> Vec<UiNode> {
        vec![self.into_node()]
    }
}

/// Creates an inline disclosure region controlled by local state.
pub fn disclosure(binding: impl Into<String>, label: impl Into<StringOrExpr>) -> DisclosureBuilder {
    DisclosureBuilder {
        id: None,
        binding: binding.into(),
        label: label.into(),
        label_expanded: None,
        children: Vec::new(),
    }
}

pub struct DisclosureBuilder {
    id: Option<String>,
    binding: String,
    label: StringOrExpr,
    label_expanded: Option<StringOrExpr>,
    children: Vec<UiNode>,
}

impl DisclosureBuilder {
    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    pub fn label_expanded(mut self, label: impl Into<StringOrExpr>) -> Self {
        self.label_expanded = Some(label.into());
        self
    }

    pub fn children(mut self, children: impl IntoChildren) -> Self {
        self.children.extend(children.into_children());
        self
    }

    pub fn child(mut self, child: impl IntoNode) -> Self {
        self.children.push(child.into_node());
        self
    }
}

impl IntoNode for DisclosureBuilder {
    fn into_node(self) -> UiNode {
        UiNode::Disclosure(DisclosureNode {
            base: NodeBase {
                id: self.id,
                meta: None,
            },
            binding: self.binding,
            label: self.label,
            label_expanded: self.label_expanded,
            children: self.children,
        })
    }
}

impl IntoChildren for DisclosureBuilder {
    fn into_children(self) -> Vec<UiNode> {
        vec![self.into_node()]
    }
}

/// Creates a menu item for use inside `menu(...)`.
pub fn menu_item(label: impl Into<StringOrExpr>, action: impl Into<ActionRef>) -> MenuItemBuilder {
    MenuItemBuilder {
        id: None,
        label: label.into(),
        action: action.into(),
        selected: None,
        disabled: None,
    }
}

pub struct MenuItemBuilder {
    id: Option<String>,
    label: StringOrExpr,
    action: ActionRef,
    selected: Option<bool>,
    disabled: Option<BoolOrExpr>,
}

impl MenuItemBuilder {
    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = Some(selected);
        self
    }

    pub fn disabled(mut self, disabled: impl Into<BoolOrExpr>) -> Self {
        self.disabled = Some(disabled.into());
        self
    }

    pub fn build(self) -> MenuItem {
        MenuItem {
            id: self.id,
            label: self.label,
            action: self.action,
            selected: self.selected,
            disabled: self.disabled,
        }
    }
}

impl IntoMenuItem for MenuItemBuilder {
    fn into_menu_item(self) -> MenuItem {
        self.build()
    }
}

/// Creates a semantic popup menu trigger and option list.
pub fn menu(label: impl Into<StringOrExpr>) -> MenuBuilder {
    MenuBuilder {
        id: None,
        label: label.into(),
        items: Vec::new(),
        intent: None,
        align: None,
    }
}

pub struct MenuBuilder {
    id: Option<String>,
    label: StringOrExpr,
    items: Vec<MenuItem>,
    intent: Option<ActionIntent>,
    align: Option<Align>,
}

impl MenuBuilder {
    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    pub fn intent(mut self, intent: ActionIntent) -> Self {
        self.intent = Some(intent);
        self
    }

    pub fn align(mut self, align: Align) -> Self {
        self.align = Some(align);
        self
    }

    pub fn items(mut self, items: impl IntoMenuItems) -> Self {
        self.items.extend(items.into_menu_items());
        self
    }

    pub fn item(mut self, item: impl IntoMenuItem) -> Self {
        self.items.push(item.into_menu_item());
        self
    }
}

impl IntoNode for MenuBuilder {
    fn into_node(self) -> UiNode {
        UiNode::Menu(MenuNode {
            base: NodeBase {
                id: self.id,
                meta: None,
            },
            label: self.label,
            items: self.items,
            intent: self.intent,
            align: self.align,
        })
    }
}

impl IntoChildren for MenuBuilder {
    fn into_children(self) -> Vec<UiNode> {
        vec![self.into_node()]
    }
}

/// Creates a select-like option for use in certain input kinds.
pub fn select_choice(label: impl Into<String>, value: impl IntoPrimitive) -> SelectChoice {
    SelectChoice {
        label: label.into(),
        value: value.into_primitive(),
        disabled: None,
    }
}

/// Creates an input field description.
pub fn input(
    name: impl Into<String>,
    input_kind: InputKind,
    label: impl Into<StringOrExpr>,
) -> InputBuilder {
    InputBuilder {
        id: None,
        name: name.into(),
        input_kind,
        label: label.into(),
        value: None,
        placeholder: None,
        help_text: None,
        required: None,
        disabled: None,
        options: Vec::new(),
        constraints: None,
    }
}

pub struct InputBuilder {
    id: Option<String>,
    name: String,
    input_kind: InputKind,
    label: StringOrExpr,
    value: Option<PrimitiveOrExpr>,
    placeholder: Option<StringOrExpr>,
    help_text: Option<StringOrExpr>,
    required: Option<bool>,
    disabled: Option<BoolOrExpr>,
    options: Vec<SelectChoice>,
    constraints: Option<InputConstraints>,
}

impl InputBuilder {
    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    pub fn value(mut self, value: impl IntoPrimitiveOrExpr) -> Self {
        self.value = Some(value.into_primitive_or_expr());
        self
    }

    pub fn placeholder(mut self, placeholder: impl Into<StringOrExpr>) -> Self {
        self.placeholder = Some(placeholder.into());
        self
    }

    pub fn help_text(mut self, help_text: impl Into<StringOrExpr>) -> Self {
        self.help_text = Some(help_text.into());
        self
    }

    pub fn required(mut self, required: bool) -> Self {
        self.required = Some(required);
        self
    }

    pub fn disabled(mut self, disabled: impl Into<BoolOrExpr>) -> Self {
        self.disabled = Some(disabled.into());
        self
    }

    pub fn options(mut self, options: Vec<SelectChoice>) -> Self {
        self.options.extend(options);
        self
    }

    pub fn option(mut self, option: SelectChoice) -> Self {
        self.options.push(option);
        self
    }

    pub fn constraints(mut self, constraints: InputConstraints) -> Self {
        self.constraints = Some(constraints);
        self
    }
}

impl IntoNode for InputBuilder {
    fn into_node(self) -> UiNode {
        UiNode::Input(InputNode {
            base: NodeBase {
                id: self.id,
                meta: None,
            },
            name: self.name,
            input_kind: self.input_kind,
            label: self.label,
            value: self.value,
            placeholder: self.placeholder,
            help_text: self.help_text,
            required: self.required,
            disabled: self.disabled,
            options: self.options,
            constraints: self.constraints,
        })
    }
}

impl IntoChildren for InputBuilder {
    fn into_children(self) -> Vec<UiNode> {
        vec![self.into_node()]
    }
}

/// Creates a semantic form container.
pub fn form(name: impl Into<String>) -> FormBuilder {
    FormBuilder {
        id: None,
        name: name.into(),
        children: Vec::new(),
        submit: None,
    }
}

pub struct FormBuilder {
    id: Option<String>,
    name: String,
    children: Vec<UiNode>,
    submit: Option<ActionRef>,
}

impl FormBuilder {
    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    pub fn children(mut self, children: impl IntoChildren) -> Self {
        self.children.extend(children.into_children());
        self
    }

    pub fn child(mut self, child: impl IntoNode) -> Self {
        self.children.push(child.into_node());
        self
    }

    pub fn submit(mut self, action: impl Into<ActionRef>) -> Self {
        self.submit = Some(action.into());
        self
    }
}

impl IntoNode for FormBuilder {
    fn into_node(self) -> UiNode {
        UiNode::Form(FormNode {
            base: NodeBase {
                id: self.id,
                meta: None,
            },
            name: self.name,
            children: self.children,
            submit: self.submit,
        })
    }
}

impl IntoChildren for FormBuilder {
    fn into_children(self) -> Vec<UiNode> {
        vec![self.into_node()]
    }
}

/// Creates a feedback/status block.
pub fn status(severity: StatusSeverity, message: impl Into<StringOrExpr>) -> StatusBuilder {
    StatusBuilder {
        id: None,
        severity,
        message: message.into(),
        title: None,
        actions: Vec::new(),
    }
}

pub struct StatusBuilder {
    id: Option<String>,
    severity: StatusSeverity,
    message: StringOrExpr,
    title: Option<StringOrExpr>,
    actions: Vec<ActionNode>,
}

impl StatusBuilder {
    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    pub fn title(mut self, title: impl Into<StringOrExpr>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn actions(mut self, actions: impl IntoActions) -> Self {
        self.actions.extend(actions.into_actions());
        self
    }

    pub fn action(mut self, action: impl IntoAction) -> Self {
        self.actions.push(action.into_action());
        self
    }
}

impl IntoNode for StatusBuilder {
    fn into_node(self) -> UiNode {
        UiNode::Status(StatusNode {
            base: NodeBase {
                id: self.id,
                meta: None,
            },
            severity: self.severity,
            title: self.title,
            message: self.message,
            actions: self.actions,
        })
    }
}

impl IntoChildren for StatusBuilder {
    fn into_children(self) -> Vec<UiNode> {
        vec![self.into_node()]
    }
}

/// Creates an empty-state block.
pub fn empty(title: impl Into<StringOrExpr>) -> EmptyBuilder {
    EmptyBuilder {
        id: None,
        icon: None,
        title: title.into(),
        message: None,
        actions: Vec::new(),
    }
}

pub struct EmptyBuilder {
    id: Option<String>,
    icon: Option<String>,
    title: StringOrExpr,
    message: Option<StringOrExpr>,
    actions: Vec<ActionNode>,
}

impl EmptyBuilder {
    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    pub fn icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    pub fn message(mut self, message: impl Into<StringOrExpr>) -> Self {
        self.message = Some(message.into());
        self
    }

    pub fn actions(mut self, actions: impl IntoActions) -> Self {
        self.actions.extend(actions.into_actions());
        self
    }

    pub fn action(mut self, action: impl IntoAction) -> Self {
        self.actions.push(action.into_action());
        self
    }
}

impl IntoNode for EmptyBuilder {
    fn into_node(self) -> UiNode {
        UiNode::Empty(EmptyStateNode {
            base: NodeBase {
                id: self.id,
                meta: None,
            },
            icon: self.icon,
            title: self.title,
            message: self.message,
            actions: self.actions,
        })
    }
}

impl IntoChildren for EmptyBuilder {
    fn into_children(self) -> Vec<UiNode> {
        vec![self.into_node()]
    }
}

/// Creates a loading indicator block.
pub fn loading() -> LoadingBuilder {
    LoadingBuilder {
        id: None,
        label: None,
        progress: None,
    }
}

pub struct LoadingBuilder {
    id: Option<String>,
    label: Option<StringOrExpr>,
    progress: Option<NumberOrExpr>,
}

impl LoadingBuilder {
    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    pub fn label(mut self, label: impl Into<StringOrExpr>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn progress(mut self, progress: impl Into<NumberOrExpr>) -> Self {
        self.progress = Some(progress.into());
        self
    }
}

impl IntoNode for LoadingBuilder {
    fn into_node(self) -> UiNode {
        UiNode::Loading(LoadingNode {
            base: NodeBase {
                id: self.id,
                meta: None,
            },
            label: self.label,
            progress: self.progress,
        })
    }
}

impl IntoChildren for LoadingBuilder {
    fn into_children(self) -> Vec<UiNode> {
        vec![self.into_node()]
    }
}

/// Creates an inline plugin anchor with optional fallback content.
///
/// `SlotNode`s let a plugin or host declare a named extension point inside a
/// screen. The host resolves contributions before mounting so framework and TUI
/// renderers normally receive an already-injected tree.
pub fn slot(name: impl Into<String>) -> SlotBuilder {
    SlotBuilder {
        id: None,
        name: name.into(),
        fallback: None,
    }
}

pub struct SlotBuilder {
    id: Option<String>,
    name: String,
    fallback: Option<UiNode>,
}

impl SlotBuilder {
    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    pub fn fallback(mut self, fallback: impl IntoNode) -> Self {
        self.fallback = Some(fallback.into_node());
        self
    }
}

impl IntoNode for SlotBuilder {
    fn into_node(self) -> UiNode {
        UiNode::Slot(SlotNode {
            base: NodeBase {
                id: self.id,
                meta: None,
            },
            name: self.name,
            fallback: self.fallback.map(Box::new),
        })
    }
}

impl IntoChildren for SlotBuilder {
    fn into_children(self) -> Vec<UiNode> {
        vec![self.into_node()]
    }
}

// --- Macro ---

#[macro_export]
macro_rules! ui_children {
    ($($expr:expr),* $(,)?) => {{
        let mut nodes = Vec::new();
        $(
            nodes.extend($crate::core::dsl::IntoChildren::into_children($expr));
        )*
        nodes
    }};
}
