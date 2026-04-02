use crate::provider::models::GeminiModel;
use crate::state::app_state::AppState;

/// Displays the current configuration.
pub fn execute(state: &AppState) -> String {
    let mut lines = Vec::new();
    lines.push("Current configuration:".to_string());
    lines.push(format!("  Model:           {}", state.model));
    lines.push(format!("  Permission mode: {:?}", state.permission_mode));
    lines.push(format!("  Verbose:         {}", state.verbose));
    lines.push(format!("  Working dir:     {}", state.cwd.display()));
    lines.push(format!("  Session ID:      {}", state.session_id));

    // Show Gemini-specific info if the model is a known Gemini model
    if let Ok(gm) = state.model.parse::<GeminiModel>() {
        lines.push(format!(
            "  Context window:  {} tokens",
            gm.context_window()
        ));
        lines.push(format!(
            "  Max output:      {} tokens",
            gm.max_output_tokens()
        ));
    }

    lines.join("\n")
}
