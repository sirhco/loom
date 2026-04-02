use ratatui::style::{Color, Style};
use ratatui::widgets::Block;
use tui_textarea::TextArea;

/// Creates a pre-configured `TextArea` for the TUI input field.
pub fn create_textarea() -> TextArea<'static> {
    let mut textarea = TextArea::default();
    textarea.set_block(
        Block::bordered()
            .border_type(ratatui::widgets::BorderType::Rounded)
            .title(" Input ")
            .title_style(Style::default().fg(Color::Cyan).add_modifier(ratatui::style::Modifier::BOLD))
            .border_style(Style::default().fg(Color::DarkGray)),
    );
    textarea.set_cursor_line_style(Style::default());
    textarea.set_placeholder_text("Type a message... (Enter to send, Shift+Enter for newline)");
    textarea
}
