use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, BorderType, Borders, Clear, List, ListItem, ListState};
use ratatui::Frame;

use crate::tui::theme::Theme;

/// What triggered the completion popup.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompletionTrigger {
    /// `/` prefix -- slash command completion.
    Slash,
    /// `@` prefix -- file reference completion.
    AtFile,
    /// Model selection from `/model` command.
    Model,
    /// Plan action selection when plan is ready.
    PlanAction,
}

/// State for the completion popup overlay.
pub struct CompletionState {
    pub trigger: CompletionTrigger,
    pub filter: String,
    pub candidates: Vec<String>,
    pub selected: usize,
}

const MAX_VISIBLE: usize = 10;

impl CompletionState {
    /// Creates a new completion state for slash commands.
    pub fn new_slash(commands: &[String], filter: &str) -> Self {
        let candidates: Vec<String> = commands
            .iter()
            .filter(|c| c.starts_with(&format!("/{filter}")))
            .cloned()
            .collect();
        Self {
            trigger: CompletionTrigger::Slash,
            filter: filter.to_string(),
            candidates,
            selected: 0,
        }
    }

    /// Creates a new completion state for `@file` references.
    pub fn new_at_file(file_index: &[String], filter: &str) -> Self {
        let candidates: Vec<String> = crate::tui::completion::fuzzy_match_files(file_index, filter);
        Self {
            trigger: CompletionTrigger::AtFile,
            filter: filter.to_string(),
            candidates,
            selected: 0,
        }
    }

    pub fn move_up(&mut self) {
        if !self.candidates.is_empty() {
            if self.selected == 0 {
                self.selected = self.candidates.len() - 1;
            } else {
                self.selected -= 1;
            }
        }
    }

    pub fn move_down(&mut self) {
        if !self.candidates.is_empty() {
            self.selected = (self.selected + 1) % self.candidates.len();
        }
    }

    pub fn selected_value(&self) -> Option<&str> {
        self.candidates.get(self.selected).map(|s| s.as_str())
    }

    /// Renders the completion popup as a rounded-border list above the anchor rect.
    pub fn render(&self, frame: &mut Frame, anchor: Rect, _theme: &Theme) {
        if self.candidates.is_empty() {
            return;
        }

        let visible_count = self.candidates.len().min(MAX_VISIBLE) as u16;
        // Position the popup above the anchor area
        let popup_height = visible_count + 2; // +2 for borders
        let popup_y = anchor.y.saturating_sub(popup_height);
        let popup_width = anchor.width.min(50);

        let popup_area = Rect {
            x: anchor.x,
            y: popup_y,
            width: popup_width,
            height: popup_height,
        };

        let title = match self.trigger {
            CompletionTrigger::Slash => " Commands ",
            CompletionTrigger::AtFile => " Files ",
            CompletionTrigger::Model => " Select Model ",
            CompletionTrigger::PlanAction => " Plan Options ",
        };

        let items: Vec<ListItem> = self
            .candidates
            .iter()
            .enumerate()
            .take(MAX_VISIBLE)
            .map(|(i, candidate)| {
                let (prefix, style) = if i == self.selected {
                    (
                        "\u{25B8} ",
                        Style::default()
                            .fg(Color::Cyan)
                            .bg(Color::Black)
                            .add_modifier(Modifier::BOLD),
                    )
                } else {
                    (
                        "  ",
                        Style::default().fg(Color::White).bg(Color::Black),
                    )
                };
                ListItem::new(Line::styled(
                    format!("{prefix}{candidate}"),
                    style,
                ))
            })
            .collect();

        let list = List::new(items).block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Cyan))
                .title(title)
                .title_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
                .style(Style::default().bg(Color::Black)),
        );

        // Clear the area first so underlying text doesn't bleed through
        frame.render_widget(Clear, popup_area);

        let mut list_state = ListState::default().with_selected(Some(self.selected));
        frame.render_stateful_widget(list, popup_area, &mut list_state);
    }
}
