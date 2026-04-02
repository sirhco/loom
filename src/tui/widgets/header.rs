use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::state::app_state::PlanMode;
use crate::tui::theme::Theme;

/// Renders the single-line header bar at the top of the TUI.
pub fn render_header(
    frame: &mut Frame,
    area: Rect,
    model_name: &str,
    plan_mode: &PlanMode,
    _theme: &Theme,
) {
    let sep = Style::default().fg(Color::DarkGray);
    let dim = Style::default().fg(Color::DarkGray);

    let (icon, icon_style, bg) = match plan_mode {
        PlanMode::Off => (
            "\u{25C6}",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
            Color::Reset,
        ),
        PlanMode::Researching => (
            "\u{25C7}",
            Style::default()
                .fg(Color::Magenta)
                .add_modifier(Modifier::BOLD),
            Color::Reset,
        ),
        PlanMode::PlanReady => (
            "\u{25C7}",
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
            Color::Reset,
        ),
    };

    let line = Line::from(vec![
        Span::styled(format!(" {icon} "), icon_style),
        Span::styled(
            "Loom",
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" \u{2502} ", sep),
        Span::styled(
            model_name.to_string(),
            Style::default().fg(Color::Yellow),
        ),
        Span::styled(" \u{2502} ", sep),
        Span::styled("/help for commands", dim),
    ]);

    let base = Style::default().bg(bg).fg(Color::White);
    let paragraph = Paragraph::new(line).style(base);
    frame.render_widget(paragraph, area);
}
