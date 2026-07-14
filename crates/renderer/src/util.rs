use ratatui::style::{Color, Modifier, Style};
use unode::core::ast::{ActionIntent, StringOrExpr, Tone};

pub fn render_string_or_expr(value: &StringOrExpr) -> String {
    match value {
        StringOrExpr::Value(text) => text.clone(),
        StringOrExpr::Expr(expr) => serde_json::to_string(expr).unwrap_or_else(|_| "<expr>".to_string()),
    }
}

pub fn tone_style(tone: Option<&Tone>) -> Style {
    match tone {
        Some(Tone::Muted) => Style::default().fg(Color::Gray),
        Some(Tone::Info) => Style::default().fg(Color::Cyan),
        Some(Tone::Success) => Style::default().fg(Color::Green),
        Some(Tone::Warning) => Style::default().fg(Color::Yellow),
        Some(Tone::Danger) => Style::default().fg(Color::Red),
        _ => Style::default().fg(Color::White),
    }
}

pub fn action_style(intent: Option<&ActionIntent>, active: bool) -> Style {
    let base = match intent {
        Some(ActionIntent::Primary) => Style::default().fg(Color::Black).bg(Color::Cyan),
        Some(ActionIntent::Secondary) => Style::default().fg(Color::White).bg(Color::DarkGray),
        Some(ActionIntent::Danger) => Style::default().fg(Color::White).bg(Color::Red),
        Some(ActionIntent::Ghost) | None => Style::default().fg(Color::White),
    };

    if active {
        base.add_modifier(Modifier::BOLD | Modifier::REVERSED)
    } else {
        base.add_modifier(Modifier::BOLD)
    }
}
