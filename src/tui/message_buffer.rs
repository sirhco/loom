use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};

use crate::tui::markdown::markdown_to_lines;

/// A message to be displayed in the scrollable conversation history.
pub enum DisplayMessage {
    /// A message entered by the user.
    User(String),
    /// A response from the AI assistant (markdown).
    Assistant(String),
    /// A tool call made by the assistant.
    ToolCall { name: String, args: String },
    /// The result of a tool call.
    ToolResult {
        name: String,
        content: String,
        is_error: bool,
    },
    /// A system-level informational message.
    System(String),
    /// An error message.
    Error(String),
}

/// Scrollable buffer of rendered conversation messages.
pub struct MessageBuffer {
    messages: Vec<DisplayMessage>,
    rendered_lines: Vec<Line<'static>>,
    rendered_width: u16,
    scroll_offset: u16,
    auto_scroll: bool,
}

impl MessageBuffer {
    /// Creates a new empty message buffer.
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
            rendered_lines: Vec::new(),
            rendered_width: 0,
            scroll_offset: 0,
            auto_scroll: true,
        }
    }

    /// Adds a message to the buffer, renders it, and appends the lines.
    /// If auto-scroll is enabled, the scroll offset is reset to bottom.
    pub fn push(&mut self, msg: DisplayMessage, width: u16) {
        self.rendered_width = width;
        let new_lines = render_message(&msg, width);
        self.rendered_lines.extend(new_lines);
        // Add a blank separator line between messages
        self.rendered_lines
            .push(Line::from(Vec::<Span<'static>>::new()));
        self.messages.push(msg);
        if self.auto_scroll {
            self.scroll_offset = 0;
        }
    }

    /// Scrolls up by the given number of lines. Disables auto-scroll.
    pub fn scroll_up(&mut self, amount: u16) {
        self.scroll_offset = self.scroll_offset.saturating_add(amount);
        self.auto_scroll = false;
    }

    /// Scrolls down by the given number of lines. Re-enables auto-scroll
    /// when reaching the bottom.
    pub fn scroll_down(&mut self, amount: u16) {
        self.scroll_offset = self.scroll_offset.saturating_sub(amount);
        if self.scroll_offset == 0 {
            self.auto_scroll = true;
        }
    }

    /// Scrolls to the bottom and re-enables auto-scroll.
    pub fn scroll_to_bottom(&mut self) {
        self.scroll_offset = 0;
        self.auto_scroll = true;
    }

    /// Returns the total number of rendered lines.
    pub fn total_lines(&self) -> u16 {
        self.rendered_lines.len() as u16
    }

    /// Returns the visible portion of rendered lines for the given viewport height.
    /// Lines are indexed from the bottom: offset 0 shows the most recent lines.
    pub fn render(&self, visible_height: u16) -> ratatui::text::Text<'static> {
        let total = self.rendered_lines.len();
        if total == 0 || visible_height == 0 {
            return ratatui::text::Text::from(Vec::<Line<'static>>::new());
        }

        let visible = visible_height as usize;
        let offset = self.scroll_offset as usize;

        // The "end" is how far from the bottom we've scrolled
        let end = if total > offset {
            total - offset
        } else {
            0
        };
        let start = end.saturating_sub(visible);

        let visible_lines: Vec<Line<'static>> =
            self.rendered_lines[start..end].to_vec();

        ratatui::text::Text::from(visible_lines)
    }

    /// Clears all messages and rendered output.
    pub fn clear(&mut self) {
        self.messages.clear();
        self.rendered_lines.clear();
        self.scroll_offset = 0;
        self.auto_scroll = true;
    }

    /// Marks the render cache as invalid (for use on terminal resize).
    pub fn invalidate_cache(&mut self) {
        self.rendered_width = 0;
    }

    /// Re-renders all messages for a new terminal width.
    pub fn re_render(&mut self, width: u16) {
        self.rendered_width = width;
        self.rendered_lines.clear();
        // Re-render each stored message
        for msg in &self.messages {
            let lines = render_message(msg, width);
            self.rendered_lines.extend(lines);
            self.rendered_lines
                .push(Line::from(Vec::<Span<'static>>::new()));
        }
    }

    /// Whether auto-scroll is currently enabled.
    pub fn is_auto_scroll(&self) -> bool {
        self.auto_scroll
    }
}

/// Renders a single `DisplayMessage` into styled `Line<'static>` items.
fn render_message(msg: &DisplayMessage, _width: u16) -> Vec<Line<'static>> {
    let border_cyan = Style::default().fg(Color::Cyan);
    let border_green = Style::default().fg(Color::Green);
    let label_cyan = Style::default()
        .fg(Color::Cyan)
        .add_modifier(Modifier::BOLD);
    let label_green = Style::default()
        .fg(Color::Green)
        .add_modifier(Modifier::BOLD);
    let dim = Style::default().fg(Color::DarkGray);
    let white = Style::default().fg(Color::White);

    match msg {
        DisplayMessage::User(text) => {
            // Left-border style: ┃ label, then ┃ content lines
            let mut lines = Vec::new();
            lines.push(Line::from(vec![
                Span::styled("  \u{2503} ", border_cyan),
                Span::styled("You", label_cyan),
            ]));
            for line_text in text.lines() {
                lines.push(Line::from(vec![
                    Span::styled("  \u{2503} ", border_cyan),
                    Span::styled(line_text.to_string(), white),
                ]));
            }
            if text.is_empty() {
                lines.push(Line::from(vec![
                    Span::styled("  \u{2503} ", border_cyan),
                ]));
            }
            lines
        }
        DisplayMessage::Assistant(text) => {
            // Left-border style: ┃ label, then ┃ prefix on each markdown line
            let mut lines = Vec::new();
            lines.push(Line::from(vec![
                Span::styled("  \u{2503} ", border_green),
                Span::styled("Assistant", label_green),
            ]));
            let md_lines = markdown_to_lines(text);
            for md_line in md_lines {
                // Prepend the green left-border to each rendered markdown line
                let mut spans = vec![Span::styled("  \u{2503} ", border_green)];
                spans.extend(md_line.spans.into_iter().map(|s| {
                    Span::styled(s.content.into_owned(), s.style)
                }));
                lines.push(Line::from(spans));
            }
            lines
        }
        DisplayMessage::ToolCall { name, args } => {
            // Compact inline: ⚙ tool_name ❯ args_preview
            let truncated_args = if args.len() > 100 {
                let end = args
                    .char_indices()
                    .nth(100)
                    .map(|(i, _)| i)
                    .unwrap_or(args.len());
                format!("{}...", &args[..end])
            } else {
                args.clone()
            };
            vec![Line::from(vec![
                Span::styled("  \u{2699} ", dim),
                Span::styled(
                    name.to_string(),
                    Style::default().fg(Color::White),
                ),
                Span::styled(" \u{276F} ", dim),
                Span::styled(truncated_args, dim),
            ])]
        }
        DisplayMessage::ToolResult {
            name,
            content,
            is_error,
        } => {
            // Compact: ✓ name: first line summary  (or ✗ for errors)
            let first_line = content.lines().next().unwrap_or("");
            let line_count = content.lines().count();
            let summary = if *is_error {
                first_line.to_string()
            } else if line_count > 1 {
                format!("{line_count} lines output")
            } else {
                first_line.to_string()
            };

            let (icon, icon_style) = if *is_error {
                ("\u{2717} ", Style::default().fg(Color::Red))
            } else {
                ("\u{2713} ", Style::default().fg(Color::Green))
            };

            vec![Line::from(vec![
                Span::styled("  ", dim),
                Span::styled(icon.to_string(), icon_style),
                Span::styled(format!("{name}: "), dim),
                Span::styled(summary, dim),
            ])]
        }
        DisplayMessage::System(text) => {
            // Dim horizontal rule with "system" label, then yellow italic text
            let rule_style = Style::default().fg(Color::DarkGray);
            let text_style = Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::ITALIC);
            let mut lines = Vec::new();
            lines.push(Line::from(vec![
                Span::styled("  \u{2500}\u{2500} ", rule_style),
                Span::styled("system", rule_style),
                Span::styled(
                    " \u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}",
                    rule_style,
                ),
            ]));
            for line_text in text.lines() {
                lines.push(Line::from(vec![
                    Span::styled("  ", rule_style),
                    Span::styled(line_text.to_string(), text_style),
                ]));
            }
            lines
        }
        DisplayMessage::Error(text) => {
            let err_style = Style::default().fg(Color::Red);
            vec![Line::from(vec![
                Span::styled("  \u{2717} ", err_style),
                Span::styled(
                    "Error: ".to_string(),
                    err_style.add_modifier(Modifier::BOLD),
                ),
                Span::styled(text.to_string(), err_style),
            ])]
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_buffer_empty() {
        let buf = MessageBuffer::new();
        assert_eq!(buf.total_lines(), 0);
        assert!(buf.is_auto_scroll());
    }

    #[test]
    fn test_push_and_scroll() {
        let mut buf = MessageBuffer::new();
        buf.push(DisplayMessage::User("hello".to_string()), 80);
        // label line + 1 content line + 1 blank separator
        assert!(buf.total_lines() >= 3);

        buf.scroll_up(1);
        assert!(!buf.is_auto_scroll());

        buf.scroll_to_bottom();
        assert!(buf.is_auto_scroll());
    }

    #[test]
    fn test_render_visible() {
        let mut buf = MessageBuffer::new();
        buf.push(DisplayMessage::System("Line 1".to_string()), 80);
        buf.push(DisplayMessage::System("Line 2".to_string()), 80);
        buf.push(DisplayMessage::System("Line 3".to_string()), 80);

        let text = buf.render(100);
        assert!(!text.lines.is_empty());
    }

    #[test]
    fn test_clear() {
        let mut buf = MessageBuffer::new();
        buf.push(DisplayMessage::User("test".to_string()), 80);
        buf.clear();
        assert_eq!(buf.total_lines(), 0);
    }
}
