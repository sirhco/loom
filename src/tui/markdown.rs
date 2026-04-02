use pulldown_cmark::{Event, HeadingLevel, Parser, Tag, TagEnd};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};

/// Converts a markdown string into a vector of styled `Line<'static>` items
/// suitable for rendering in a ratatui widget.
///
/// Falls back to plain text rendering if markdown parsing fails.
pub fn markdown_to_lines(text: &str) -> Vec<Line<'static>> {
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let parser = Parser::new(text);
        let mut renderer = MarkdownRenderer::new();
        renderer.render(parser);
        renderer.finish()
    })) {
        Ok(lines) => lines,
        Err(_) => {
            // Fallback: render as plain text
            text.lines()
                .map(|line| Line::from(Span::raw(line.to_string())))
                .collect()
        }
    }
}

struct MarkdownRenderer {
    lines: Vec<Line<'static>>,
    current_spans: Vec<Span<'static>>,
    style_stack: Vec<Style>,
    in_code_block: bool,
    in_blockquote: bool,
    in_heading: Option<HeadingLevel>,
    list_stack: Vec<ListContext>,
    link_url: Option<String>,
}

#[derive(Clone)]
struct ListContext {
    ordered: bool,
    item_index: u64,
}

impl MarkdownRenderer {
    fn new() -> Self {
        Self {
            lines: Vec::new(),
            current_spans: Vec::new(),
            style_stack: vec![Style::default()],
            in_code_block: false,
            in_blockquote: false,
            in_heading: None,
            list_stack: Vec::new(),
            link_url: None,
        }
    }

    fn current_style(&self) -> Style {
        self.style_stack
            .last()
            .copied()
            .unwrap_or_default()
    }

    fn push_style(&mut self, modifier: Style) {
        let base = self.current_style();
        self.style_stack.push(base.patch(modifier));
    }

    fn pop_style(&mut self) {
        if self.style_stack.len() > 1 {
            self.style_stack.pop();
        }
    }

    fn flush_line(&mut self) {
        let spans = std::mem::take(&mut self.current_spans);
        if !spans.is_empty() {
            self.lines.push(Line::from(spans));
        } else {
            self.lines.push(Line::from(Vec::<Span<'static>>::new()));
        }
    }

    fn add_text(&mut self, text: &str) {
        if self.in_code_block {
            // Each line in a code block gets its own Line with prefix
            let code_style = Style::default().fg(Color::Green);
            let prefix_style = Style::default().fg(Color::DarkGray);
            for (i, line) in text.split('\n').enumerate() {
                if i > 0 {
                    self.flush_line();
                }
                if i > 0 || self.current_spans.is_empty() {
                    self.current_spans.push(Span::styled(
                        "  \u{2502} ".to_string(),
                        prefix_style,
                    ));
                }
                self.current_spans
                    .push(Span::styled(line.to_string(), code_style));
            }
            return;
        }

        if self.in_blockquote {
            let prefix_style = Style::default().fg(Color::DarkGray);
            let content_style = Style::default().add_modifier(Modifier::ITALIC);
            for (i, line) in text.split('\n').enumerate() {
                if i > 0 {
                    self.flush_line();
                }
                if i > 0 || self.current_spans.is_empty() {
                    self.current_spans.push(Span::styled(
                        "  > ".to_string(),
                        prefix_style,
                    ));
                }
                self.current_spans
                    .push(Span::styled(line.to_string(), content_style));
            }
            return;
        }

        let style = self.current_style();
        // Handle newlines within text
        for (i, line) in text.split('\n').enumerate() {
            if i > 0 {
                self.flush_line();
            }
            if !line.is_empty() {
                self.current_spans
                    .push(Span::styled(line.to_string(), style));
            }
        }
    }

    fn render<'a>(&mut self, parser: Parser<'a>) {
        for event in parser {
            match event {
                Event::Start(tag) => self.start_tag(tag),
                Event::End(tag_end) => self.end_tag(tag_end),
                Event::Text(text) => {
                    self.add_text(&text);
                }
                Event::Code(code) => {
                    let style = Style::default().fg(Color::Yellow);
                    self.current_spans
                        .push(Span::styled(code.to_string(), style));
                }
                Event::SoftBreak => {
                    self.current_spans
                        .push(Span::styled(" ".to_string(), self.current_style()));
                }
                Event::HardBreak => {
                    self.flush_line();
                }
                Event::Rule => {
                    self.flush_line();
                    let rule = "\u{2500}".repeat(40);
                    self.lines.push(Line::from(vec![Span::styled(
                        rule,
                        Style::default().fg(Color::DarkGray),
                    )]));
                    self.lines.push(Line::from(Vec::<Span<'static>>::new()));
                }
                Event::Html(html) | Event::InlineHtml(html) => {
                    self.current_spans.push(Span::styled(
                        html.to_string(),
                        Style::default().fg(Color::DarkGray),
                    ));
                }
                Event::FootnoteReference(label) => {
                    self.current_spans.push(Span::styled(
                        format!("[{label}]"),
                        Style::default().fg(Color::Blue),
                    ));
                }
                Event::TaskListMarker(checked) => {
                    let marker = if checked { "[x] " } else { "[ ] " };
                    self.current_spans
                        .push(Span::styled(marker.to_string(), self.current_style()));
                }
                _ => {}
            }
        }
    }

    fn start_tag(&mut self, tag: Tag<'_>) {
        match tag {
            Tag::Paragraph => {}
            Tag::Heading { level, .. } => {
                self.in_heading = Some(level);
                let style = match level {
                    HeadingLevel::H1 => Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                    _ => Style::default()
                        .fg(Color::Blue)
                        .add_modifier(Modifier::BOLD),
                };
                self.push_style(style);
            }
            Tag::BlockQuote(_) => {
                self.in_blockquote = true;
            }
            Tag::CodeBlock(_) => {
                self.in_code_block = true;
                self.flush_line();
            }
            Tag::List(first_index) => {
                self.list_stack.push(ListContext {
                    ordered: first_index.is_some(),
                    item_index: first_index.unwrap_or(1),
                });
            }
            Tag::Item => {
                self.flush_line();
                if let Some(ctx) = self.list_stack.last() {
                    if ctx.ordered {
                        let prefix = format!("  {}. ", ctx.item_index);
                        self.current_spans.push(Span::styled(
                            prefix,
                            Style::default().fg(Color::DarkGray),
                        ));
                    } else {
                        self.current_spans.push(Span::styled(
                            "  \u{2022} ".to_string(),
                            Style::default().fg(Color::DarkGray),
                        ));
                    }
                }
            }
            Tag::Emphasis => {
                self.push_style(Style::default().add_modifier(Modifier::ITALIC));
            }
            Tag::Strong => {
                self.push_style(Style::default().add_modifier(Modifier::BOLD));
            }
            Tag::Strikethrough => {
                self.push_style(Style::default().add_modifier(Modifier::CROSSED_OUT));
            }
            Tag::Link { dest_url, .. } => {
                self.link_url = Some(dest_url.to_string());
                self.push_style(
                    Style::default()
                        .fg(Color::Blue)
                        .add_modifier(Modifier::UNDERLINED),
                );
            }
            Tag::Image { .. } => {
                self.push_style(Style::default().fg(Color::Magenta));
            }
            Tag::Table(_) | Tag::TableHead | Tag::TableRow | Tag::TableCell => {}
            Tag::FootnoteDefinition(_) => {}
            Tag::HtmlBlock => {}
            Tag::DefinitionList
            | Tag::DefinitionListTitle
            | Tag::DefinitionListDefinition => {}
            Tag::MetadataBlock(_) => {}
        }
    }

    fn end_tag(&mut self, tag_end: TagEnd) {
        match tag_end {
            TagEnd::Paragraph => {
                self.flush_line();
                // Blank line after paragraphs
                self.lines.push(Line::from(Vec::<Span<'static>>::new()));
            }
            TagEnd::Heading(_level) => {
                self.pop_style();
                self.flush_line();
                // Blank line after headings
                self.lines.push(Line::from(Vec::<Span<'static>>::new()));
                self.in_heading = None;
            }
            TagEnd::BlockQuote(_) => {
                self.in_blockquote = false;
                self.flush_line();
                self.lines.push(Line::from(Vec::<Span<'static>>::new()));
            }
            TagEnd::CodeBlock => {
                self.in_code_block = false;
                self.flush_line();
                self.lines.push(Line::from(Vec::<Span<'static>>::new()));
            }
            TagEnd::List(_) => {
                if let Some(ctx) = self.list_stack.pop() {
                    let _ = ctx;
                }
                self.flush_line();
            }
            TagEnd::Item => {
                // Increment ordered list counter
                if let Some(ctx) = self.list_stack.last_mut() {
                    if ctx.ordered {
                        ctx.item_index += 1;
                    }
                }
            }
            TagEnd::Emphasis | TagEnd::Strong | TagEnd::Strikethrough => {
                self.pop_style();
            }
            TagEnd::Link => {
                self.pop_style();
                // Show URL inline if we have it
                if let Some(url) = self.link_url.take() {
                    self.current_spans.push(Span::styled(
                        format!(" ({url})"),
                        Style::default().fg(Color::DarkGray),
                    ));
                }
            }
            TagEnd::Image => {
                self.pop_style();
            }
            TagEnd::Table
            | TagEnd::TableHead
            | TagEnd::TableRow
            | TagEnd::TableCell => {}
            TagEnd::FootnoteDefinition => {}
            TagEnd::HtmlBlock => {
                self.flush_line();
            }
            TagEnd::DefinitionList
            | TagEnd::DefinitionListTitle
            | TagEnd::DefinitionListDefinition => {}
            TagEnd::MetadataBlock(_) => {}
        }
    }

    fn finish(mut self) -> Vec<Line<'static>> {
        // Flush any remaining spans
        if !self.current_spans.is_empty() {
            self.flush_line();
        }
        // Remove trailing empty lines
        while self
            .lines
            .last()
            .is_some_and(|l| l.spans.is_empty())
        {
            self.lines.pop();
        }
        self.lines
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plain_text() {
        let lines = markdown_to_lines("Hello, world!");
        assert!(!lines.is_empty());
    }

    #[test]
    fn test_heading() {
        let lines = markdown_to_lines("# Title\n\nSome text");
        assert!(lines.len() >= 2);
    }

    #[test]
    fn test_code_block() {
        let lines = markdown_to_lines("```\nfn main() {}\n```");
        assert!(!lines.is_empty());
    }

    #[test]
    fn test_bold_italic() {
        let lines = markdown_to_lines("**bold** and *italic*");
        assert!(!lines.is_empty());
    }

    #[test]
    fn test_list() {
        let lines = markdown_to_lines("- item 1\n- item 2\n- item 3");
        assert!(lines.len() >= 3);
    }

    #[test]
    fn test_empty_input() {
        let lines = markdown_to_lines("");
        assert!(lines.is_empty());
    }
}
