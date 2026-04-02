use crate::engine::query_engine::QueryEngine;

/// Forces context compaction and returns a status message.
pub async fn execute(engine: &mut QueryEngine) -> String {
    let before = engine.messages().len();
    engine.compact_if_needed().await;
    let after = engine.messages().len();

    if after < before {
        format!(
            "Compacted conversation: {} messages -> {} messages.",
            before, after
        )
    } else {
        format!(
            "No compaction needed ({} messages, within context limits).",
            before
        )
    }
}
