use crate::engine::query_engine::QueryEngine;

/// Displays the current token usage and cost summary.
pub fn execute(engine: &QueryEngine) -> String {
    let tracker = engine.cost_tracker();
    let mut lines = Vec::new();
    lines.push("Session cost summary:".to_string());
    lines.push(format!("  {}", tracker.summary()));
    lines.push(format!(
        "  Messages in history: {}",
        engine.messages().len()
    ));
    lines.join("\n")
}
