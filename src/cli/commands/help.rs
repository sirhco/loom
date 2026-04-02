use crate::cli::commands::all_commands;

/// Formats and returns help text listing all available slash commands.
pub fn execute() -> String {
    let commands = all_commands();
    let mut lines = Vec::new();
    lines.push("Available commands:".to_string());
    lines.push(String::new());

    // Find the longest usage string for alignment
    let max_usage_len = commands.iter().map(|c| c.usage.len()).max().unwrap_or(0);

    for cmd in &commands {
        lines.push(format!(
            "  {:<width$}  {}",
            cmd.usage,
            cmd.description,
            width = max_usage_len
        ));
    }

    lines.push(String::new());
    lines.push("Prefix with ! to run a shell command (e.g., !ls -la)".to_string());

    lines.join("\n")
}
