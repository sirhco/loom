use std::path::Path;

/// Generates a git commit with an AI-generated message.
///
/// This is a placeholder that currently runs `git status` and shows the result.
/// A future version will use the AI engine to generate a commit message.
pub async fn execute(cwd: &Path) -> String {
    // Run git status to show what would be committed
    let output = match std::process::Command::new("git")
        .args(["status", "--short"])
        .current_dir(cwd)
        .output()
    {
        Ok(output) => output,
        Err(err) => {
            return format!("Failed to run git status: {err}");
        }
    };

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return format!("git status failed: {stderr}");
    }

    let status = String::from_utf8_lossy(&output.stdout);
    if status.trim().is_empty() {
        return "Nothing to commit (working tree clean).".to_string();
    }

    let mut lines = Vec::new();
    lines.push("Current git status:".to_string());
    lines.push(status.to_string());
    lines.push(
        "AI-generated commit messages coming soon. For now, use !git commit -m \"message\""
            .to_string(),
    );
    lines.join("\n")
}
