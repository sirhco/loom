use std::time::Duration;

use rig::completion::ToolDefinition;
use rig::tool::Tool;
use serde::{Deserialize, Serialize};
use tokio::process::Command;

/// Maximum output size in bytes (100KB).
const MAX_OUTPUT_BYTES: usize = 100 * 1024;

/// Default timeout in milliseconds (2 minutes).
const DEFAULT_TIMEOUT_MS: u64 = 120_000;

/// Tool for executing shell commands.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BashTool;

/// Arguments for the BashTool.
#[derive(Debug, Clone, Deserialize)]
pub struct BashArgs {
    /// The command to execute.
    pub command: String,
    /// A description of what the command does.
    pub description: Option<String>,
    /// Timeout in milliseconds (default 120000).
    pub timeout: Option<u64>,
    /// Whether to run the command in the background.
    pub run_in_background: Option<bool>,
}

/// Error type for BashTool.
#[derive(Debug, thiserror::Error)]
pub enum BashError {
    #[error("Command execution failed: {0}")]
    Execution(String),
    #[error("Command timed out after {0}ms")]
    Timeout(u64),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

impl Tool for BashTool {
    const NAME: &'static str = "bash";

    type Error = BashError;
    type Args = BashArgs;
    type Output = String;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: "bash".to_string(),
            description: "Execute a shell command and return its output. Use for running CLI tools, scripts, git commands, and other system operations.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "command": {
                        "type": "string",
                        "description": "The bash command to execute"
                    },
                    "description": {
                        "type": "string",
                        "description": "A brief description of what this command does"
                    },
                    "timeout": {
                        "type": "number",
                        "description": "Optional timeout in milliseconds (default 120000, max 600000)"
                    },
                    "run_in_background": {
                        "type": "boolean",
                        "description": "Set to true to run the command in the background"
                    }
                },
                "required": ["command"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let timeout_ms = args.timeout.unwrap_or(DEFAULT_TIMEOUT_MS).min(600_000);
        let timeout_duration = Duration::from_millis(timeout_ms);
        let run_in_background = args.run_in_background.unwrap_or(false);

        if run_in_background {
            // Spawn the command without waiting for completion.
            let _child = Command::new("sh")
                .arg("-c")
                .arg(&args.command)
                .stdin(std::process::Stdio::null())
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .spawn()
                .map_err(|e| BashError::Execution(format!("Failed to spawn background command: {e}")))?;

            return Ok(format!(
                "Command started in background: {}",
                args.command
            ));
        }

        let result = tokio::time::timeout(timeout_duration, async {
            Command::new("sh")
                .arg("-c")
                .arg(&args.command)
                .output()
                .await
        })
        .await;

        match result {
            Ok(Ok(output)) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);
                let exit_code = output.status.code().unwrap_or(-1);

                let mut combined = String::new();

                if !stdout.is_empty() {
                    combined.push_str(&stdout);
                }

                if !stderr.is_empty() {
                    if !combined.is_empty() {
                        combined.push('\n');
                    }
                    combined.push_str("STDERR:\n");
                    combined.push_str(&stderr);
                }

                if combined.is_empty() {
                    combined = "(no output)".to_string();
                }

                // Truncate if too large.
                if combined.len() > MAX_OUTPUT_BYTES {
                    combined.truncate(MAX_OUTPUT_BYTES);
                    combined.push_str("\n... (output truncated)");
                }

                if exit_code != 0 {
                    Ok(format!(
                        "Exit code: {exit_code}\n{combined}"
                    ))
                } else {
                    Ok(combined)
                }
            }
            Ok(Err(e)) => Err(BashError::Io(e)),
            Err(_) => Err(BashError::Timeout(timeout_ms)),
        }
    }
}
