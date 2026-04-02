use std::path::PathBuf;

use rig::completion::ToolDefinition;
use rig::tool::Tool;
use serde::{Deserialize, Serialize};

/// Maximum number of results to return.
const MAX_RESULTS: usize = 1000;

/// Tool for file pattern matching using glob patterns.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobTool;

/// Arguments for the GlobTool.
#[derive(Debug, Clone, Deserialize)]
pub struct GlobArgs {
    /// The glob pattern to match files against (e.g. "**/*.rs").
    pub pattern: String,
    /// The directory to search in (default: current working directory).
    pub path: Option<String>,
}

/// Error type for GlobTool.
#[derive(Debug, thiserror::Error)]
pub enum GlobError {
    #[error("Invalid glob pattern: {0}")]
    InvalidPattern(#[from] glob::PatternError),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Glob error: {0}")]
    GlobError(String),
}

impl Tool for GlobTool {
    const NAME: &'static str = "glob";

    type Error = GlobError;
    type Args = GlobArgs;
    type Output = String;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: "glob".to_string(),
            description: "Find files matching a glob pattern. Supports patterns like \"**/*.rs\" or \"src/**/*.ts\". Results are sorted by modification time (newest first).".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "pattern": {
                        "type": "string",
                        "description": "The glob pattern to match files against (e.g. \"**/*.rs\", \"src/**/*.ts\")"
                    },
                    "path": {
                        "type": "string",
                        "description": "The directory to search in (default: current working directory)"
                    }
                },
                "required": ["pattern"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let base_dir = match &args.path {
            Some(p) => PathBuf::from(p),
            None => std::env::current_dir()?,
        };

        // Build the full pattern by joining base_dir with the pattern.
        let full_pattern = if args.pattern.starts_with('/') {
            args.pattern.clone()
        } else {
            format!("{}/{}", base_dir.display(), args.pattern)
        };

        let glob_results = glob::glob(&full_pattern)?;

        // Collect results with modification times for sorting.
        let mut entries: Vec<(PathBuf, std::time::SystemTime)> = Vec::new();

        for entry in glob_results {
            match entry {
                Ok(path) => {
                    if path.is_file() {
                        let mtime = path
                            .metadata()
                            .and_then(|m| m.modified())
                            .unwrap_or(std::time::SystemTime::UNIX_EPOCH);
                        entries.push((path, mtime));
                    }
                }
                Err(e) => {
                    tracing::debug!("Glob entry error: {e}");
                }
            }

            if entries.len() >= MAX_RESULTS {
                break;
            }
        }

        // Sort by modification time, newest first.
        entries.sort_by(|a, b| b.1.cmp(&a.1));

        if entries.is_empty() {
            return Ok("No files found".to_string());
        }

        let total = entries.len();
        let output: String = entries
            .iter()
            .map(|(path, _)| path.display().to_string())
            .collect::<Vec<_>>()
            .join("\n");

        if total >= MAX_RESULTS {
            Ok(format!(
                "{output}\n\n(Results limited to {MAX_RESULTS} files)"
            ))
        } else {
            Ok(format!("{output}\n\n({total} files found)"))
        }
    }
}
