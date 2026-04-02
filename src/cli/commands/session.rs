use crate::config::paths;

/// Session management: list or resume sessions.
pub fn execute(args: &str) -> String {
    let parts: Vec<&str> = args.split_whitespace().collect();

    match parts.first().copied() {
        None | Some("list") => list_sessions(),
        Some("resume") => {
            if let Some(id) = parts.get(1) {
                resume_session(id)
            } else {
                "Usage: /session resume <session-id>".to_string()
            }
        }
        Some(sub) => format!(
            "Unknown session subcommand: {sub}\nUsage: /session [list|resume <id>]"
        ),
    }
}

/// Lists available sessions from the sessions directory.
fn list_sessions() -> String {
    let sessions_dir = paths::sessions_dir();
    if !sessions_dir.exists() {
        return "No sessions found.".to_string();
    }

    let entries = match std::fs::read_dir(&sessions_dir) {
        Ok(entries) => entries,
        Err(err) => return format!("Failed to read sessions directory: {err}"),
    };

    let mut sessions: Vec<String> = entries
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path()
                .extension()
                .map(|ext| ext == "json")
                .unwrap_or(false)
        })
        .filter_map(|e| {
            let name = e.file_name().to_string_lossy().to_string();
            let name = name.trim_end_matches(".json").to_string();
            let modified = e.metadata().ok()?.modified().ok()?;
            Some((name, modified))
        })
        .map(|(name, _modified)| format!("  {name}"))
        .collect();

    if sessions.is_empty() {
        return "No sessions found.".to_string();
    }

    sessions.sort();
    let mut lines = vec!["Sessions:".to_string()];
    lines.extend(sessions);
    lines.push("\nUse /session resume <id> to resume a session.".to_string());
    lines.join("\n")
}

/// Placeholder for resuming a session by ID.
fn resume_session(id: &str) -> String {
    let session_file = paths::sessions_dir().join(format!("{id}.json"));
    if !session_file.exists() {
        return format!("Session not found: {id}");
    }

    format!("Session resume not yet implemented. Session file: {}", session_file.display())
}
