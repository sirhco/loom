use crate::provider::models::GeminiModel;

/// Tracks token usage and cost across the session.
pub struct CostTracker {
    turns: Vec<TurnUsage>,
}

/// Token usage for a single turn (one request/response cycle).
struct TurnUsage {
    model: Option<GeminiModel>,
    input_tokens: u64,
    output_tokens: u64,
    #[allow(dead_code)]
    timestamp: std::time::Instant,
}

impl CostTracker {
    /// Creates a new empty cost tracker.
    pub fn new() -> Self {
        Self { turns: Vec::new() }
    }

    /// Records token usage for a single turn.
    ///
    /// The `model_id` string is parsed to see if it matches a known Gemini model
    /// for cost estimation. Non-Gemini models record zero cost.
    pub fn record(&mut self, model_id: &str, input_tokens: u64, output_tokens: u64) {
        let model = model_id.parse::<GeminiModel>().ok();
        self.turns.push(TurnUsage {
            model,
            input_tokens,
            output_tokens,
            timestamp: std::time::Instant::now(),
        });
    }

    /// Returns the total number of input tokens across all turns.
    pub fn total_input_tokens(&self) -> u64 {
        self.turns.iter().map(|t| t.input_tokens).sum()
    }

    /// Returns the total number of output tokens across all turns.
    pub fn total_output_tokens(&self) -> u64 {
        self.turns.iter().map(|t| t.output_tokens).sum()
    }

    /// Computes the total estimated cost in USD across all turns.
    ///
    /// Only Gemini models have known pricing; other providers report zero cost.
    pub fn total_cost(&self) -> f64 {
        self.turns
            .iter()
            .map(|t| {
                if let Some(model) = &t.model {
                    let input_cost =
                        (t.input_tokens as f64 / 1_000_000.0) * model.input_cost_per_million();
                    let output_cost =
                        (t.output_tokens as f64 / 1_000_000.0) * model.output_cost_per_million();
                    input_cost + output_cost
                } else {
                    0.0
                }
            })
            .sum()
    }

    /// Returns a formatted summary of usage and cost.
    pub fn summary(&self) -> String {
        let input = self.total_input_tokens();
        let output = self.total_output_tokens();
        let cost = self.total_cost();

        format!(
            "Tokens: {} input, {} output ({} total) | Cost: ${:.4} | Turns: {}",
            input,
            output,
            input + output,
            cost,
            self.turns.len(),
        )
    }

    /// Returns the number of recorded turns.
    pub fn turn_count(&self) -> usize {
        self.turns.len()
    }
}

impl Default for CostTracker {
    fn default() -> Self {
        Self::new()
    }
}
