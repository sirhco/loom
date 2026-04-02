use ratatui::style::{Color, Modifier, Style};

/// Color theme for the TUI, using ratatui styles.
pub struct Theme {
    /// Style for the input prompt indicator.
    pub prompt: Style,
    /// Style for user messages in the history.
    pub user_message: Style,
    /// Default style for assistant response text.
    pub assistant: Style,
    /// Style for tool call display.
    pub tool_call: Style,
    /// Style for successful tool results.
    pub tool_result: Style,
    /// Style for failed tool results.
    pub tool_error: Style,
    /// Style for error messages.
    pub error: Style,
    /// Style for system informational messages.
    pub system: Style,
    /// Dim / muted style for secondary content.
    pub dim: Style,
    /// Header bar background style.
    pub header_bg: Style,
    /// Footer bar background style.
    pub footer_bg: Style,
    /// Plan mode header background style.
    pub plan_header_bg: Style,
    /// Plan mode footer background style.
    pub plan_footer_bg: Style,
    /// Spinner animation style.
    pub spinner: Style,
    /// Inline code style.
    pub code: Style,
    /// Code block style.
    pub code_block: Style,
    /// Level-1 heading style.
    pub heading1: Style,
    /// Level-2 heading style.
    pub heading2: Style,
    /// Link text style.
    pub link: Style,
    /// Input area border style.
    pub input_border: Style,
    /// Selected completion item style.
    pub completion_selected: Style,
    /// Normal (unselected) completion item style.
    pub completion_normal: Style,
    /// Left-border style for user messages (Cyan).
    pub user_border: Style,
    /// Left-border style for assistant messages (Green).
    pub assistant_border: Style,
    /// Separator characters (e.g. `│` between header items).
    pub separator: Style,
    /// Label for user messages (Cyan Bold).
    pub user_label: Style,
    /// Label for assistant messages (Green Bold).
    pub assistant_label: Style,
    /// Tool name in compact tool-call lines.
    pub tool_name: Style,
    /// The `❯` separator in tool-call lines.
    pub tool_separator: Style,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            prompt: Style::default().fg(Color::Cyan),
            user_message: Style::default().fg(Color::White),
            assistant: Style::default().fg(Color::White),
            tool_call: Style::default().fg(Color::DarkGray),
            tool_result: Style::default().fg(Color::Green),
            tool_error: Style::default().fg(Color::Red),
            error: Style::default().fg(Color::Red),
            system: Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::ITALIC),
            dim: Style::default().fg(Color::DarkGray),
            header_bg: Style::default()
                .bg(Color::DarkGray)
                .fg(Color::White),
            footer_bg: Style::default()
                .bg(Color::DarkGray)
                .fg(Color::White),
            plan_header_bg: Style::default()
                .bg(Color::Magenta)
                .fg(Color::White),
            plan_footer_bg: Style::default()
                .bg(Color::Magenta)
                .fg(Color::White),
            spinner: Style::default().fg(Color::Cyan),
            code: Style::default().fg(Color::Yellow),
            code_block: Style::default().fg(Color::Green),
            heading1: Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
            heading2: Style::default()
                .fg(Color::Blue)
                .add_modifier(Modifier::BOLD),
            link: Style::default()
                .fg(Color::Blue)
                .add_modifier(Modifier::UNDERLINED),
            input_border: Style::default().fg(Color::Cyan),
            completion_selected: Style::default()
                .add_modifier(Modifier::REVERSED),
            completion_normal: Style::default().fg(Color::White),
            user_border: Style::default().fg(Color::Cyan),
            assistant_border: Style::default().fg(Color::Green),
            separator: Style::default().fg(Color::DarkGray),
            user_label: Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
            assistant_label: Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
            tool_name: Style::default().fg(Color::White),
            tool_separator: Style::default().fg(Color::DarkGray),
        }
    }
}
