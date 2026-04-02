use std::io::{self, BufRead, Write};

use anyhow::Result;
use crossterm::style::Stylize;

/// Prompt the user for approval of a tool operation.
///
/// Displays a formatted, colored prompt and reads a y/n response from stdin.
/// Returns `Ok(true)` if the user approves, `Ok(false)` otherwise.
/// Defaults to deny (N) if the user presses Enter without input.
pub fn prompt_user_approval(tool_name: &str, description: &str, details: &str) -> Result<bool> {
    let mut stderr = io::stderr().lock();

    // Warning header.
    writeln!(
        stderr,
        "\n{} {}",
        "\u{26a0}".yellow(),
        format!("Permission required: {}", tool_name).yellow().bold()
    )?;

    // Description.
    writeln!(stderr, "  {}", description)?;

    // Details (if non-empty).
    if !details.is_empty() {
        writeln!(stderr, "  {}", details.dark_grey())?;
    }

    // Prompt.
    write!(stderr, "  {} ", "Allow? [y/N]:".bold())?;
    stderr.flush()?;

    // Read response from stdin.
    let response = read_line_from_stdin()?;
    let trimmed = response.trim().to_lowercase();

    Ok(trimmed == "y" || trimmed == "yes")
}

/// Prompt the user for approval with an "always allow" option.
///
/// Returns:
/// - `Ok(ApprovalResponse::Allow)` for one-time approval
/// - `Ok(ApprovalResponse::AlwaysAllow)` for permanent approval
/// - `Ok(ApprovalResponse::Deny)` for denial
pub fn prompt_user_approval_extended(
    tool_name: &str,
    description: &str,
    details: &str,
) -> Result<ApprovalResponse> {
    let mut stderr = io::stderr().lock();

    writeln!(
        stderr,
        "\n{} {}",
        "\u{26a0}".yellow(),
        format!("Permission required: {}", tool_name).yellow().bold()
    )?;

    writeln!(stderr, "  {}", description)?;

    if !details.is_empty() {
        writeln!(stderr, "  {}", details.dark_grey())?;
    }

    write!(
        stderr,
        "  {} ",
        "[y]es / [n]o / [a]lways allow:".bold()
    )?;
    stderr.flush()?;

    let response = read_line_from_stdin()?;
    let trimmed = response.trim().to_lowercase();

    match trimmed.as_str() {
        "y" | "yes" => Ok(ApprovalResponse::Allow),
        "a" | "always" => Ok(ApprovalResponse::AlwaysAllow),
        _ => Ok(ApprovalResponse::Deny),
    }
}

/// Response from an extended approval prompt.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApprovalResponse {
    Allow,
    AlwaysAllow,
    Deny,
}

/// Read a single line from stdin.
fn read_line_from_stdin() -> Result<String> {
    let stdin = io::stdin();
    let mut line = String::new();
    stdin.lock().read_line(&mut line)?;
    Ok(line)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_approval_response_variants() {
        // Basic sanity check on the enum.
        assert_ne!(ApprovalResponse::Allow, ApprovalResponse::Deny);
        assert_ne!(ApprovalResponse::AlwaysAllow, ApprovalResponse::Deny);
    }
}
