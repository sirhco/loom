use std::io;

use rig::completion::ToolDefinition;
use rig::tool::Tool;
use serde::{Deserialize, Serialize};

/// Tool for prompting the user for input.
///
/// In TUI mode, direct stdin reads are not possible (crossterm raw mode).
/// Instead, this tool returns a message indicating it asked the user,
/// and the model should proceed with its best judgment or note the question.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AskUserTool;

/// Arguments for the AskUserTool.
#[derive(Debug, Clone, Deserialize)]
pub struct AskUserArgs {
    /// The question to ask the user.
    pub question: String,
}

/// Error type for AskUserTool.
#[derive(Debug, thiserror::Error)]
pub enum AskUserError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
}

impl Tool for AskUserTool {
    const NAME: &'static str = "ask_user";

    type Error = AskUserError;
    type Args = AskUserArgs;
    type Output = String;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: "ask_user".to_string(),
            description: "Ask the user a question. The question will be displayed to the user. \
                Since the user cannot respond in real-time during tool execution, \
                proceed with your best judgment after asking."
                .to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "question": {
                        "type": "string",
                        "description": "The question to ask the user"
                    }
                },
                "required": ["question"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        // In TUI mode we cannot block on stdin (crossterm raw mode).
        // The question is shown to the user via the ToolCallStarted event.
        // Return a message so the model can proceed.
        Ok(format!(
            "Question displayed to user: \"{}\". \
             The user cannot respond during tool execution. \
             Please proceed with your best judgment or continue with the task.",
            args.question
        ))
    }
}
