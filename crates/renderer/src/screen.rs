use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui::Frame;
use unode::core::ast::{
    ActionNode, ActionRef, BoolOrExpr, ItemNode, ListNode, NodeBase, ScreenNode, StatusNode,
    StringOrExpr, UiNode,
};
use unode::core::chrome::{read_route_tabs_meta, ScreenRouteTab, ScreenRouteTabsMeta};

use crate::nodes::{actions, list, section, stack, status, text};
use crate::util::render_string_or_expr;

#[derive(Debug, Clone)]
pub struct TuiScreenView {
    pub plugin_id: String,
    pub source: String,
    pub screen: ScreenNode,
    pub focused_interaction: Option<usize>,
}

#[derive(Debug, Clone)]
pub enum TuiInteractiveKind {
    RouteTab { to: String },
    Action { action: ActionRef },
    ListItem { action: ActionRef },
}

#[derive(Debug, Clone)]
pub struct TuiInteractiveElement {
    pub node_id: Option<String>,
    pub label: String,
    pub kind: TuiInteractiveKind,
}

#[derive(Debug, Default)]
struct ScreenRenderContext {
    focused_interaction: Option<usize>,
    next_interaction: usize,
}

impl ScreenRenderContext {
    fn new(focused_interaction: Option<usize>) -> Self {
        Self {
            focused_interaction,
            next_interaction: 0,
        }
    }

    fn consume(&mut self, enabled: bool) -> bool {
        if !enabled {
            return false;
        }

        let index = self.next_interaction;
        self.next_interaction += 1;
        self.focused_interaction == Some(index)
    }
}

pub fn collect_screen_interactions(screen: &ScreenNode) -> Vec<TuiInteractiveElement> {
    let mut interactions = Vec::new();

    if let Some(route_tabs) = read_route_tabs_meta(screen) {
        interactions.extend(route_tabs.tabs.iter().cloned().map(|tab| TuiInteractiveElement {
            node_id: Some(format!("route-tab:{}", tab.id)),
            label: tab.label,
            kind: TuiInteractiveKind::RouteTab { to: tab.to },
        }));
    }

    for child in &screen.children {
        collect_node_interactions(child, &mut interactions);
    }

    interactions
}

pub fn render_tui_screen(frame: &mut Frame, area: Rect, view: &TuiScreenView) {
    if area.width == 0 || area.height == 0 {
        return;
    }

    let route_tabs = read_route_tabs_meta(&view.screen);
    let top_constraints = if route_tabs.is_some() {
        vec![Constraint::Length(2), Constraint::Length(3), Constraint::Min(0)]
    } else {
        vec![Constraint::Length(2), Constraint::Min(0)]
    };
    let sections = Layout::default()
        .direction(Direction::Vertical)
        .constraints(top_constraints)
        .split(area);

    let title = view
        .screen
        .title
        .as_ref()
        .map(render_string_or_expr)
        .unwrap_or_else(|| "Screen".to_string());
    let subtitle = view
        .screen
        .subtitle
        .as_ref()
        .map(render_string_or_expr)
        .unwrap_or_else(|| format!("{} • {}", view.plugin_id, view.source));
    let header = Paragraph::new(vec![
        Line::from(Span::styled(
            title,
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            subtitle,
            Style::default().fg(Color::Gray),
        )),
    ]);
    frame.render_widget(header, sections[0]);

    let mut ctx = ScreenRenderContext::new(view.focused_interaction);
    let content_area = if let Some(route_tabs) = route_tabs.as_ref() {
        render_route_tabs(frame, sections[1], route_tabs, &mut ctx);
        sections[2]
    } else {
        sections[1]
    };

    render_children(frame, content_area, &view.screen.children, &mut ctx);
}

fn collect_node_interactions(node: &UiNode, interactions: &mut Vec<TuiInteractiveElement>) {
    match node {
        UiNode::Action(node) => push_action_interaction(&node.base, &node.label, &node.action, !is_disabled(node), interactions),
        UiNode::Actions(node) => {
            for child in &node.children {
                push_action_interaction(&child.base, &child.label, &child.action, !is_disabled(child), interactions);
            }
        }
        UiNode::Status(node) => {
            for action in &node.actions {
                push_action_interaction(&action.base, &action.label, &action.action, !is_disabled(action), interactions);
            }
        }
        UiNode::List(node) => collect_list_interactions(node, interactions),
        UiNode::Section(node) => {
            for child in &node.children {
                collect_node_interactions(child, interactions);
            }
        }
        UiNode::Stack(node) => {
            for child in &node.children {
                collect_node_interactions(child, interactions);
            }
        }
        UiNode::Scroll(node) => {
            for child in &node.children {
                collect_node_interactions(child, interactions);
            }
        }
        UiNode::Inline(node) => {
            for child in &node.children {
                collect_node_interactions(child, interactions);
            }
        }
        UiNode::Grid(node) => {
            for child in &node.children {
                collect_node_interactions(child, interactions);
            }
        }
        _ => {}
    }
}

fn collect_list_interactions(node: &ListNode, interactions: &mut Vec<TuiInteractiveElement>) {
    for item in &node.items {
        if let Some(action) = &item.action {
            interactions.push(TuiInteractiveElement {
                node_id: Some(item.id.clone()),
                label: list_item_label(item),
                kind: TuiInteractiveKind::ListItem {
                    action: action.clone(),
                },
            });
        }
    }
}

fn push_action_interaction(
    base: &NodeBase,
    label: &StringOrExpr,
    action: &ActionRef,
    enabled: bool,
    interactions: &mut Vec<TuiInteractiveElement>,
) {
    if !enabled {
        return;
    }

    interactions.push(TuiInteractiveElement {
        node_id: base.id.clone(),
        label: render_string_or_expr(label),
        kind: TuiInteractiveKind::Action {
            action: action.clone(),
        },
    });
}

fn render_route_tabs(
    frame: &mut Frame,
    area: Rect,
    route_tabs: &ScreenRouteTabsMeta,
    ctx: &mut ScreenRenderContext,
) {
    let tabs = route_tabs
        .tabs
        .iter()
        .map(|tab| render_route_tab_span(tab, route_tabs.active == tab.id, ctx.consume(true)))
        .collect::<Vec<_>>();
    let widget = Paragraph::new(Line::from(tabs))
        .block(Block::default().borders(Borders::ALL).title("Tabs"))
        .wrap(Wrap { trim: false });
    frame.render_widget(widget, area);
}

fn render_route_tab_span(tab: &ScreenRouteTab, active: bool, focused: bool) -> Span<'static> {
    let badge = tab
        .badge
        .as_ref()
        .map(|badge| format!(" {badge}"))
        .unwrap_or_default();
    let label = format!(" {}{} ", tab.label, badge);
    let mut style = if active {
        Style::default()
            .fg(Color::Black)
            .bg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Gray)
    };

    if focused {
        style = style
            .fg(Color::Black)
            .bg(Color::Cyan)
            .add_modifier(Modifier::BOLD | Modifier::REVERSED);
    }

    Span::styled(label, style)
}

fn render_children(frame: &mut Frame, area: Rect, children: &[UiNode], ctx: &mut ScreenRenderContext) {
    if children.is_empty() || area.height == 0 || area.width == 0 {
        return;
    }

    let constraints = children
        .iter()
        .map(|child| Constraint::Length(measure_node(child, area.width)))
        .collect::<Vec<_>>();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(area);

    for (child, chunk) in children.iter().zip(chunks.iter()) {
        render_node(frame, *chunk, child, ctx);
    }
}

fn measure_node(node: &UiNode, width: u16) -> u16 {
    match node {
        UiNode::Text(node) => text::measure(node, width),
        UiNode::Stack(node) => {
            let mut total = 0;
            for child in &node.children {
                total += measure_node(child, width);
            }
            total + stack::gap_lines(node) * node.children.len().saturating_sub(1) as u16
        }
        UiNode::Section(node) => {
            let child_total = node
                .children
                .iter()
                .map(|child| measure_node(child, width.saturating_sub(2)))
                .sum::<u16>();
            2 + section::measure_header(node) + child_total
        }
        UiNode::Action(_) => 3,
        UiNode::Actions(node) => actions::measure_actions(node),
        UiNode::Status(node) => status::measure(node),
        UiNode::List(node) => list::measure(node),
        UiNode::Scroll(node) => node
            .children
            .iter()
            .map(|child| measure_node(child, width))
            .sum::<u16>()
            .max(3),
        UiNode::Inline(node) => node.children.len().max(1) as u16,
        UiNode::Grid(node) => node.children.len().max(1) as u16,
        _ => 3,
    }
    .max(1)
}

fn render_node(frame: &mut Frame, area: Rect, node: &UiNode, ctx: &mut ScreenRenderContext) {
    match node {
        UiNode::Text(node) => text::render(frame, area, node),
        UiNode::Stack(node) => render_stack(frame, area, node, ctx),
        UiNode::Section(node) => render_section(frame, area, node, ctx),
        UiNode::Action(node) => {
            let active = !is_disabled(node) && ctx.consume(true);
            actions::render_action(frame, area, node, active);
        }
        UiNode::Actions(node) => render_actions_group(frame, area, node, ctx),
        UiNode::Status(node) => render_status(frame, area, node, ctx),
        UiNode::List(node) => render_list(frame, area, node, ctx),
        UiNode::Scroll(node) => render_children(frame, area, &node.children, ctx),
        UiNode::Inline(node) => render_inline(frame, area, &node.children, ctx),
        UiNode::Grid(node) => render_inline(frame, area, &node.children, ctx),
        other => render_fallback(frame, area, other),
    }
}

fn render_actions_group(
    frame: &mut Frame,
    area: Rect,
    node: &unode::core::ast::ActionsNode,
    ctx: &mut ScreenRenderContext,
) {
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
        let active = !is_disabled(child) && ctx.consume(true);
        actions::render_action(frame, *chunk, child, active);
    }
}

fn render_status(
    frame: &mut Frame,
    area: Rect,
    node: &StatusNode,
    ctx: &mut ScreenRenderContext,
) {
    let focus = if node.actions.is_empty() {
        None
    } else {
        let start = ctx.next_interaction;
        let mut selected = None;
        let mut enabled_index = 0usize;

        for action in &node.actions {
            if is_disabled(action) {
                continue;
            }
            if ctx.consume(true) {
                selected = Some(enabled_index);
            }
            enabled_index += 1;
        }

        let _ = start;
        selected
    };
    status::render(frame, area, node, focus)
}

fn render_list(frame: &mut Frame, area: Rect, node: &ListNode, ctx: &mut ScreenRenderContext) {
    let mut selected = None;
    let mut interactive_index = 0usize;

    for item in &node.items {
        if item.action.is_none() {
            continue;
        }
        if ctx.consume(true) {
            selected = Some(interactive_index);
        }
        interactive_index += 1;
    }

    list::render(frame, area, node, selected)
}

fn render_stack(
    frame: &mut Frame,
    area: Rect,
    node: &unode::core::ast::StackNode,
    ctx: &mut ScreenRenderContext,
) {
    if node.children.is_empty() {
        return;
    }

    let gap = stack::gap_lines(node);
    let mut constraints = Vec::new();
    for (index, child) in node.children.iter().enumerate() {
        constraints.push(Constraint::Length(measure_node(child, area.width)));
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
        render_node(frame, chunks[chunk_index], child, ctx);
        chunk_index += 1 + usize::from(gap > 0);
    }
}

fn render_section(
    frame: &mut Frame,
    area: Rect,
    node: &unode::core::ast::SectionNode,
    ctx: &mut ScreenRenderContext,
) {
    let inner = section::render_block(frame, area, node);
    let mut constraints = Vec::new();

    if let Some(description) = &node.description {
        let desc_len = render_string_or_expr(description);
        constraints.push(Constraint::Length(desc_len.lines().count().max(1) as u16));
    }

    for child in &node.children {
        constraints.push(Constraint::Length(measure_node(child, inner.width)));
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
        frame.render_widget(
            Paragraph::new(render_string_or_expr(description))
                .style(Style::default().fg(Color::Gray))
                .wrap(Wrap { trim: false }),
            chunks[0],
        );
        offset = 1;
    }

    for (child, chunk) in node.children.iter().zip(chunks.iter().skip(offset)) {
        render_node(frame, *chunk, child, ctx);
    }
}

fn render_inline(frame: &mut Frame, area: Rect, children: &[UiNode], ctx: &mut ScreenRenderContext) {
    if children.is_empty() {
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
        render_node(frame, *chunk, child, ctx);
    }
}

fn render_fallback(frame: &mut Frame, area: Rect, node: &UiNode) {
    let text = serde_json::to_string(node).unwrap_or_else(|_| "<unserializable node>".to_string());
    let widget = Paragraph::new(text)
        .block(Block::default().title("Node").borders(Borders::ALL))
        .wrap(Wrap { trim: false });
    frame.render_widget(widget, area);
}

fn is_disabled(node: &ActionNode) -> bool {
    matches!(node.disabled, Some(BoolOrExpr::Value(true)))
}

fn list_item_label(item: &ItemNode) -> String {
    let primary = item
        .primary
        .iter()
        .map(list::render_node_inline)
        .filter(|value| !value.is_empty())
        .collect::<Vec<_>>();

    if primary.is_empty() {
        item.id.clone()
    } else {
        primary.join(" ")
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;
    use unode::core::ast::{ActionType, CoreActionType, NodeBase};
    use unode::core::chrome::{create_route_tabs_meta, with_route_tabs, ScreenRouteTab};
    use unode::core::dsl::{self as ui, IntoNode};

    use super::{collect_screen_interactions, TuiInteractiveKind};

    #[test]
    fn collects_route_tabs_and_actions_in_render_order() {
        let screen = with_route_tabs(
            ui::screen()
                .id("screen")
                .children([
                    ui::action(
                        "Refresh".to_string(),
                        unode::core::ast::ActionRef {
                            r#type: ActionType::Core(CoreActionType::Refresh),
                            params: None,
                            confirm: None,
                        },
                    )
                    .id("refresh")
                    .into_node(),
                    unode::core::ast::UiNode::List(unode::core::ast::ListNode {
                        base: NodeBase::default(),
                        items: vec![unode::core::ast::ItemNode {
                            id: "row-1".to_string(),
                            meta: None,
                            leading: vec![],
                            primary: vec![ui::text("Open row".to_string()).into_node()],
                            secondary: vec![],
                            trailing: vec![],
                            action: Some(unode::core::ast::ActionRef {
                                r#type: ActionType::Custom("row.open".to_string()),
                                params: Some(std::collections::BTreeMap::from([(
                                    "id".to_string(),
                                    json!("row-1"),
                                )])),
                                confirm: None,
                            }),
                        }],
                        density: None,
                        continuation: None,
                    }),
                ])
                .build(),
            create_route_tabs_meta(
                "overview",
                vec![ScreenRouteTab {
                    id: "overview".to_string(),
                    label: "Overview".to_string(),
                    to: "/plugins/demo".to_string(),
                    badge: None,
                }],
            ),
        );

        let interactions = collect_screen_interactions(&screen);
        assert_eq!(interactions.len(), 3);
        assert!(matches!(
            interactions[0].kind,
            TuiInteractiveKind::RouteTab { .. }
        ));
        assert_eq!(interactions[1].node_id.as_deref(), Some("refresh"));
        assert_eq!(interactions[2].node_id.as_deref(), Some("row-1"));
    }
}
