use ratatui::layout::Rect;
use ratatui::widgets::{Block, Borders};
use ratatui::Frame;
use unode::core::ast::SectionNode;

use crate::util::render_string_or_expr;

pub fn measure_header(node: &SectionNode) -> u16 {
    1 + u16::from(node.description.is_some())
}

pub fn title(node: &SectionNode) -> String {
    node.title
        .as_ref()
        .map(render_string_or_expr)
        .unwrap_or_else(|| "Section".to_string())
}

pub fn render_block(frame: &mut Frame, area: Rect, node: &SectionNode) -> Rect {
    let block = Block::default().title(title(node)).borders(Borders::ALL);
    let inner = block.inner(area);
    frame.render_widget(block, area);
    inner
}
