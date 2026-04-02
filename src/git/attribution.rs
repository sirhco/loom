/// Returns the co-author attribution line for git commits.
pub fn co_author_line() -> String {
    "Co-Authored-By: Loom (Gemini) <noreply@loom.dev>".to_string()
}

/// Appends the co-author attribution to a commit message.
///
/// Inserts a blank line separator before the co-author line if the message
/// does not already end with a newline.
pub fn append_co_author(message: &str) -> String {
    let trimmed = message.trim_end();
    format!("{trimmed}\n\n{}", co_author_line())
}
