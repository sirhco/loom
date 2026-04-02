use crate::engine::query_engine::QueryEngine;
use crate::provider::models::GeminiModel;
use crate::state::app_state::AppState;

/// Shows or switches the current model.
///
/// If no argument is given, displays the current model.
/// If an argument is given, switches to that model ID.
pub fn execute(args: &str, state: &mut AppState, engine: &mut QueryEngine) -> String {
    if args.is_empty() {
        let current = &state.model;
        let gemini_list = GeminiModel::all()
            .iter()
            .map(|m| format!("  {}", m.model_id()))
            .collect::<Vec<_>>()
            .join("\n");
        return format!(
            "Current model: {current}\n\nGemini models:\n{gemini_list}\n\n\
             For other providers, pass any model ID string (e.g. gpt-4o, claude-sonnet-4-20250514, llama3)."
        );
    }

    let new_model = args.trim().to_string();
    state.model = new_model.clone();
    engine.set_model(&new_model);
    format!("Switched to model: {new_model}")
}
