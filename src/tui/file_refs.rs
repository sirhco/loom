use std::path::Path;

use anyhow::Result;
use regex::Regex;

/// A file referenced via `@path` in user input.
#[derive(Debug, Clone)]
pub struct FileReference {
    pub path: String,
    pub content: String,
}

/// Parses `@path` references from user input, reads the referenced files,
/// and returns expanded input with file contents prepended.
///
/// File content is wrapped in `<file path="...">` tags so the model can
/// distinguish between user text and file context.
pub fn expand_file_references(input: &str, cwd: &Path) -> Result<(String, Vec<FileReference>)> {
    // Match @path references — only paths under cwd (no ../ prefix)
    let re = Regex::new(r"@([\w][\w./\-]*)").expect("valid regex");
    let mut refs = Vec::new();
    let mut seen = std::collections::HashSet::new();

    for cap in re.captures_iter(input) {
        let rel_path = cap[1].to_string();
        if seen.contains(&rel_path) {
            continue;
        }
        let abs_path = cwd.join(&rel_path);

        if abs_path.is_file() {
            // Single file reference
            match std::fs::read_to_string(&abs_path) {
                Ok(content) => {
                    refs.push(FileReference {
                        path: rel_path.clone(),
                        content,
                    });
                    seen.insert(rel_path);
                }
                Err(e) => {
                    tracing::warn!("Failed to read @{}: {e}", rel_path);
                }
            }
        } else if abs_path.is_dir() {
            // Directory reference — list files and include as context
            let mut listing = Vec::new();
            if let Ok(entries) = std::fs::read_dir(&abs_path) {
                for entry in entries.filter_map(|e| e.ok()) {
                    let name = entry.file_name().to_string_lossy().to_string();
                    let kind = if entry.path().is_dir() { "dir" } else { "file" };
                    listing.push(format!("  [{kind}] {name}"));
                }
            }
            listing.sort();
            if !listing.is_empty() {
                let content = format!(
                    "Directory listing of {}:\n{}",
                    rel_path,
                    listing.join("\n")
                );
                refs.push(FileReference {
                    path: rel_path.clone(),
                    content,
                });
                seen.insert(rel_path);
            }
        }
    }

    if refs.is_empty() {
        return Ok((input.to_string(), refs));
    }

    // Build expanded message with file contents prepended
    let mut expanded = String::new();
    for file_ref in &refs {
        expanded.push_str(&format!(
            "<file path=\"{}\">\n{}\n</file>\n\n",
            file_ref.path, file_ref.content
        ));
    }

    // Remove @path tokens from user text to avoid confusion
    let cleaned = re.replace_all(input, "").to_string();
    let cleaned = cleaned.split_whitespace().collect::<Vec<_>>().join(" ");
    if !cleaned.is_empty() {
        expanded.push_str(&cleaned);
    }

    Ok((expanded, refs))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_no_refs() {
        let tmp = TempDir::new().unwrap();
        let (expanded, refs) = expand_file_references("hello world", tmp.path()).unwrap();
        assert_eq!(expanded, "hello world");
        assert!(refs.is_empty());
    }

    #[test]
    fn test_single_ref() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("test.txt"), "file content").unwrap();
        let (expanded, refs) = expand_file_references("explain @test.txt", tmp.path()).unwrap();
        assert_eq!(refs.len(), 1);
        assert_eq!(refs[0].path, "test.txt");
        assert!(expanded.contains("<file path=\"test.txt\">"));
        assert!(expanded.contains("file content"));
        assert!(expanded.contains("explain"));
    }

    #[test]
    fn test_missing_ref_ignored() {
        let tmp = TempDir::new().unwrap();
        let (expanded, refs) =
            expand_file_references("look at @nonexistent.rs", tmp.path()).unwrap();
        assert!(refs.is_empty());
        assert_eq!(expanded, "look at @nonexistent.rs");
    }

    #[test]
    fn test_deduplication() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("a.txt"), "aaa").unwrap();
        let (_, refs) = expand_file_references("@a.txt and @a.txt", tmp.path()).unwrap();
        assert_eq!(refs.len(), 1);
    }
}
