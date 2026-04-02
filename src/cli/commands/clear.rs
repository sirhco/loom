use crate::engine::query_engine::QueryEngine;

/// Clears the conversation history and returns a confirmation message.
pub fn execute(engine: &mut QueryEngine) -> String {
    engine.clear_messages();
    "Conversation history cleared.".to_string()
}
