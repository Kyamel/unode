use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui::{Frame, layout::Rect};
use unode::core::ast::{ItemNode, ListNode};

use crate::util::render_string_or_expr;

pub fn measure(node: &ListNode) -> u16 {
    let item_lines = node.items.iter().map(measure_item).sum::<u16>();
    item_lines.saturating_add(2).max(3)
}

pub fn render(frame: &mut Frame, area: Rect, node: &ListNode, active_item: Option<usize>) {
    let mut lines = Vec::new();
    let mut interactive_index = 0usize;

    for item in &node.items {
        let active = if item.action.is_some() {
            let is_active = active_item == Some(interactive_index);
            interactive_index += 1;
            is_active
        } else {
            false
        };
        lines.extend(render_item_lines(item, active));
    }

    let widget = Paragraph::new(Text::from(lines))
        .block(Block::default().title("List").borders(Borders::ALL))
        .wrap(Wrap { trim: false });
    frame.render_widget(widget, area);
}

fn measure_item(item: &ItemNode) -> u16 {
    let primary = item.primary.len().max(1) as u16;
    primary
        + if item.secondary.is_empty() {
            0
        } else {
            item.secondary.len() as u16
        }
}

fn render_item_lines(item: &ItemNode, active: bool) -> Vec<Line<'static>> {
    let mut lines = Vec::new();
    let mut primary = item
        .primary
        .iter()
        .map(render_node_inline)
        .filter(|value| !value.is_empty())
        .collect::<Vec<_>>();

    if primary.is_empty() {
        primary.push(item.id.clone());
    }

    let trailing = if item.action.is_some() {
        "  [open]"
    } else {
        ""
    };
    let item_style = if active {
        ratatui::style::Style::default()
            .fg(ratatui::style::Color::Black)
            .bg(ratatui::style::Color::Cyan)
            .add_modifier(ratatui::style::Modifier::BOLD)
    } else {
        ratatui::style::Style::default()
    };
    lines.push(Line::from(vec![Span::styled(
        format!("• {}{}", primary.join(" "), trailing),
        item_style,
    )]));

    for secondary in item
        .secondary
        .iter()
        .map(render_node_inline)
        .filter(|value| !value.is_empty())
    {
        lines.push(Line::from(vec![
            Span::raw("  "),
            Span::styled(
                secondary,
                ratatui::style::Style::default().fg(ratatui::style::Color::Gray),
            ),
        ]));
    }

    lines
}

pub fn render_node_inline(node: &unode::core::ast::UiNode) -> String {
    match node {
        unode::core::ast::UiNode::Text(text) => render_string_or_expr(&text.content),
        unode::core::ast::UiNode::Value(value) => {
            serde_json::to_string(&value.value).unwrap_or_default()
        }
        _ => String::new(),
    }
}
