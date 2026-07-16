//! The host's native button — the TUI counterpart of the web examples'
//! `Button.tsx` / `Button.svelte`. Where the web demos back `action` nodes
//! with a framework component via `hostSlot("Button")`, here the same is done
//! by overriding the `Action` recipe with this painter.

use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, Paragraph};
use unode::core::ast::{ActionIntent, ActionNode, BoolOrExpr};
use unode_ratatui_renderer::util::render_string_or_expr;
use unode_ratatui_renderer::{NodeKind, TuiRecipe, rect};

/// `action` nodes render as this host-styled button.
pub fn button_recipe() -> (NodeKind, TuiRecipe) {
    TuiRecipe::action(
        |_, _, _| 3,
        |ctx, node, area| {
            let focused = !is_disabled(node) && ctx.focus_next(true);
            let label = render_string_or_expr(&node.label);

            let mut style = match node.intent {
                Some(ActionIntent::Primary) => Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
                Some(ActionIntent::Danger) => Style::default().fg(Color::Red),
                _ => Style::default().fg(Color::Gray),
            };
            if is_disabled(node) {
                style = style.add_modifier(Modifier::DIM);
            }
            if focused {
                style = style
                    .fg(Color::Black)
                    .bg(Color::Cyan)
                    .add_modifier(Modifier::BOLD);
            }

            let button = Paragraph::new(format!(" {label} "))
                .style(style)
                .block(Block::default().borders(Borders::ALL).style(style));
            ctx.surface.render_widget(button, rect(area));
        },
    )
}

fn is_disabled(node: &ActionNode) -> bool {
    matches!(node.disabled, Some(BoolOrExpr::Value(true)))
}
