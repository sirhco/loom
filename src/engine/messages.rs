use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A message in the conversation history.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: MessageRole,
    pub content: MessageContent,
    pub timestamp: DateTime<Utc>,
    pub uuid: String,
    pub usage: Option<TokenUsage>,
}

/// The role of a message sender.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessageRole {
    User,
    Assistant,
    System,
    ToolResult,
}

/// Content of a message, which can be plain text, a tool call, a tool result, or multi-part.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageContent {
    Text(String),
    ToolCall(ToolCallInfo),
    ToolResult(ToolResultInfo),
    MultiPart(Vec<ContentPart>),
}

/// A single part within a multi-part message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContentPart {
    Text(String),
    ToolCall(ToolCallInfo),
    ToolResult(ToolResultInfo),
}

/// Information about a tool call made by the assistant.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallInfo {
    pub id: String,
    pub name: String,
    pub arguments: serde_json::Value,
}

/// Information about the result of a tool call.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResultInfo {
    pub tool_call_id: String,
    pub content: String,
    pub is_error: bool,
}

/// Token usage for a single message or turn.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenUsage {
    pub input_tokens: u64,
    pub output_tokens: u64,
}

impl Message {
    /// Creates a new user message with the given text.
    pub fn user(text: &str) -> Self {
        Self {
            role: MessageRole::User,
            content: MessageContent::Text(text.to_string()),
            timestamp: Utc::now(),
            uuid: Uuid::new_v4().to_string(),
            usage: None,
        }
    }

    /// Creates a new assistant message with the given text.
    pub fn assistant(text: &str) -> Self {
        Self {
            role: MessageRole::Assistant,
            content: MessageContent::Text(text.to_string()),
            timestamp: Utc::now(),
            uuid: Uuid::new_v4().to_string(),
            usage: None,
        }
    }

    /// Creates a new system message with the given text.
    pub fn system(text: &str) -> Self {
        Self {
            role: MessageRole::System,
            content: MessageContent::Text(text.to_string()),
            timestamp: Utc::now(),
            uuid: Uuid::new_v4().to_string(),
            usage: None,
        }
    }

    /// Extracts text content from the message.
    ///
    /// For `Text` content, returns the text directly.
    /// For `MultiPart` content, returns the first text part found.
    /// Returns `None` for `ToolCall` and `ToolResult` variants.
    pub fn text_content(&self) -> Option<&str> {
        match &self.content {
            MessageContent::Text(text) => Some(text.as_str()),
            MessageContent::MultiPart(parts) => {
                for part in parts {
                    if let ContentPart::Text(text) = part {
                        return Some(text.as_str());
                    }
                }
                None
            }
            MessageContent::ToolCall(_) | MessageContent::ToolResult(_) => None,
        }
    }
}
