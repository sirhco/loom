use crate::engine::messages::{Message, MessageContent};
use crate::provider::models::GeminiModel;

/// Rough estimate of tokens per character for cost/context estimation.
const CHARS_PER_TOKEN: usize = 4;

/// Threshold fraction of context window at which compaction is triggered.
const COMPACTION_THRESHOLD: f64 = 0.80;

/// Default context window size for non-Gemini models (128k tokens).
const DEFAULT_CONTEXT_WINDOW: usize = 128_000;

/// Estimates the token count for a slice of messages.
fn estimate_tokens(messages: &[Message]) -> usize {
    messages
        .iter()
        .map(|msg| {
            let text_len = match &msg.content {
                MessageContent::Text(text) => text.len(),
                MessageContent::ToolCall(info) => {
                    info.name.len() + info.arguments.to_string().len()
                }
                MessageContent::ToolResult(info) => info.content.len(),
                MessageContent::MultiPart(parts) => parts
                    .iter()
                    .map(|part| match part {
                        crate::engine::messages::ContentPart::Text(t) => t.len(),
                        crate::engine::messages::ContentPart::ToolCall(info) => {
                            info.name.len() + info.arguments.to_string().len()
                        }
                        crate::engine::messages::ContentPart::ToolResult(info) => {
                            info.content.len()
                        }
                    })
                    .sum(),
            };
            // Add overhead for role, timestamp, etc.
            (text_len + 50) / CHARS_PER_TOKEN
        })
        .sum()
}

/// Returns true if the estimated token usage exceeds 80% of the model's context window.
pub fn should_compact(messages: &[Message], model_id: &str) -> bool {
    let estimated = estimate_tokens(messages);
    let context_window = model_id
        .parse::<GeminiModel>()
        .map(|m| m.context_window())
        .unwrap_or(DEFAULT_CONTEXT_WINDOW);
    let threshold = (context_window as f64 * COMPACTION_THRESHOLD) as usize;
    estimated > threshold
}

/// Compacts the message history by replacing older messages with a summary.
///
/// Keeps the most recent `keep_recent` messages and replaces everything before
/// them with a single system message summarizing the compacted history.
pub fn compact_messages(messages: &mut Vec<Message>, keep_recent: usize) {
    if messages.len() <= keep_recent {
        return;
    }

    let removed_count = messages.len() - keep_recent;
    let summary_text = format!(
        "[Earlier conversation compacted. Key context: {} messages were exchanged about various topics.]",
        removed_count
    );

    // Keep only the most recent messages
    let recent: Vec<Message> = messages.split_off(messages.len() - keep_recent);

    // Replace the old messages with a summary
    messages.clear();
    messages.push(Message::system(&summary_text));
    messages.extend(recent);
}
