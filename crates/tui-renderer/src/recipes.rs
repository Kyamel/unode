//! The ratatui specialization of the `unode-renderer` SDK: a [`Backend`]
//! whose surface is a `ratatui::Frame`, plus default recipes for the node
//! set. Hosts start from [`ratatui_renderer`] and override per node kind,
//! mirroring the web `defineRenderer()` flow.

use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use unode::core::ast::{ActionNode, BoolOrExpr, UiNode};
use unode_renderer::{
    Backend, NodeKind, Recipe, Region, RenderCtx, Renderer, RendererBuilder, define_renderer,
};

use crate::nodes::{actions, list, section, stack, status, text};
use crate::util::render_string_or_expr;

/// Backend whose surface is the per-draw ratatui frame.
pub struct RatatuiBackend;

impl Backend for RatatuiBackend {
    type Surface<'f> = Frame<'f>;
}

pub type TuiRenderer = Renderer<RatatuiBackend>;
pub type TuiRecipe = Recipe<RatatuiBackend>;
pub type TuiRenderCtx<'a, 'f> = RenderCtx<'a, 'f, RatatuiBackend>;

pub fn rect(region: Region) -> Rect {
    Rect::new(region.x, region.y, region.width, region.height)
}

pub fn region(rect: Rect) -> Region {
    Region::new(rect.x, rect.y, rect.width, rect.height)
}

/// Renders `children` stacked vertically, each sized by its measure.
pub fn render_vertical_children(ctx: &mut TuiRenderCtx<'_, '_>, children: &[UiNode], area: Rect) {
    if children.is_empty() || area.height == 0 || area.width == 0 {
        return;
    }

    let constraints = children
        .iter()
        .map(|child| Constraint::Length(ctx.measure(child, area.width)))
        .collect::<Vec<_>>();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(area);

    for (child, chunk) in children.iter().zip(chunks.iter()) {
        ctx.render(child, region(*chunk));
    }
}

fn render_inline_children(ctx: &mut TuiRenderCtx<'_, '_>, children: &[UiNode], area: Rect) {
    if children.is_empty() || area.height == 0 || area.width == 0 {
        return;
    }

    let constraints = children
        .iter()
        .map(|child| match child {
            UiNode::Action(action) => Constraint::Length(actions::button_width(action)),
            _ => Constraint::Fill(1),
        })
        .collect::<Vec<_>>();
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(constraints)
        .split(area);

    for (child, chunk) in children.iter().zip(chunks.iter()) {
        ctx.render(child, region(*chunk));
    }
}

fn is_disabled(node: &ActionNode) -> bool {
    matches!(node.disabled, Some(BoolOrExpr::Value(true)))
}

fn text_recipe() -> (NodeKind, TuiRecipe) {
    Recipe::text(
        |_, node, width| text::measure(node, width),
        |ctx: &mut TuiRenderCtx<'_, '_>, node, area| {
            text::render(ctx.surface, rect(area), node);
        },
    )
}

fn stack_recipe() -> (NodeKind, TuiRecipe) {
    Recipe::stack(
        |ctx, node, width| {
            let children = node
                .children
                .iter()
                .map(|child| ctx.measure(child, width))
                .sum::<u16>();
            children + stack::gap_lines(node) * node.children.len().saturating_sub(1) as u16
        },
        |ctx: &mut TuiRenderCtx<'_, '_>, node, area| {
            if node.children.is_empty() {
                return;
            }
            let area = rect(area);

            let gap = stack::gap_lines(node);
            let mut constraints = Vec::new();
            for (index, child) in node.children.iter().enumerate() {
                constraints.push(Constraint::Length(ctx.measure(child, area.width)));
                if gap > 0 && index + 1 < node.children.len() {
                    constraints.push(Constraint::Length(gap));
                }
            }

            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints(constraints)
                .split(area);

            let mut chunk_index = 0;
            for child in &node.children {
                ctx.render(child, region(chunks[chunk_index]));
                chunk_index += 1 + usize::from(gap > 0);
            }
        },
    )
}

fn section_recipe() -> (NodeKind, TuiRecipe) {
    Recipe::section(
        |ctx, node, width| {
            let child_total = node
                .children
                .iter()
                .map(|child| ctx.measure(child, width.saturating_sub(2)))
                .sum::<u16>();
            2 + section::measure_header(node) + child_total
        },
        |ctx: &mut TuiRenderCtx<'_, '_>, node, area| {
            let inner = section::render_block(ctx.surface, rect(area), node);

            let mut constraints = Vec::new();
            if let Some(description) = &node.description {
                let desc = render_string_or_expr(description);
                constraints.push(Constraint::Length(desc.lines().count().max(1) as u16));
            }
            for child in &node.children {
                constraints.push(Constraint::Length(ctx.measure(child, inner.width)));
            }
            if constraints.is_empty() {
                return;
            }

            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints(constraints)
                .split(inner);

            let mut offset = 0;
            if let Some(description) = &node.description {
                ctx.surface.render_widget(
                    Paragraph::new(render_string_or_expr(description))
                        .style(ratatui::style::Style::default().fg(ratatui::style::Color::Gray))
                        .wrap(Wrap { trim: false }),
                    chunks[0],
                );
                offset = 1;
            }

            for (child, chunk) in node.children.iter().zip(chunks.iter().skip(offset)) {
                ctx.render(child, region(*chunk));
            }
        },
    )
}

fn action_recipe() -> (NodeKind, TuiRecipe) {
    Recipe::action(
        |_, _, _| 3,
        |ctx: &mut TuiRenderCtx<'_, '_>, node, area| {
            let active = !is_disabled(node) && ctx.focus_next(true);
            actions::render_action(ctx.surface, rect(area), node, active);
        },
    )
}

fn actions_recipe() -> (NodeKind, TuiRecipe) {
    Recipe::actions(
        |_, node, _| actions::measure_actions(node),
        |ctx: &mut TuiRenderCtx<'_, '_>, node, area| {
            let area = rect(area);
            if node.children.is_empty() || area.height == 0 || area.width == 0 {
                return;
            }

            let constraints = node
                .children
                .iter()
                .map(|child| Constraint::Length(actions::button_width(child)))
                .collect::<Vec<_>>();
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(constraints)
                .split(area);

            for (child, chunk) in node.children.iter().zip(chunks.iter()) {
                let active = !is_disabled(child) && ctx.focus_next(true);
                actions::render_action(ctx.surface, *chunk, child, active);
            }
        },
    )
}

fn status_recipe() -> (NodeKind, TuiRecipe) {
    Recipe::status(
        |_, node, _| status::measure(node),
        |ctx: &mut TuiRenderCtx<'_, '_>, node, area| {
            let focus = if node.actions.is_empty() {
                None
            } else {
                let mut selected = None;
                let mut enabled_index = 0usize;
                for action in &node.actions {
                    if is_disabled(action) {
                        continue;
                    }
                    if ctx.focus_next(true) {
                        selected = Some(enabled_index);
                    }
                    enabled_index += 1;
                }
                selected
            };
            status::render(ctx.surface, rect(area), node, focus);
        },
    )
}

fn list_recipe() -> (NodeKind, TuiRecipe) {
    Recipe::list(
        |_, node, _| list::measure(node),
        |ctx: &mut TuiRenderCtx<'_, '_>, node, area| {
            let mut selected = None;
            let mut interactive_index = 0usize;
            for item in &node.items {
                if item.action.is_none() {
                    continue;
                }
                if ctx.focus_next(true) {
                    selected = Some(interactive_index);
                }
                interactive_index += 1;
            }
            list::render(ctx.surface, rect(area), node, selected);
        },
    )
}

fn scroll_recipe() -> (NodeKind, TuiRecipe) {
    Recipe::scroll(
        |ctx, node, width| {
            node.children
                .iter()
                .map(|child| ctx.measure(child, width))
                .sum::<u16>()
                .max(3)
        },
        |ctx: &mut TuiRenderCtx<'_, '_>, node, area| {
            render_vertical_children(ctx, &node.children, rect(area));
        },
    )
}

fn inline_recipe() -> (NodeKind, TuiRecipe) {
    Recipe::inline(
        |_, node, _| node.children.len().max(1) as u16,
        |ctx: &mut TuiRenderCtx<'_, '_>, node, area| {
            render_inline_children(ctx, &node.children, rect(area));
        },
    )
}

fn grid_recipe() -> (NodeKind, TuiRecipe) {
    Recipe::grid(
        |_, node, _| node.children.len().max(1) as u16,
        |ctx: &mut TuiRenderCtx<'_, '_>, node, area| {
            render_inline_children(ctx, &node.children, rect(area));
        },
    )
}

/// Bordered JSON dump for node kinds without a dedicated recipe.
fn fallback_recipe() -> TuiRecipe {
    Recipe::new(
        |_, _, _| 3,
        |ctx: &mut TuiRenderCtx<'_, '_>, node, area| {
            let text =
                serde_json::to_string(node).unwrap_or_else(|_| "<unserializable node>".to_string());
            let widget = Paragraph::new(text)
                .block(Block::default().title("Node").borders(Borders::ALL))
                .wrap(Wrap { trim: false });
            ctx.surface.render_widget(widget, rect(area));
        },
    )
}

/// Starts a renderer builder seeded with the ratatui default recipes —
/// the Rust analog of the web `defineRenderer()`. Every default is
/// overridable per node kind; unknown kinds hit the fallback.
pub fn ratatui_renderer() -> RendererBuilder<RatatuiBackend> {
    define_renderer::<RatatuiBackend>()
        .recipes([
            text_recipe(),
            stack_recipe(),
            section_recipe(),
            action_recipe(),
            actions_recipe(),
            status_recipe(),
            list_recipe(),
            scroll_recipe(),
            inline_recipe(),
            grid_recipe(),
        ])
        .fallback(fallback_recipe())
}

#[cfg(test)]
mod tests {
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;
    use unode::core::dsl::{self as ui, IntoNode};
    use unode_renderer::Region;

    use super::*;

    fn row_text(terminal: &Terminal<TestBackend>, row: u16, width: u16) -> String {
        let buffer = terminal.backend().buffer();
        (0..width)
            .map(|x| buffer[(x, row)].symbol().to_string())
            .collect::<String>()
    }

    #[test]
    fn default_text_recipe_paints_content() {
        let renderer = ratatui_renderer().build();
        let node = ui::text("hello recipes").into_node();
        assert_eq!(renderer.measure(&node, 20), 1);

        let mut terminal = Terminal::new(TestBackend::new(20, 3)).expect("terminal");
        terminal
            .draw(|frame| {
                renderer
                    .pass(frame, None)
                    .render(&node, Region::new(0, 0, 20, 1));
            })
            .expect("draw");

        assert!(row_text(&terminal, 0, 20).contains("hello recipes"));
    }

    #[test]
    fn overriding_a_recipe_changes_painted_cells() {
        // Typed constructor: the closure receives `&TextNode` directly, so
        // the node kind is stated once and never restated inside the body.
        let renderer = ratatui_renderer()
            .recipes([Recipe::text(
                |_, _, _| 1,
                |ctx: &mut TuiRenderCtx<'_, '_>, _, area| {
                    ctx.surface
                        .render_widget(Paragraph::new("OVERRIDDEN"), rect(area));
                },
            )])
            .build();

        let mut terminal = Terminal::new(TestBackend::new(20, 3)).expect("terminal");
        terminal
            .draw(|frame| {
                renderer
                    .pass(frame, None)
                    .render(&ui::text("ignored").into_node(), Region::new(0, 0, 20, 1));
            })
            .expect("draw");

        assert!(row_text(&terminal, 0, 20).contains("OVERRIDDEN"));
    }
}
