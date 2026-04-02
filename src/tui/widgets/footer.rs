use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::state::app_state::PlanMode;
use crate::tui::theme::Theme;
use crate::utils::tokens::format_tokens;

/// Renders the single-line footer bar at the bottom of the TUI.
pub fn render_footer(
    frame: &mut Frame,
    area: Rect,
    _model_name: &str,
    tokens: u64,
    cost: f64,
    plan_mode: &PlanMode,
    _theme: &Theme,
) {
    let dim = Style::default().fg(Color::DarkGray);
    let sep = Style::default().fg(Color::DarkGray);

    let token_str = format_tokens(tokens as usize);

    let mut spans = vec![
        Span::styled(" tokens: ", dim),
        Span::styled(token_str, dim),
        Span::styled(" \u{2502} ", sep),
        Span::styled(format!("${cost:.2}"), dim),
        Span::styled(" \u{2502} ", sep),
    ];

    // Plan mode badge
    match plan_mode {
        PlanMode::Off => {}
        PlanMode::Researching => {
            spans.push(Span::styled(
                "[PLAN]",
                Style::default()
                    .fg(Color::Magenta)
                    .add_modifier(Modifier::BOLD),
            ));
            spans.push(Span::styled(" \u{2502} ", sep));
        }
        PlanMode::PlanReady => {
            spans.push(Span::styled(
                "[PLAN READY]",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ));
            spans.push(Span::styled(" \u{2502} ", sep));
        }
    }

    spans.push(Span::styled("Ctrl+D quit", dim));
    spans.push(Span::styled(" \u{2502} ", sep));
    spans.push(Span::styled("Shift+\u{2191}\u{2193} scroll", dim));

    let line = Line::from(spans);
    let paragraph = Paragraph::new(line).style(Style::default());
    frame.render_widget(paragraph, area);
}
