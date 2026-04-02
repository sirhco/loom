use std::path::Path;
use std::process::Command;

/// Returns a formatted git context string for inclusion in the system prompt.
///
/// Combines branch name, user name, short status, and recent commits.
/// Returns `None` if the directory is not inside a git repository.
pub fn get_git_context(cwd: &Path) -> Option<String> {
    // Quick check: is this a git repo?
    let branch = get_branch(cwd)?;

    let mut parts = Vec::new();
    parts.push(format!("Current branch: {branch}"));

    if let Some(user) = get_user_name(cwd) {
        parts.push(format!("Git user: {user}"));
    }

    if let Some(status) = get_status_short(cwd) {
        if !status.is_empty() {
            parts.push(format!("\nStatus:\n{status}"));
        } else {
            parts.push("\nStatus: clean (no uncommitted changes)".to_string());
        }
    }

    if let Some(log) = get_recent_commits(cwd, 5) {
        if !log.is_empty() {
            parts.push(format!("\nRecent commits:\n{log}"));
        }
    }

    Some(parts.join("\n"))
}

/// Returns the current branch name, or None if not in a git repo.
pub fn get_branch(cwd: &Path) -> Option<String> {
    run_git(cwd, &["rev-parse", "--abbrev-ref", "HEAD"])
}

/// Returns the short status output from `git status --short`.
pub fn get_status_short(cwd: &Path) -> Option<String> {
    run_git(cwd, &["status", "--short"])
}

/// Returns the recent commit log as oneline summaries.
pub fn get_recent_commits(cwd: &Path, count: usize) -> Option<String> {
    let count_arg = format!("-{count}");
    run_git(cwd, &["log", "--oneline", &count_arg])
}

/// Returns the git user name from config.
fn get_user_name(cwd: &Path) -> Option<String> {
    run_git(cwd, &["config", "user.name"])
}

/// Runs a git command in the given directory and returns trimmed stdout.
///
/// Returns `None` if the command fails or produces no output.
fn run_git(cwd: &Path, args: &[&str]) -> Option<String> {
    let output = Command::new("git")
        .args(args)
        .current_dir(cwd)
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if stdout.is_empty() {
        return None;
    }

    Some(stdout)
}
