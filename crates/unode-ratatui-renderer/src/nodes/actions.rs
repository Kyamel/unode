use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use unode::core::ast::{ActionNode, ActionsNode};

use crate::util::{action_style, render_string_or_expr};

pub fn measure_actions(node: &ActionsNode) -> u16 {
    if node.children.is_empty() { 0 } else { 3 }
}

pub fn render_actions(frame: &mut Frame, area: Rect, node: &ActionsNode) {
    if node.children.is_empty() || area.height == 0 || area.width == 0 {
        return;
    }

    let constraints = node
        .children
        .iter()
        .map(|child| Constraint::Length(button_width(child)))
        .collect::<Vec<_>>();

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(constraints)
        .split(area);

    for (child, chunk) in node.children.iter().zip(chunks.iter()) {
        render_action(frame, *chunk, child, false);
    }
}

pub fn render_action(frame: &mut Frame, area: Rect, node: &ActionNode, active: bool) {
    let label = render_string_or_expr(&node.label);
    let style = if is_disabled(node) {
        Style::default().fg(ratatui::style::Color::DarkGray)
    } else {
        action_style(node.intent.as_ref(), active)
    };

    let widget = Paragraph::new(Line::from(vec![
        Span::raw(" "),
        Span::styled(label, style),
        Span::raw(" "),
    ]))
    .block(Block::default().borders(Borders::ALL));

    frame.render_widget(widget, area);
}

pub fn action_label(node: &ActionNode) -> String {
    render_string_or_expr(&node.label)
}

pub fn button_width(node: &ActionNode) -> u16 {
    (action_label(node).chars().count() as u16)
        .saturating_add(4)
        .max(10)
}

fn is_disabled(node: &ActionNode) -> bool {
    matches!(
        node.disabled,
        Some(unode::core::ast::OneOrExpr::Value(true))
    )
}
