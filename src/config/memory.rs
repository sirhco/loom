use std::fs;
use std::path::Path;

use crate::config::paths;

/// Reads the global memory file at `~/.loom/LOOM.md`.
/// Returns `None` if the file does not exist or cannot be read.
pub fn load_global_memory() -> Option<String> {
    let path = paths::memory_path();
    fs::read_to_string(path).ok().filter(|s| !s.is_empty())
}

/// Walks up from `cwd` looking for `.loom/context.md` in any ancestor directory.
/// Returns `None` if no such file is found.
pub fn load_project_memory(cwd: &Path) -> Option<String> {
    let mut dir = cwd.to_path_buf();
    loop {
        let candidate = dir.join(".loom").join("context.md");
        if candidate.is_file() {
            if let Ok(contents) = fs::read_to_string(&candidate) {
                if !contents.is_empty() {
                    return Some(contents);
                }
            }
        }
        if !dir.pop() {
            break;
        }
    }
    None
}

/// Combines global and project memory into a single string with section headers.
/// Returns an empty string if no memory files are found.
pub fn load_all_memory(cwd: &Path) -> String {
    let mut parts: Vec<String> = Vec::new();

    if let Some(global) = load_global_memory() {
        parts.push(format!("# Global Memory (~/.loom/LOOM.md)\n\n{global}"));
    }

    if let Some(project) = load_project_memory(cwd) {
        parts.push(format!("# Project Memory (.loom/context.md)\n\n{project}"));
    }

    parts.join("\n\n---\n\n")
}
