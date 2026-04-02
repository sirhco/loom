use crossterm::style::{self, Stylize};

use crate::tui::theme::Theme;

/// Renders a single-line status bar with model, token, and cost info.
pub fn render_status_bar(model_id: &str, input_tokens: u64, output_tokens: u64, cost: f64) {
    let theme = Theme::default();
    let total_tokens = input_tokens + output_tokens;

    let bar = format!(
        " [{}] | tokens: {} | cost: ${:.4} | /help for commands ",
        model_id,
        total_tokens,
        cost,
    );

    let styled = style::style(&bar)
        .on(theme.status_bg)
        .with(theme.status_fg);
    eprintln!("{styled}");
}
