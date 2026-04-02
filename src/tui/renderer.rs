use ratatui::layout::{Constraint, Layout};
use ratatui::widgets::{Paragraph, Wrap};
use ratatui::Frame;

use crate::tui::app::TuiApp;
use crate::tui::theme::Theme;
use crate::tui::widgets;

// --- Legacy compatibility functions used by cli::commands::plan ---
// These print directly to stderr for commands that haven't been migrated
// to return their output through CommandResult yet.

/// Renders a system message to stderr (legacy compatibility).
pub fn render_system_message(text: &str) {
    eprintln!("{text}");
}

/// Renders an assistant message to stderr (legacy compatibility).
pub fn render_assistant_message(text: &str) {
    eprintln!("{text}");
}

/// Renders an error message to stderr (legacy compatibility).
pub fn render_error(message: &str) {
    eprintln!("Error: {message}");
}

/// Renders a plan to stderr (legacy compatibility).
pub fn render_plan(plan_text: &str) {
    eprintln!("{plan_text}");
}

/// Renders a welcome banner (legacy compatibility, no-op in TUI mode).
pub fn render_welcome() {
    // No-op in TUI mode; the welcome message is shown via MessageBuffer.
}

/// Main draw function: lays out and renders all TUI components.
pub fn draw(frame: &mut Frame, app: &TuiApp) {
    let theme = Theme::default();

    let chunks = Layout::vertical([
        Constraint::Length(1),  // Header
        Constraint::Min(5),    // Messages
        Constraint::Length(1), // Spinner
        Constraint::Length(4), // Input textarea
        Constraint::Length(1), // Footer
    ])
    .split(frame.area());

    // Header bar
    widgets::header::render_header(
        frame,
        chunks[0],
        &app.model_name,
        &app.plan_mode,
        &theme,
    );

    // Messages area
    let messages_text = app.messages.render(chunks[1].height);
    let messages_widget = Paragraph::new(messages_text).wrap(Wrap { trim: false });
    frame.render_widget(messages_widget, chunks[1]);

    // Spinner line
    app.spinner.render(frame, chunks[2], &theme);

    // Input textarea
    frame.render_widget(&app.textarea, chunks[3]);

    // Footer bar
    widgets::footer::render_footer(
        frame,
        chunks[4],
        &app.model_name,
        app.total_tokens,
        app.total_cost,
        &app.plan_mode,
        &theme,
    );

    // Completion popup overlay (rendered last so it's on top)
    if let Some(ref completion) = app.completion {
        completion.render(frame, chunks[3], &theme);
    }
}
