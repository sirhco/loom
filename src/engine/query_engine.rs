use std::sync::Arc;

use anyhow::Result;
use tokio::sync::mpsc;
use tokio::sync::RwLock;

use crate::engine::compaction::{compact_messages, should_compact};
use crate::engine::cost_tracker::CostTracker;
use crate::engine::messages::Message;
use crate::engine::query::{execute_query, rig_response_to_message};
use crate::provider::Provider;
use crate::state::app_state::AppState;
use crate::tools::registry::ToolRegistry;
use crate::tui::event::AppEvent;

/// Default number of recent messages to keep during compaction.
const COMPACTION_KEEP_RECENT: usize = 20;

/// The QueryEngine orchestrates multi-turn conversations with the AI model.
///
/// It manages message history, cost tracking, context compaction, and delegates
/// each turn to the underlying query execution logic.
pub struct QueryEngine {
    provider: Provider,
    model: String,
    cost_tracker: CostTracker,
    messages: Vec<Message>,
    system_prompt: String,
    state: Arc<RwLock<AppState>>,
    progress_tx: Option<mpsc::UnboundedSender<AppEvent>>,
}

impl QueryEngine {
    /// Creates a new QueryEngine with the given configuration.
    pub fn new(
        provider: Provider,
        model: String,
        _tool_registry: ToolRegistry,
        state: Arc<RwLock<AppState>>,
        system_prompt: String,
    ) -> Self {
        Self {
            provider,
            model,
            cost_tracker: CostTracker::new(),
            messages: Vec::new(),
            system_prompt,
            state,
            progress_tx: None,
        }
    }

    /// Sets the progress event sender for TUI feedback during query execution.
    ///
    /// When set, the engine emits `AppEvent::ToolCallStarted`,
    /// `AppEvent::ToolCallCompleted`, and `AppEvent::TurnProgress` events
    /// as the agent loop runs.
    pub fn set_progress_tx(&mut self, tx: mpsc::UnboundedSender<AppEvent>) {
        self.progress_tx = Some(tx);
    }

    /// Processes a user input and returns the assistant's response text.
    ///
    /// This is the main entry point for each conversation turn. It:
    /// 1. Adds the user message to history
    /// 2. Compacts history if the context window is getting full
    /// 3. Sends the query to the model
    /// 4. Adds the assistant response to history
    /// 5. Returns the response text
    pub async fn query(&mut self, user_input: &str) -> Result<String> {
        // Add user message to history
        self.messages.push(Message::user(user_input));

        // Check if we need to compact before sending
        self.compact_if_needed().await;

        // Create a fresh tool registry for this query turn.
        // The registry is consumed when building the agent.
        let tool_registry = ToolRegistry::new(self.state.clone());

        // Execute the query
        let result = execute_query(
            &self.provider,
            &self.model,
            &self.system_prompt,
            &self.messages,
            user_input,
            tool_registry,
            &self.state,
            &mut self.cost_tracker,
            self.progress_tx.as_ref(),
        )
        .await?;

        // Add assistant response to history
        let mut assistant_msg = rig_response_to_message(&result.response_text);
        assistant_msg.usage = Some(crate::engine::messages::TokenUsage {
            input_tokens: result.input_tokens,
            output_tokens: result.output_tokens,
        });
        self.messages.push(assistant_msg);

        Ok(result.response_text)
    }

    /// Returns the current message history.
    pub fn messages(&self) -> &[Message] {
        &self.messages
    }

    /// Clears all messages from the conversation history.
    pub fn clear_messages(&mut self) {
        self.messages.clear();
    }

    /// Switches to a different model.
    pub fn set_model(&mut self, model: &str) {
        self.model = model.to_string();
    }

    /// Replaces the current system prompt (for mode switching).
    pub fn set_system_prompt(&mut self, prompt: String) {
        self.system_prompt = prompt;
    }

    /// Returns the current system prompt.
    pub fn system_prompt(&self) -> &str {
        &self.system_prompt
    }

    /// Queries using a specific tool registry override (e.g. read-only for planning).
    pub async fn query_with_registry(
        &mut self,
        user_input: &str,
        tool_registry: ToolRegistry,
    ) -> Result<String> {
        self.messages.push(Message::user(user_input));
        self.compact_if_needed().await;

        let result = execute_query(
            &self.provider,
            &self.model,
            &self.system_prompt,
            &self.messages,
            user_input,
            tool_registry,
            &self.state,
            &mut self.cost_tracker,
            self.progress_tx.as_ref(),
        )
        .await?;

        let mut assistant_msg = rig_response_to_message(&result.response_text);
        assistant_msg.usage = Some(crate::engine::messages::TokenUsage {
            input_tokens: result.input_tokens,
            output_tokens: result.output_tokens,
        });
        self.messages.push(assistant_msg);

        Ok(result.response_text)
    }

    /// Returns a reference to the cost tracker.
    pub fn cost_tracker(&self) -> &CostTracker {
        &self.cost_tracker
    }

    /// Checks whether the context window is getting full and compacts if needed.
    pub async fn compact_if_needed(&mut self) {
        if should_compact(&self.messages, &self.model) {
            compact_messages(&mut self.messages, COMPACTION_KEEP_RECENT);
            tracing::info!(
                "Compacted message history to {} messages",
                self.messages.len()
            );
        }
    }
}
