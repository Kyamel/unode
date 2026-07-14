use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph};
use ratatui::Frame;

use crate::screen::{render_tui_screen, TuiScreenView};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TuiNavItem {
    pub id: String,
    pub label: String,
    pub route: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TuiMainPanel {
    pub title: String,
    pub subtitle: Option<String>,
    pub lines: Vec<String>,
    pub footer: Option<String>,
}

#[derive(Debug, Clone)]
pub enum TuiMainContent {
    Panel(TuiMainPanel),
    Screen(TuiScreenView),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TuiCommandBar {
    pub prompt: String,
    pub input: String,
    pub active: bool,
    pub hint: Option<String>,
}

#[derive(Debug, Clone)]
pub struct TuiShellView {
    pub title: String,
    pub status: String,
    pub nav_items: Vec<TuiNavItem>,
    pub selected_nav: usize,
    pub focused_pane: TuiFocusedPane,
    pub main: TuiMainContent,
    pub command_bar: TuiCommandBar,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TuiFocusedPane {
    Navigation,
    Main,
}

pub fn render_tui_shell(frame: &mut Frame, view: &TuiShellView) {
    let root = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(3)])
        .split(frame.area());

    let body = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(30), Constraint::Min(0)])
        .split(root[0]);

    let nav_block = Block::default()
        .title(Line::from(vec![
            Span::styled(
                " Navigation ",
                if view.focused_pane == TuiFocusedPane::Navigation {
                    Style::default().fg(Color::Black).bg(Color::Cyan)
                } else {
                    Style::default().fg(Color::Black).bg(Color::DarkGray)
                },
            ),
            Span::raw(format!(" {}", view.status)),
        ]))
        .border_style(if view.focused_pane == TuiFocusedPane::Navigation {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default()
        })
        .borders(Borders::ALL);

    let nav_items = view
        .nav_items
        .iter()
        .map(|item| ListItem::new(Line::from(vec![
            Span::styled("• ", Style::default().fg(Color::DarkGray)),
            Span::raw(item.label.clone()),
        ])))
        .collect::<Vec<_>>();

    let nav_list = List::new(nav_items)
        .block(nav_block)
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("› ");

    let mut nav_state = ListState::default();
    if !view.nav_items.is_empty() {
        nav_state.select(Some(view.selected_nav.min(view.nav_items.len().saturating_sub(1))));
    }
    frame.render_stateful_widget(nav_list, body[0], &mut nav_state);

    let main_block = Block::default()
        .title(view.title.clone())
        .border_style(if view.focused_pane == TuiFocusedPane::Main {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default()
        })
        .borders(Borders::ALL);
    let main_inner = main_block.inner(body[1]);
    frame.render_widget(main_block, body[1]);

    match &view.main {
        TuiMainContent::Panel(panel) => render_panel(frame, main_inner, panel),
        TuiMainContent::Screen(screen) => render_tui_screen(frame, main_inner, screen),
    }

    let command_style = if view.command_bar.active {
        Style::default().fg(Color::Black).bg(Color::Green)
    } else {
        Style::default().fg(Color::Gray)
    };

    let command_line = Line::from(vec![
        Span::styled(
            format!(" {} ", view.command_bar.prompt),
            command_style.add_modifier(Modifier::BOLD),
        ),
        Span::raw(view.command_bar.input.clone()),
        Span::raw(" "),
        Span::styled(
            view.command_bar.hint.clone().unwrap_or_default(),
            Style::default().fg(Color::DarkGray),
        ),
    ]);

    frame.render_widget(
        Paragraph::new(command_line).block(Block::default().borders(Borders::ALL).title("Command")),
        root[1],
    );
}

fn render_panel(frame: &mut Frame, area: ratatui::layout::Rect, panel: &TuiMainPanel) {
    let mut lines = vec![Line::from(Span::styled(
        panel.title.clone(),
        Style::default().add_modifier(Modifier::BOLD),
    ))];

    if let Some(subtitle) = &panel.subtitle {
        lines.push(Line::from(Span::styled(
            subtitle.clone(),
            Style::default().fg(Color::Gray),
        )));
        lines.push(Line::default());
    } else {
        lines.push(Line::default());
    }

    for line in &panel.lines {
        lines.push(Line::from(line.clone()));
    }

    if let Some(footer) = &panel.footer {
        lines.push(Line::default());
        lines.push(Line::from(Span::styled(
            footer.clone(),
            Style::default().fg(Color::DarkGray),
        )));
    }

    frame.render_widget(Paragraph::new(Text::from(lines)), area);
}
