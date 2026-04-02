use std::path::Path;

use rig::completion::ToolDefinition;
use rig::tool::Tool;
use serde::{Deserialize, Serialize};

/// Default number of lines to read.
const DEFAULT_LIMIT: usize = 2000;

/// Maximum total output size in bytes.
const MAX_OUTPUT_BYTES: usize = 200 * 1024;

/// Tool for reading file contents with line numbers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileReadTool;

/// Arguments for the FileReadTool.
#[derive(Debug, Clone, Deserialize)]
pub struct FileReadArgs {
    /// The absolute path to the file to read.
    pub file_path: String,
    /// Line number to start reading from (0-based offset, default 0).
    pub offset: Option<usize>,
    /// Number of lines to read (default 2000).
    pub limit: Option<usize>,
}

/// Error type for FileReadTool.
#[derive(Debug, thiserror::Error)]
pub enum FileReadError {
    #[error("File not found: {0}")]
    NotFound(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Binary file detected: {0}")]
    BinaryFile(String),
}

impl Tool for FileReadTool {
    const NAME: &'static str = "read_file";

    type Error = FileReadError;
    type Args = FileReadArgs;
    type Output = String;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: "read_file".to_string(),
            description: "Read a file from the filesystem. Returns contents with line numbers. Use for viewing source code, configuration files, and text files.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "file_path": {
                        "type": "string",
                        "description": "The absolute path to the file to read"
                    },
                    "offset": {
                        "type": "number",
                        "description": "Line number to start reading from (0-based, default 0)"
                    },
                    "limit": {
                        "type": "number",
                        "description": "Number of lines to read (default 2000)"
                    }
                },
                "required": ["file_path"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let path = Path::new(&args.file_path);

        if !path.exists() {
            return Err(FileReadError::NotFound(args.file_path));
        }

        let bytes = tokio::fs::read(path).await?;

        // Simple binary detection: check for null bytes in the first 8KB.
        let check_len = bytes.len().min(8192);
        if bytes[..check_len].contains(&0) {
            return Err(FileReadError::BinaryFile(args.file_path));
        }

        let content = String::from_utf8_lossy(&bytes);
        let lines: Vec<&str> = content.lines().collect();
        let total_lines = lines.len();

        let offset = args.offset.unwrap_or(0);
        let limit = args.limit.unwrap_or(DEFAULT_LIMIT);

        if offset >= total_lines {
            return Ok(format!(
                "File has {total_lines} lines. Offset {offset} is beyond the end of the file."
            ));
        }

        let end = (offset + limit).min(total_lines);
        let selected = &lines[offset..end];

        let mut output = String::new();
        for (i, line) in selected.iter().enumerate() {
            let line_num = offset + i + 1; // 1-based line numbers
            output.push_str(&format!("{line_num}\t{line}\n"));

            // Safety check for extremely large files.
            if output.len() > MAX_OUTPUT_BYTES {
                output.push_str(&format!(
                    "... (output truncated at {MAX_OUTPUT_BYTES} bytes, showing lines {}-{})",
                    offset + 1,
                    offset + i + 1
                ));
                break;
            }
        }

        if end < total_lines {
            output.push_str(&format!(
                "\n(Showing lines {}-{} of {total_lines}. Use offset/limit to read more.)",
                offset + 1,
                end
            ));
        }

        Ok(output)
    }
}
