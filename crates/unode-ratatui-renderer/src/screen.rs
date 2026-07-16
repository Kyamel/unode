use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use unode::core::ast::{
    ActionNode, ActionRef, BoolOrExpr, ItemNode, ListNode, NodeBase, ScreenNode, StringOrExpr,
    UiNode,
};
use unode::core::chrome::{RouteTabView, RouteTabsView};

use crate::nodes::list;
use crate::recipes::{TuiRenderCtx, TuiRenderer, render_vertical_children};
use crate::util::render_string_or_expr;

#[derive(Debug, Clone)]
pub struct TuiScreenView {
    pub plugin_id: String,
    pub source: String,
    pub screen: ScreenNode,
    /// Route tabs derived by the host from the plugin manifest's route
    /// groups (`unode::core::chrome::route_tabs_view`). `None` renders the
    /// screen standalone.
    pub route_tabs: Option<RouteTabsView>,
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

pub fn collect_screen_interactions(
    screen: &ScreenNode,
    route_tabs: Option<&RouteTabsView>,
) -> Vec<TuiInteractiveElement> {
    let mut interactions = Vec::new();

    if let Some(route_tabs) = route_tabs {
        interactions.extend(
            route_tabs
                .tabs
                .iter()
                .cloned()
                .map(|tab| TuiInteractiveElement {
                    node_id: Some(format!("route-tab:{}", tab.to)),
                    label: tab.label,
                    kind: TuiInteractiveKind::RouteTab { to: tab.to },
                }),
        );
    }

    for child in &screen.children {
        collect_node_interactions(child, &mut interactions);
    }

    interactions
}

/// Renders a plugin screen: host chrome (header, route tabs) plus the node
/// tree through the host-declared recipe set.
pub fn render_tui_screen(
    frame: &mut Frame,
    area: Rect,
    view: &TuiScreenView,
    renderer: &TuiRenderer,
) {
    if area.width == 0 || area.height == 0 {
        return;
    }

    let route_tabs = view.route_tabs.as_ref();
    let top_constraints = if route_tabs.is_some() {
        vec![
            Constraint::Length(2),
            Constraint::Length(3),
            Constraint::Min(0),
        ]
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
        Line::from(Span::styled(subtitle, Style::default().fg(Color::Gray))),
    ]);
    frame.render_widget(header, sections[0]);

    let mut ctx = renderer.pass(frame, view.focused_interaction);
    let content_area = if let Some(route_tabs) = route_tabs {
        render_route_tabs(&mut ctx, sections[1], route_tabs);
        sections[2]
    } else {
        sections[1]
    };

    render_vertical_children(&mut ctx, &view.screen.children, content_area);
}

fn collect_node_interactions(node: &UiNode, interactions: &mut Vec<TuiInteractiveElement>) {
    match node {
        UiNode::Action(node) => push_action_interaction(
            &node.base,
            &node.label,
            &node.action,
            !is_disabled(node),
            interactions,
        ),
        UiNode::Actions(node) => {
            for child in &node.children {
                push_action_interaction(
                    &child.base,
                    &child.label,
                    &child.action,
                    !is_disabled(child),
                    interactions,
                );
            }
        }
        UiNode::Status(node) => {
            for action in &node.actions {
                push_action_interaction(
                    &action.base,
                    &action.label,
                    &action.action,
                    !is_disabled(action),
                    interactions,
                );
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

fn render_route_tabs(ctx: &mut TuiRenderCtx<'_, '_>, area: Rect, route_tabs: &RouteTabsView) {
    let tabs = route_tabs
        .tabs
        .iter()
        .map(|tab| render_route_tab_span(tab, route_tabs.active == tab.to, ctx.focus_next(true)))
        .collect::<Vec<_>>();
    let widget = Paragraph::new(Line::from(tabs))
        .block(Block::default().borders(Borders::ALL).title("Tabs"))
        .wrap(Wrap { trim: false });
    ctx.surface.render_widget(widget, area);
}

fn render_route_tab_span(tab: &RouteTabView, active: bool, focused: bool) -> Span<'static> {
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
    use unode::core::chrome::{RouteTabView, RouteTabsView};
    use unode::core::dsl::{self as ui, IntoNode};

    use super::{TuiInteractiveKind, collect_screen_interactions};

    #[test]
    fn collects_route_tabs_and_actions_in_render_order() {
        let screen = ui::screen()
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
            .build();

        let route_tabs = RouteTabsView {
            group: "main".to_string(),
            active: "/plugins/demo".to_string(),
            tabs: vec![RouteTabView {
                to: "/plugins/demo".to_string(),
                label: "Overview".to_string(),
                badge: None,
            }],
        };

        let interactions = collect_screen_interactions(&screen, Some(&route_tabs));
        assert_eq!(interactions.len(), 3);
        assert!(matches!(
            interactions[0].kind,
            TuiInteractiveKind::RouteTab { .. }
        ));
        assert_eq!(interactions[1].node_id.as_deref(), Some("refresh"));
        assert_eq!(interactions[2].node_id.as_deref(), Some("row-1"));

        let without_tabs = collect_screen_interactions(&screen, None);
        assert_eq!(without_tabs.len(), 2);
    }
}
