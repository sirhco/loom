use std::path::PathBuf;

use ignore::WalkBuilder;
use regex::Regex;
use rig::completion::ToolDefinition;
use rig::tool::Tool;
use serde::{Deserialize, Serialize};

/// Default maximum number of result entries.
const DEFAULT_HEAD_LIMIT: usize = 250;

/// Maximum output size in bytes.
const MAX_OUTPUT_BYTES: usize = 200 * 1024;

/// Tool for searching file contents using regex patterns.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrepTool;

/// Arguments for the GrepTool.
#[derive(Debug, Clone, Deserialize)]
pub struct GrepArgs {
    /// The regex pattern to search for.
    pub pattern: String,
    /// File or directory to search in (default: current working directory).
    pub path: Option<String>,
    /// Glob pattern to filter files (e.g. "*.rs", "*.{ts,tsx}").
    pub glob: Option<String>,
    /// Output mode: "content", "files_with_matches", or "count" (default: "files_with_matches").
    pub output_mode: Option<String>,
    /// Maximum number of result entries (default: 250).
    pub head_limit: Option<usize>,
    /// Number of context lines to show before and after each match.
    pub context: Option<usize>,
}

/// Error type for GrepTool.
#[derive(Debug, thiserror::Error)]
pub enum GrepError {
    #[error("Invalid regex pattern: {0}")]
    InvalidPattern(#[from] regex::Error),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Search error: {0}")]
    Search(String),
}

/// A match result from a single file.
struct FileMatch {
    path: PathBuf,
    lines: Vec<MatchLine>,
}

/// A single matched line with context.
struct MatchLine {
    line_number: usize,
    content: String,
    is_match: bool,
}

impl Tool for GrepTool {
    const NAME: &'static str = "grep";

    type Error = GrepError;
    type Args = GrepArgs;
    type Output = String;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: "grep".to_string(),
            description: "Search file contents using regex patterns. Respects .gitignore. Supports multiple output modes and file filtering.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "pattern": {
                        "type": "string",
                        "description": "The regex pattern to search for"
                    },
                    "path": {
                        "type": "string",
                        "description": "File or directory to search in (default: current working directory)"
                    },
                    "glob": {
                        "type": "string",
                        "description": "Glob pattern to filter files (e.g. \"*.rs\", \"*.{ts,tsx}\")"
                    },
                    "output_mode": {
                        "type": "string",
                        "description": "Output mode: \"content\" (matching lines), \"files_with_matches\" (file paths only), \"count\" (match counts). Default: \"files_with_matches\""
                    },
                    "head_limit": {
                        "type": "number",
                        "description": "Maximum number of result entries (default: 250)"
                    },
                    "context": {
                        "type": "number",
                        "description": "Number of context lines to show before and after each match (for content mode)"
                    }
                },
                "required": ["pattern"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let regex = Regex::new(&args.pattern)?;
        let head_limit = args.head_limit.unwrap_or(DEFAULT_HEAD_LIMIT);
        let context_lines = args.context.unwrap_or(0);
        let output_mode = args.output_mode.as_deref().unwrap_or("files_with_matches");

        let search_path = match &args.path {
            Some(p) => PathBuf::from(p),
            None => std::env::current_dir()?,
        };

        // Build the file walker.
        let mut walker_builder = WalkBuilder::new(&search_path);
        walker_builder
            .hidden(false)
            .git_ignore(true)
            .git_global(true)
            .git_exclude(true);

        // Apply glob filter if specified.
        if let Some(ref glob_pattern) = args.glob {
            let mut override_builder = ignore::overrides::OverrideBuilder::new(&search_path);
            override_builder
                .add(glob_pattern)
                .map_err(|e| GrepError::Search(format!("Invalid glob filter: {e}")))?;
            let overrides = override_builder
                .build()
                .map_err(|e| GrepError::Search(format!("Failed to build glob filter: {e}")))?;
            walker_builder.overrides(overrides);
        }

        let walker = walker_builder.build();

        let mut file_matches: Vec<FileMatch> = Vec::new();
        let mut total_entries = 0usize;

        for entry in walker {
            let entry = match entry {
                Ok(e) => e,
                Err(_) => continue,
            };

            // Skip directories.
            if !entry.file_type().map_or(false, |ft| ft.is_file()) {
                continue;
            }

            let file_path = entry.path();

            // Try to read the file; skip binary/unreadable files.
            let content = match std::fs::read_to_string(file_path) {
                Ok(c) => c,
                Err(_) => continue,
            };

            let lines: Vec<&str> = content.lines().collect();
            let mut matched_line_indices: Vec<usize> = Vec::new();

            for (i, line) in lines.iter().enumerate() {
                if regex.is_match(line) {
                    matched_line_indices.push(i);
                }
            }

            if matched_line_indices.is_empty() {
                continue;
            }

            total_entries += 1;
            if total_entries > head_limit {
                break;
            }

            // Collect matched lines with context.
            let mut match_lines: Vec<MatchLine> = Vec::new();
            let mut included: std::collections::HashSet<usize> = std::collections::HashSet::new();

            for &idx in &matched_line_indices {
                let start = idx.saturating_sub(context_lines);
                let end = (idx + context_lines + 1).min(lines.len());

                for i in start..end {
                    if included.insert(i) {
                        match_lines.push(MatchLine {
                            line_number: i + 1,
                            content: lines[i].to_string(),
                            is_match: regex.is_match(lines[i]),
                        });
                    }
                }
            }

            match_lines.sort_by_key(|m| m.line_number);

            file_matches.push(FileMatch {
                path: file_path.to_path_buf(),
                lines: match_lines,
            });
        }

        // Format output based on mode.
        let mut output = String::new();

        match output_mode {
            "content" => {
                for file_match in &file_matches {
                    output.push_str(&format!("{}:\n", file_match.path.display()));
                    for ml in &file_match.lines {
                        let marker = if ml.is_match { ">" } else { " " };
                        output.push_str(&format!(
                            "{marker}{:>6}: {}\n",
                            ml.line_number, ml.content
                        ));
                    }
                    output.push('\n');

                    if output.len() > MAX_OUTPUT_BYTES {
                        output.push_str("... (output truncated)\n");
                        break;
                    }
                }
            }
            "count" => {
                for file_match in &file_matches {
                    let count = file_match.lines.iter().filter(|m| m.is_match).count();
                    output.push_str(&format!(
                        "{}:{count}\n",
                        file_match.path.display()
                    ));
                }
            }
            _ => {
                // files_with_matches (default)
                for file_match in &file_matches {
                    output.push_str(&format!("{}\n", file_match.path.display()));
                }
            }
        }

        if output.is_empty() {
            Ok("No matches found".to_string())
        } else {
            let found_count = file_matches.len();
            if total_entries > head_limit {
                output.push_str(&format!(
                    "\n(Showing {head_limit} of {total_entries}+ matching files)"
                ));
            } else {
                output.push_str(&format!("\nFound {found_count} file(s) with matches"));
            }
            Ok(output)
        }
    }
}
