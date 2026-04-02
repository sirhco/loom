use std::time::Instant;

use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::tui::theme::Theme;

const FRAMES: &[&str] = &[
    "\u{280B}", "\u{2819}", "\u{2839}", "\u{2838}", "\u{283C}", "\u{2834}", "\u{2826}",
    "\u{2827}", "\u{2807}", "\u{280F}",
];

/// Fun status words that rotate while the AI is thinking.
const THINKING_WORDS: &[&str] = &[
    "Interlacing logic...",
    "Warping reality...",
    "Knitting modules...",
    "Crossing the weft...",
    "Spinning a yarn...",
    "Aligning the bobbin...",
    "Mending breaks...",
    "Dyeing the data...",
    "Tying loose ends...",
    "Pattern matching...",
    "Looping back...",
    "Splicing fibers...",
    "Stitching context...",
    "Fabricating results...",
    "Checking the tension...",
    "Spooling thoughts...",
    "Unspooling complexity...",
    "Lacing dependencies...",
    "Braiding streams...",
];

/// How many ticks between word rotations (~2.4 seconds at 80ms ticks).
const WORD_ROTATE_TICKS: usize = 30;

/// State for an animated braille spinner with elapsed time and detail info.
pub struct SpinnerState {
    pub active: bool,
    pub message: String,
    pub detail: Option<String>,
    pub frame_idx: usize,
    tick_count: usize,
    word_idx: usize,
    use_rotating_words: bool,
    started_at: Option<Instant>,
}

impl SpinnerState {
    pub fn new() -> Self {
        Self {
            active: false,
            message: String::new(),
            detail: None,
            frame_idx: 0,
            tick_count: 0,
            word_idx: 0,
            use_rotating_words: false,
            started_at: None,
        }
    }

    pub fn start(&mut self, message: &str) {
        self.active = true;
        self.message = message.to_string();
        self.detail = None;
        self.frame_idx = 0;
        self.tick_count = 0;
        self.word_idx = 0;
        // Enable rotating words for generic "thinking" messages
        self.use_rotating_words = message == "Thinking..." || message == "Researching...";
        self.started_at = Some(Instant::now());
    }

    pub fn stop(&mut self) {
        self.active = false;
        self.message.clear();
        self.detail = None;
        self.frame_idx = 0;
        self.tick_count = 0;
        self.use_rotating_words = false;
        self.started_at = None;
    }

    pub fn update_message(&mut self, message: &str) {
        self.message = message.to_string();
        // If we go back to "thinking..." after a tool call, re-enable rotation
        if message == "thinking..." || message == "Thinking..." {
            self.use_rotating_words = true;
        } else {
            self.use_rotating_words = false;
        }
    }

    pub fn set_detail(&mut self, detail: &str) {
        self.detail = Some(detail.to_string());
    }

    pub fn tick(&mut self) {
        if self.active {
            self.frame_idx = (self.frame_idx + 1) % FRAMES.len();
            self.tick_count += 1;

            // Rotate the thinking word periodically
            if self.use_rotating_words && self.tick_count % WORD_ROTATE_TICKS == 0 {
                self.word_idx = (self.word_idx + 1) % THINKING_WORDS.len();
                self.message = THINKING_WORDS[self.word_idx].to_string();
            }
        }
    }

    fn elapsed_str(&self) -> String {
        match self.started_at {
            Some(start) => {
                let secs = start.elapsed().as_secs_f64();
                if secs < 60.0 {
                    format!("{secs:.1}s")
                } else {
                    let mins = secs as u64 / 60;
                    let remaining = secs as u64 % 60;
                    format!("{mins}m {remaining}s")
                }
            }
            None => String::new(),
        }
    }

    pub fn render(&self, frame: &mut Frame, area: Rect, _theme: &Theme) {
        if !self.active {
            let paragraph = Paragraph::new("");
            frame.render_widget(paragraph, area);
            return;
        }

        let spinner_char = FRAMES[self.frame_idx];
        let elapsed = self.elapsed_str();
        let dim = Style::default().fg(Color::DarkGray);

        let mut spans = vec![
            Span::styled(
                format!("  {spinner_char} "),
                Style::default().fg(Color::Cyan),
            ),
            Span::styled(
                self.message.clone(),
                Style::default().fg(Color::Yellow),
            ),
        ];

        // Add elapsed time in brackets
        if !elapsed.is_empty() {
            spans.push(Span::styled(format!("  [{elapsed}]"), dim));
        }

        // Add detail (e.g. "turn 2/25")
        if let Some(ref detail) = self.detail {
            spans.push(Span::styled(format!("  {detail}"), dim));
        }

        let line = Line::from(spans);
        let paragraph = Paragraph::new(line);
        frame.render_widget(paragraph, area);
    }
}
