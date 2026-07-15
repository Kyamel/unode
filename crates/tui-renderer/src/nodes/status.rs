use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Text};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use unode::core::ast::StatusNode;

use crate::nodes::actions;
use crate::util::render_string_or_expr;

pub fn measure(node: &StatusNode) -> u16 {
    let action_rows = if node.actions.is_empty() { 0 } else { 3 };
    3 + action_rows
}

pub fn render(frame: &mut Frame, area: Rect, node: &StatusNode, focused_action: Option<usize>) {
    let style = match node.severity {
        unode::core::ast::StatusSeverity::Info => Style::default().fg(Color::Cyan),
        unode::core::ast::StatusSeverity::Success => Style::default().fg(Color::Green),
        unode::core::ast::StatusSeverity::Warning => Style::default().fg(Color::Yellow),
        unode::core::ast::StatusSeverity::Danger => Style::default().fg(Color::Red),
    };

    let rows = if node.actions.is_empty() {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0)])
            .split(area)
    } else {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0), Constraint::Length(3)])
            .split(area)
    };

    let title = node
        .title
        .as_ref()
        .map(render_string_or_expr)
        .unwrap_or_else(|| "Status".to_string());
    let body = Text::from(vec![
        Line::from(title),
        Line::from(render_string_or_expr(&node.message)),
    ]);
    let block = Paragraph::new(body)
        .style(style)
        .block(Block::default().title("Status").borders(Borders::ALL))
        .wrap(Wrap { trim: false });
    frame.render_widget(block, rows[0]);

    if rows.len() > 1 {
        let actions_node = unode::core::ast::ActionsNode {
            base: unode::core::ast::NodeBase::default(),
            align: None,
            children: node.actions.clone(),
        };
        if actions_node.children.is_empty() {
            return;
        }

        let constraints = actions_node
            .children
            .iter()
            .map(actions::button_width)
            .map(Constraint::Length)
            .collect::<Vec<_>>();
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(constraints)
            .split(rows[1]);

        for (index, (child, chunk)) in actions_node.children.iter().zip(chunks.iter()).enumerate() {
            actions::render_action(frame, *chunk, child, focused_action == Some(index));
        }
    }
}
