use std::path::Path;

use rig::completion::ToolDefinition;
use rig::tool::Tool;
use serde::{Deserialize, Serialize};

/// Tool for writing content to files.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileWriteTool;

/// Arguments for the FileWriteTool.
#[derive(Debug, Clone, Deserialize)]
pub struct FileWriteArgs {
    /// The absolute path to the file to write.
    pub file_path: String,
    /// The content to write to the file.
    pub content: String,
}

/// Error type for FileWriteTool.
#[derive(Debug, thiserror::Error)]
pub enum FileWriteError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Write failed: {0}")]
    WriteFailed(String),
}

impl Tool for FileWriteTool {
    const NAME: &'static str = "write_file";

    type Error = FileWriteError;
    type Args = FileWriteArgs;
    type Output = String;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: "write_file".to_string(),
            description: "Write content to a file, creating it and any parent directories if they don't exist. Overwrites the file if it already exists.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "file_path": {
                        "type": "string",
                        "description": "The absolute path to the file to write"
                    },
                    "content": {
                        "type": "string",
                        "description": "The content to write to the file"
                    }
                },
                "required": ["file_path", "content"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let path = Path::new(&args.file_path);

        // Create parent directories if they don't exist.
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                tokio::fs::create_dir_all(parent).await?;
            }
        }

        tokio::fs::write(path, &args.content).await?;

        let line_count = args.content.lines().count();
        let byte_count = args.content.len();

        Ok(format!(
            "Successfully wrote {byte_count} bytes ({line_count} lines) to {}",
            args.file_path
        ))
    }
}
