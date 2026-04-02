use std::path::Path;

use crate::config::paths;

/// Displays the content of loaded memory files.
pub fn execute(cwd: &Path) -> String {
    let mut lines = Vec::new();
    lines.push("Memory files:".to_string());

    // Global memory file
    let global_memory = paths::memory_path();
    if global_memory.exists() {
        match std::fs::read_to_string(&global_memory) {
            Ok(content) => {
                lines.push(format!("\n--- {} ---", global_memory.display()));
                lines.push(content);
            }
            Err(err) => {
                lines.push(format!(
                    "  [global] {} (error: {err})",
                    global_memory.display()
                ));
            }
        }
    } else {
        lines.push(format!(
            "  [global] {} (not found)",
            global_memory.display()
        ));
    }

    // Project-local memory file
    let local_memory = cwd.join("LOOM.md");
    if local_memory.exists() {
        match std::fs::read_to_string(&local_memory) {
            Ok(content) => {
                lines.push(format!("\n--- {} ---", local_memory.display()));
                lines.push(content);
            }
            Err(err) => {
                lines.push(format!(
                    "  [project] {} (error: {err})",
                    local_memory.display()
                ));
            }
        }
    } else {
        lines.push(format!(
            "  [project] {} (not found)",
            local_memory.display()
        ));
    }

    lines.join("\n")
}
