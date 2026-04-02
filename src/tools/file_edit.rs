use std::path::Path;

use rig::completion::ToolDefinition;
use rig::tool::Tool;
use serde::{Deserialize, Serialize};

/// Tool for performing search-and-replace edits in files.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEditTool;

/// Arguments for the FileEditTool.
#[derive(Debug, Clone, Deserialize)]
pub struct FileEditArgs {
    /// The absolute path to the file to edit.
    pub file_path: String,
    /// The exact string to search for.
    pub old_string: String,
    /// The replacement string.
    pub new_string: String,
    /// If true, replace all occurrences (default false).
    pub replace_all: Option<bool>,
}

/// Error type for FileEditTool.
#[derive(Debug, thiserror::Error)]
pub enum FileEditError {
    #[error("File not found: {0}")]
    NotFound(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("String not found in file: the old_string was not found in {0}")]
    StringNotFound(String),
    #[error("Found {0} occurrences of old_string in {1}. To fix: either include more surrounding text in old_string to make it unique, OR set replace_all to true to replace all {0} occurrences.")]
    AmbiguousMatch(usize, String),
    #[error("No change: old_string and new_string are identical")]
    NoChange,
}

impl Tool for FileEditTool {
    const NAME: &'static str = "edit_file";

    type Error = FileEditError;
    type Args = FileEditArgs;
    type Output = String;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: "edit_file".to_string(),
            description: "Perform exact string replacement in a file. Searches for old_string and replaces it with new_string. By default, requires the match to be unique.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "file_path": {
                        "type": "string",
                        "description": "The absolute path to the file to edit"
                    },
                    "old_string": {
                        "type": "string",
                        "description": "The exact string to search for and replace"
                    },
                    "new_string": {
                        "type": "string",
                        "description": "The replacement string"
                    },
                    "replace_all": {
                        "type": "boolean",
                        "description": "Replace all occurrences (default false)"
                    }
                },
                "required": ["file_path", "old_string", "new_string"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        if args.old_string == args.new_string {
            return Err(FileEditError::NoChange);
        }

        let path = Path::new(&args.file_path);

        if !path.exists() {
            return Err(FileEditError::NotFound(args.file_path));
        }

        let content = tokio::fs::read_to_string(path).await?;
        let replace_all = args.replace_all.unwrap_or(false);

        // Count occurrences.
        let match_count = content.matches(&args.old_string).count();

        if match_count == 0 {
            return Err(FileEditError::StringNotFound(args.file_path));
        }

        if !replace_all && match_count > 1 {
            return Err(FileEditError::AmbiguousMatch(match_count, args.file_path));
        }

        // Perform replacement.
        let new_content = if replace_all {
            content.replace(&args.old_string, &args.new_string)
        } else {
            content.replacen(&args.old_string, &args.new_string, 1)
        };

        tokio::fs::write(path, &new_content).await?;

        let replacements = if replace_all { match_count } else { 1 };
        Ok(format!(
            "Successfully made {replacements} replacement(s) in {}",
            args.file_path
        ))
    }
}
