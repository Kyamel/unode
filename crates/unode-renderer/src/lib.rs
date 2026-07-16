//! Renderer-definition SDK for Rust hosts — the Rust counterpart of the
//! `unode-renderer` TypeScript package.
//!
//! Hosts map semantic `UiNode`s onto their presentation stack with *recipes*:
//! per-node-kind functions that measure (rows needed at a width) and render
//! (paint into a region of a surface). The machinery here is stack-agnostic —
//! the surface type is chosen by a [`Backend`] implementation, so the same
//! recipe model drives ratatui (see the `tui-renderer` crate), a hand-rolled
//! ANSI writer, or anything else cell-grid shaped.
//!
//! ```ignore
//! let renderer = tui_renderer::ratatui_renderer()   // defaults included
//!     .recipe(NodeKind::Text, my_text_recipe)       // override one node kind
//!     .fallback(my_fallback)
//!     .build();
//! ```

use std::collections::HashMap;
use std::sync::Arc;

use unode::core::ast::{self as ast, UiNode};

/// Chooses the drawing surface recipes paint on. The generic-associated
/// lifetime lets backends hand out per-frame surfaces (e.g. `ratatui::Frame`).
pub trait Backend {
    type Surface<'f>;
}

/// A rectangular region of the surface, in character cells.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Region {
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
}

impl Region {
    pub fn new(x: u16, y: u16, width: u16, height: u16) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.width == 0 || self.height == 0
    }
}

/// Node type key — the closed `UiNode` set, mirroring the string keys the
/// web renderer uses (`.recipe("text", ...)`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NodeKind {
    Section,
    Stack,
    Inline,
    Grid,
    Scroll,
    Text,
    Value,
    Icon,
    Badge,
    Divider,
    Media,
    Pressable,
    Item,
    List,
    Action,
    Actions,
    Disclosure,
    Menu,
    Input,
    Form,
    Status,
    Empty,
    Loading,
    Conditional,
    Slot,
}

impl NodeKind {
    pub fn of(node: &UiNode) -> Self {
        match node {
            UiNode::Section(_) => Self::Section,
            UiNode::Stack(_) => Self::Stack,
            UiNode::Inline(_) => Self::Inline,
            UiNode::Grid(_) => Self::Grid,
            UiNode::Scroll(_) => Self::Scroll,
            UiNode::Text(_) => Self::Text,
            UiNode::Value(_) => Self::Value,
            UiNode::Icon(_) => Self::Icon,
            UiNode::Badge(_) => Self::Badge,
            UiNode::Divider(_) => Self::Divider,
            UiNode::Media(_) => Self::Media,
            UiNode::Pressable(_) => Self::Pressable,
            UiNode::Item(_) => Self::Item,
            UiNode::List(_) => Self::List,
            UiNode::Action(_) => Self::Action,
            UiNode::Actions(_) => Self::Actions,
            UiNode::Disclosure(_) => Self::Disclosure,
            UiNode::Menu(_) => Self::Menu,
            UiNode::Input(_) => Self::Input,
            UiNode::Form(_) => Self::Form,
            UiNode::Status(_) => Self::Status,
            UiNode::Empty(_) => Self::Empty,
            UiNode::Loading(_) => Self::Loading,
            UiNode::Conditional(_) => Self::Conditional,
            UiNode::Slot(_) => Self::Slot,
        }
    }
}

pub type MeasureFn<B> = Arc<dyn Fn(&MeasureCtx<'_, B>, &UiNode, u16) -> u16 + Send + Sync>;
pub type RenderFn<B> =
    Arc<dyn for<'a, 'f> Fn(&mut RenderCtx<'a, 'f, B>, &UiNode, Region) + Send + Sync>;

/// How one node kind is measured and painted.
pub struct Recipe<B: Backend> {
    pub measure: MeasureFn<B>,
    pub render: RenderFn<B>,
}

impl<B: Backend> Clone for Recipe<B> {
    fn clone(&self) -> Self {
        Self {
            measure: self.measure.clone(),
            render: self.render.clone(),
        }
    }
}

impl<B: Backend> Recipe<B> {
    pub fn new(
        measure: impl Fn(&MeasureCtx<'_, B>, &UiNode, u16) -> u16 + Send + Sync + 'static,
        render: impl for<'a, 'f> Fn(&mut RenderCtx<'a, 'f, B>, &UiNode, Region) + Send + Sync + 'static,
    ) -> Self {
        Self {
            measure: Arc::new(measure),
            render: Arc::new(render),
        }
    }

    /// A recipe that reserves `rows` and paints nothing. Useful as a neutral
    /// fallback before a backend installs a real one.
    pub fn blank(rows: u16) -> Self {
        Self {
            measure: Arc::new(move |_, _, _| rows),
            render: Arc::new(|_, _, _| {}),
        }
    }
}

/// Generates typed recipe constructors: each pairs the [`NodeKind`] with
/// closures that receive the concrete node struct, eliminating the manual
/// `let UiNode::X(..) = node else ...` downcast. They return the
/// `(NodeKind, Recipe)` entry expected by [`RendererBuilder::recipes`]:
///
/// ```ignore
/// ratatui_renderer().recipes([Recipe::text(
///     |_, node, width| measure_text(node, width),
///     |ctx: &mut TuiRenderCtx<'_, '_>, node, area| paint_text(ctx, node, area),
/// )])
/// ```
macro_rules! typed_recipe_constructors {
    ($($method:ident => $kind:ident / $variant:ident : $node_ty:ty;)*) => {
        impl<B: Backend> Recipe<B> {
            $(
                pub fn $method(
                    measure: impl Fn(&MeasureCtx<'_, B>, &$node_ty, u16) -> u16
                    + Send
                    + Sync
                    + 'static,
                    render: impl for<'a, 'f> Fn(&mut RenderCtx<'a, 'f, B>, &$node_ty, Region)
                    + Send
                    + Sync
                    + 'static,
                ) -> (NodeKind, Recipe<B>) {
                    (
                        NodeKind::$kind,
                        Recipe::new(
                            move |ctx, node, width| match node {
                                UiNode::$variant(node) => measure(ctx, node, width),
                                _ => 1,
                            },
                            move |ctx: &mut RenderCtx<'_, '_, B>, node, region| {
                                if let UiNode::$variant(node) = node {
                                    render(ctx, node, region);
                                }
                            },
                        ),
                    )
                }
            )*
        }
    };
}

typed_recipe_constructors! {
    section => Section / Section: ast::SectionNode;
    stack => Stack / Stack: ast::StackNode;
    inline => Inline / Inline: ast::InlineNode;
    grid => Grid / Grid: ast::GridNode;
    scroll => Scroll / Scroll: ast::ScrollNode;
    text => Text / Text: ast::TextNode;
    value => Value / Value: ast::ValueNode;
    icon => Icon / Icon: ast::IconNode;
    badge => Badge / Badge: ast::BadgeNode;
    divider => Divider / Divider: ast::DividerNode;
    media => Media / Media: ast::MediaNode;
    pressable => Pressable / Pressable: ast::PressableNode;
    item => Item / Item: ast::ItemNode;
    list => List / List: ast::ListNode;
    action => Action / Action: ast::ActionNode;
    actions => Actions / Actions: ast::ActionsNode;
    disclosure => Disclosure / Disclosure: ast::DisclosureNode;
    menu => Menu / Menu: ast::MenuNode;
    input => Input / Input: ast::InputNode;
    form => Form / Form: ast::FormNode;
    status => Status / Status: ast::StatusNode;
    empty => Empty / Empty: ast::EmptyStateNode;
    loading => Loading / Loading: ast::LoadingNode;
    conditional => Conditional / Conditional: ast::ConditionalNode;
    slot => Slot / Slot: ast::SlotNode;
}

/// The resolved recipe set: per-kind recipes plus a fallback.
pub struct RendererSpec<B: Backend> {
    recipes: HashMap<NodeKind, Recipe<B>>,
    fallback: Recipe<B>,
}

impl<B: Backend> RendererSpec<B> {
    pub fn recipe_for(&self, node: &UiNode) -> Recipe<B> {
        self.recipes
            .get(&NodeKind::of(node))
            .unwrap_or(&self.fallback)
            .clone()
    }
}

/// Tracks which interactive element currently has focus while a screen is
/// painted. Recipes call [`RenderCtx::focus_next`] once per interactive
/// element, in render order, and receive whether that element is focused.
#[derive(Debug, Clone, Copy, Default)]
pub struct FocusCursor {
    focused: Option<usize>,
    next: usize,
}

impl FocusCursor {
    pub fn new(focused: Option<usize>) -> Self {
        Self { focused, next: 0 }
    }

    fn consume(&mut self, enabled: bool) -> bool {
        if !enabled {
            return false;
        }
        let index = self.next;
        self.next += 1;
        self.focused == Some(index)
    }
}

/// Measurement pass context: recurses through the recipe set.
pub struct MeasureCtx<'a, B: Backend> {
    spec: &'a RendererSpec<B>,
}

impl<B: Backend> MeasureCtx<'_, B> {
    /// Rows `node` needs when laid out at `width`.
    pub fn measure(&self, node: &UiNode, width: u16) -> u16 {
        let recipe = self.spec.recipe_for(node);
        (recipe.measure)(self, node, width).max(1)
    }
}

/// Render pass context: owns the surface, the focus cursor, and recursion.
pub struct RenderCtx<'a, 'f, B: Backend> {
    spec: &'a RendererSpec<B>,
    pub surface: &'a mut B::Surface<'f>,
    focus: FocusCursor,
}

impl<B: Backend> RenderCtx<'_, '_, B> {
    /// Renders `node` into `region` through the recipe set.
    pub fn render(&mut self, node: &UiNode, region: Region) {
        let recipe = self.spec.recipe_for(node);
        (recipe.render)(self, node, region);
    }

    /// Rows `node` needs when laid out at `width`.
    pub fn measure(&self, node: &UiNode, width: u16) -> u16 {
        MeasureCtx { spec: self.spec }.measure(node, width)
    }

    /// Claims the next interactive slot; returns whether it is focused.
    /// Disabled elements pass `enabled: false` and never claim a slot.
    pub fn focus_next(&mut self, enabled: bool) -> bool {
        self.focus.consume(enabled)
    }
}

/// A built renderer, cheap to clone and reusable across frames.
#[derive(Clone)]
pub struct Renderer<B: Backend> {
    spec: Arc<RendererSpec<B>>,
}

impl<B: Backend> Renderer<B> {
    /// Rows `node` needs when laid out at `width`.
    pub fn measure(&self, node: &UiNode, width: u16) -> u16 {
        MeasureCtx { spec: &self.spec }.measure(node, width)
    }

    /// Starts a render pass over `surface`. `focused` selects which
    /// interactive element (in render order) is highlighted.
    pub fn pass<'a, 'f>(
        &'a self,
        surface: &'a mut B::Surface<'f>,
        focused: Option<usize>,
    ) -> RenderCtx<'a, 'f, B> {
        RenderCtx {
            spec: &self.spec,
            surface,
            focus: FocusCursor::new(focused),
        }
    }
}

/// Fluent builder mirroring the web `defineRenderer()` surface.
pub struct RendererBuilder<B: Backend> {
    recipes: HashMap<NodeKind, Recipe<B>>,
    fallback: Recipe<B>,
}

/// Entry point: start from an empty recipe set. Backends typically wrap this
/// with a seeded variant (e.g. `tui_renderer::ratatui_renderer()`).
pub fn define_renderer<B: Backend>() -> RendererBuilder<B> {
    RendererBuilder {
        recipes: HashMap::new(),
        fallback: Recipe::blank(1),
    }
}

impl<B: Backend> RendererBuilder<B> {
    /// Sets (or overrides) the recipe for a node kind.
    pub fn recipe(mut self, kind: NodeKind, recipe: Recipe<B>) -> Self {
        self.recipes.insert(kind, recipe);
        self
    }

    /// Sets many recipes at once.
    pub fn recipes(mut self, entries: impl IntoIterator<Item = (NodeKind, Recipe<B>)>) -> Self {
        self.recipes.extend(entries);
        self
    }

    /// Recipe used for node kinds without a dedicated recipe.
    pub fn fallback(mut self, recipe: Recipe<B>) -> Self {
        self.fallback = recipe;
        self
    }

    pub fn build(self) -> Renderer<B> {
        Renderer {
            spec: Arc::new(RendererSpec {
                recipes: self.recipes,
                fallback: self.fallback,
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use unode::core::dsl::{self as ui, IntoNode};

    /// A backend that writes plain lines into a string buffer — the "manual
    /// terminal without ratatui" case.
    struct PlainText;

    #[derive(Default)]
    struct LineBuffer {
        lines: Vec<(u16, String)>,
    }

    impl Backend for PlainText {
        type Surface<'f> = LineBuffer;
    }

    fn text_recipe() -> Recipe<PlainText> {
        Recipe::new(
            |_, _, _| 1,
            |ctx: &mut RenderCtx<'_, '_, PlainText>, node, region| {
                let UiNode::Text(text) = node else { return };
                let unode::core::ast::StringOrExpr::Value(content) = &text.content else {
                    return;
                };
                ctx.surface.lines.push((region.y, content.clone()));
            },
        )
    }

    fn stack_recipe() -> Recipe<PlainText> {
        Recipe::new(
            |ctx, node, width| {
                let UiNode::Stack(stack) = node else { return 1 };
                stack
                    .children
                    .iter()
                    .map(|child| ctx.measure(child, width))
                    .sum()
            },
            |ctx: &mut RenderCtx<'_, '_, PlainText>, node, region| {
                let UiNode::Stack(stack) = node else { return };
                let mut y = region.y;
                for child in &stack.children {
                    let rows = ctx.measure(child, region.width);
                    ctx.render(child, Region::new(region.x, y, region.width, rows));
                    y += rows;
                }
            },
        )
    }

    fn sample_stack() -> UiNode {
        ui::stack()
            .children([
                ui::text("first").into_node(),
                ui::text("second").into_node(),
                ui::badge("misc").into_node(),
            ])
            .into_node()
    }

    #[test]
    fn renders_through_recipes_with_fallback() {
        let renderer = define_renderer::<PlainText>()
            .recipe(NodeKind::Text, text_recipe())
            .recipe(NodeKind::Stack, stack_recipe())
            .fallback(Recipe::new(
                |_, _, _| 1,
                |ctx: &mut RenderCtx<'_, '_, PlainText>, _, region| {
                    ctx.surface.lines.push((region.y, "<fallback>".to_string()))
                },
            ))
            .build();

        let node = sample_stack();
        assert_eq!(renderer.measure(&node, 40), 3);

        let mut surface = LineBuffer::default();
        renderer
            .pass(&mut surface, None)
            .render(&node, Region::new(0, 0, 40, 3));

        assert_eq!(
            surface.lines,
            vec![
                (0, "first".to_string()),
                (1, "second".to_string()),
                (2, "<fallback>".to_string()),
            ]
        );
    }

    #[test]
    fn overriding_a_recipe_changes_output() {
        let renderer = define_renderer::<PlainText>()
            .recipe(NodeKind::Text, text_recipe())
            .recipe(NodeKind::Stack, stack_recipe())
            .recipe(
                NodeKind::Text,
                Recipe::new(
                    |_, _, _| 1,
                    |ctx: &mut RenderCtx<'_, '_, PlainText>, _, region| {
                        ctx.surface.lines.push((region.y, "custom".to_string()))
                    },
                ),
            )
            .build();

        let mut surface = LineBuffer::default();
        renderer
            .pass(&mut surface, None)
            .render(&ui::text("ignored").into_node(), Region::new(0, 0, 10, 1));
        assert_eq!(surface.lines, vec![(0, "custom".to_string())]);
    }

    #[test]
    fn focus_cursor_assigns_slots_in_render_order() {
        let mut cursor = FocusCursor::new(Some(1));
        assert!(!cursor.consume(true)); // slot 0
        assert!(!cursor.consume(false)); // disabled: no slot
        assert!(cursor.consume(true)); // slot 1 → focused
        assert!(!cursor.consume(true)); // slot 2
    }
}
