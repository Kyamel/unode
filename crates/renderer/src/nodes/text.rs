use ratatui::widgets::{Paragraph, Wrap};
use ratatui::{Frame, layout::Rect};
use unode::core::ast::TextNode;

use crate::util::{render_string_or_expr, tone_style};

pub fn measure(node: &TextNode, width: u16) -> u16 {
    let content = render_string_or_expr(&node.content);
    let max_width = width.max(1) as usize;
    let lines = content
        .lines()
        .map(|line| ((line.chars().count().max(1) - 1) / max_width) + 1)
        .sum::<usize>();
    lines.max(1) as u16
}

pub fn render(frame: &mut Frame, area: Rect, node: &TextNode) {
    let content = render_string_or_expr(&node.content);
    let paragraph = Paragraph::new(content)
        .style(tone_style(node.tone.as_ref()))
        .wrap(Wrap { trim: false });
    frame.render_widget(paragraph, area);
}
