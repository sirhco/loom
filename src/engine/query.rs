use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use anyhow::{Context, Result};
use rig::agent::{HookAction, PromptHook, ToolCallHookAction};
use rig::completion::{CompletionModel, Prompt};
use tokio::sync::mpsc;
use tokio::sync::RwLock;

use crate::engine::cost_tracker::CostTracker;
use crate::engine::messages::{Message, MessageRole, ToolCallInfo};
use crate::provider::Provider;
use crate::state::app_state::AppState;
use crate::tools::registry::ToolRegistry;
use crate::tui::event::AppEvent;

/// Maximum number of tool-call turns the agent is allowed.
const MAX_TURNS: usize = 25;

/// The result of executing a single query turn.
pub struct QueryResult {
    pub response_text: String,
    pub tool_calls_made: Vec<ToolCallInfo>,
    pub input_tokens: u64,
    pub output_tokens: u64,
}

/// Hook that emits progress events to the TUI during the agent's multi-turn loop.
///
/// Implements rig's `PromptHook` trait so the agent loop itself drives event emission
/// without requiring a hand-rolled completion loop.
#[derive(Clone)]
struct ProgressHook {
    tx: mpsc::UnboundedSender<AppEvent>,
    turn: Arc<AtomicUsize>,
    max_turns: usize,
}

impl ProgressHook {
    fn new(tx: mpsc::UnboundedSender<AppEvent>, max_turns: usize) -> Self {
        Self {
            tx,
            turn: Arc::new(AtomicUsize::new(0)),
            max_turns,
        }
    }
}

/// Truncate a string to the given byte-length limit, appending "..." if truncated.
fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        let mut end = max_len;
        while end > 0 && !s.is_char_boundary(end) {
            end -= 1;
        }
        format!("{}...", &s[..end])
    }
}

impl<M: CompletionModel> PromptHook<M> for ProgressHook {
    fn on_completion_call(
        &self,
        _prompt: &rig::message::Message,
        _history: &[rig::message::Message],
    ) -> impl std::future::Future<Output = HookAction> + Send {
        let current = self.turn.fetch_add(1, Ordering::SeqCst);
        // Only emit turn progress after the first turn (turn 0 is the initial prompt)
        if current > 0 {
            let _ = self.tx.send(AppEvent::TurnProgress {
                turn: current,
                max_turns: self.max_turns,
            });
        }
        async { HookAction::cont() }
    }

    fn on_tool_call(
        &self,
        tool_name: &str,
        _tool_call_id: Option<String>,
        _internal_call_id: &str,
        args: &str,
    ) -> impl std::future::Future<Output = ToolCallHookAction> + Send {
        let _ = self.tx.send(AppEvent::ToolCallStarted {
            name: tool_name.to_string(),
            args_preview: truncate(args, 120),
        });
        async { ToolCallHookAction::cont() }
    }

    fn on_tool_result(
        &self,
        tool_name: &str,
        _tool_call_id: Option<String>,
        _internal_call_id: &str,
        _args: &str,
        result: &str,
    ) -> impl std::future::Future<Output = HookAction> + Send {
        let _ = self.tx.send(AppEvent::ToolCallCompleted {
            name: tool_name.to_string(),
            result_preview: truncate(result, 200),
            is_error: false,
        });
        async { HookAction::cont() }
    }
}

/// Converts our internal message history into rig's `Message` type for chat history.
pub fn messages_to_rig_chat_history(messages: &[Message]) -> Vec<rig::completion::Message> {
    messages
        .iter()
        .filter_map(|msg| {
            let text = msg.text_content()?;
            match msg.role {
                MessageRole::User => Some(rig::completion::Message::User {
                    content: rig::OneOrMany::one(rig::message::UserContent::Text(
                        rig::message::Text {
                            text: text.to_string(),
                        },
                    )),
                }),
                MessageRole::Assistant => Some(rig::completion::Message::Assistant {
                    id: None,
                    content: rig::OneOrMany::one(rig::message::AssistantContent::Text(
                        rig::message::Text {
                            text: text.to_string(),
                        },
                    )),
                }),
                MessageRole::System => Some(rig::completion::Message::System {
                    content: text.to_string(),
                }),
                // Tool results are typically paired with tool calls in the history;
                // for now we include them as user messages with the result text.
                MessageRole::ToolResult => Some(rig::completion::Message::User {
                    content: rig::OneOrMany::one(rig::message::UserContent::Text(
                        rig::message::Text {
                            text: text.to_string(),
                        },
                    )),
                }),
            }
        })
        .collect()
}

/// Converts a rig response string into our internal Message type.
pub fn rig_response_to_message(response: &str) -> Message {
    Message::assistant(response)
}

/// Executes a single query turn against the Gemini model.
///
/// Dispatches to either the direct Gemini API or Vertex AI based on the
/// configured provider, then runs the common agent logic via `run_agent_query`.
pub async fn execute_query(
    provider: &Provider,
    model_id: &str,
    system_prompt: &str,
    messages: &[Message],
    user_input: &str,
    tool_registry: ToolRegistry,
    state: &Arc<RwLock<AppState>>,
    cost_tracker: &mut CostTracker,
    progress_tx: Option<&mpsc::UnboundedSender<AppEvent>>,
) -> Result<QueryResult> {
    match provider {
        Provider::Gemini(client) => {
            let completion_model = client.completion_model(model_id);
            run_agent_query(completion_model, model_id, system_prompt, messages, user_input, tool_registry, state, cost_tracker, progress_tx).await
        }
        Provider::Vertex(client) => {
            let completion_model = client.completion_model(model_id);
            run_agent_query(completion_model, model_id, system_prompt, messages, user_input, tool_registry, state, cost_tracker, progress_tx).await
        }
        Provider::Anthropic(client) => {
            let completion_model = client.completion_model(model_id);
            run_agent_query(completion_model, model_id, system_prompt, messages, user_input, tool_registry, state, cost_tracker, progress_tx).await
        }
        Provider::OpenAI(client) => {
            let completion_model = client.completion_model(model_id);
            run_agent_query(completion_model, model_id, system_prompt, messages, user_input, tool_registry, state, cost_tracker, progress_tx).await
        }
        Provider::Azure(client) => {
            let completion_model = client.completion_model(model_id);
            run_agent_query(completion_model, model_id, system_prompt, messages, user_input, tool_registry, state, cost_tracker, progress_tx).await
        }
        Provider::Cohere(client) => {
            let completion_model = client.completion_model(model_id);
            run_agent_query(completion_model, model_id, system_prompt, messages, user_input, tool_registry, state, cost_tracker, progress_tx).await
        }
        Provider::DeepSeek(client) => {
            let completion_model = client.completion_model(model_id);
            run_agent_query(completion_model, model_id, system_prompt, messages, user_input, tool_registry, state, cost_tracker, progress_tx).await
        }
        Provider::Galadriel(client) => {
            let completion_model = client.completion_model(model_id);
            run_agent_query(completion_model, model_id, system_prompt, messages, user_input, tool_registry, state, cost_tracker, progress_tx).await
        }
        Provider::Groq(client) => {
            let completion_model = client.completion_model(model_id);
            run_agent_query(completion_model, model_id, system_prompt, messages, user_input, tool_registry, state, cost_tracker, progress_tx).await
        }
        Provider::HuggingFace(client) => {
            let completion_model = client.completion_model(model_id);
            run_agent_query(completion_model, model_id, system_prompt, messages, user_input, tool_registry, state, cost_tracker, progress_tx).await
        }
        Provider::Hyperbolic(client) => {
            let completion_model = client.completion_model(model_id);
            run_agent_query(completion_model, model_id, system_prompt, messages, user_input, tool_registry, state, cost_tracker, progress_tx).await
        }
        Provider::Llamafile(client) => {
            let completion_model = client.completion_model(model_id);
            run_agent_query(completion_model, model_id, system_prompt, messages, user_input, tool_registry, state, cost_tracker, progress_tx).await
        }
        Provider::Mira(client) => {
            let completion_model = client.completion_model(model_id);
            run_agent_query(completion_model, model_id, system_prompt, messages, user_input, tool_registry, state, cost_tracker, progress_tx).await
        }
        Provider::Mistral(client) => {
            let completion_model = client.completion_model(model_id);
            run_agent_query(completion_model, model_id, system_prompt, messages, user_input, tool_registry, state, cost_tracker, progress_tx).await
        }
        Provider::Moonshot(client) => {
            let completion_model = client.completion_model(model_id);
            run_agent_query(completion_model, model_id, system_prompt, messages, user_input, tool_registry, state, cost_tracker, progress_tx).await
        }
        Provider::Ollama(client) => {
            let completion_model = client.completion_model(model_id);
            run_agent_query(completion_model, model_id, system_prompt, messages, user_input, tool_registry, state, cost_tracker, progress_tx).await
        }
        Provider::OpenRouter(client) => {
            let completion_model = client.completion_model(model_id);
            run_agent_query(completion_model, model_id, system_prompt, messages, user_input, tool_registry, state, cost_tracker, progress_tx).await
        }
        Provider::Perplexity(client) => {
            let completion_model = client.completion_model(model_id);
            run_agent_query(completion_model, model_id, system_prompt, messages, user_input, tool_registry, state, cost_tracker, progress_tx).await
        }
        Provider::Together(client) => {
            let completion_model = client.completion_model(model_id);
            run_agent_query(completion_model, model_id, system_prompt, messages, user_input, tool_registry, state, cost_tracker, progress_tx).await
        }
        Provider::XAI(client) => {
            let completion_model = client.completion_model(model_id);
            run_agent_query(completion_model, model_id, system_prompt, messages, user_input, tool_registry, state, cost_tracker, progress_tx).await
        }
    }
}

/// Runs the agent query loop with a concrete completion model type.
///
/// This is the shared implementation used by both Gemini and Vertex AI providers.
/// It builds the rig agent, sends the prompt with history, and processes the response.
async fn run_agent_query<M: CompletionModel + 'static>(
    completion_model: M,
    model_id: &str,
    system_prompt: &str,
    messages: &[Message],
    user_input: &str,
    tool_registry: ToolRegistry,
    state: &Arc<RwLock<AppState>>,
    cost_tracker: &mut CostTracker,
    progress_tx: Option<&mpsc::UnboundedSender<AppEvent>>,
) -> Result<QueryResult> {
    let agent = tool_registry
        .build_agent(completion_model, system_prompt)
        .context("failed to build rig agent")?;

    // Convert existing messages to rig chat history
    let chat_history = messages_to_rig_chat_history(messages);

    // Send the prompt with chat history and allow multi-turn tool calls.
    // When we have a progress sender, attach a PromptHook that emits events
    // for tool calls and turn progress without changing the underlying loop.
    let response = if let Some(tx) = progress_tx {
        let hook = ProgressHook::new(tx.clone(), MAX_TURNS);
        agent
            .prompt(user_input)
            .with_history(chat_history)
            .max_turns(MAX_TURNS)
            .with_hook(hook)
            .extended_details()
            .await
            .map_err(|e| anyhow::anyhow!("API error: {e}"))?
    } else {
        agent
            .prompt(user_input)
            .with_history(chat_history)
            .max_turns(MAX_TURNS)
            .extended_details()
            .await
            .map_err(|e| anyhow::anyhow!("API error: {e}"))?
    };

    let response_text = response.output.clone();
    let input_tokens = response.usage.input_tokens;
    let output_tokens = response.usage.output_tokens;

    // Record cost
    cost_tracker.record(model_id, input_tokens, output_tokens);

    // Update shared app state with cumulative totals
    {
        let mut app = state.write().await;
        app.total_input_tokens += input_tokens;
        app.total_output_tokens += output_tokens;
        app.total_cost_usd = cost_tracker.total_cost();
    }

    // Extract any tool calls from the response messages if available
    let tool_calls_made = extract_tool_calls_from_history(&response.messages);

    Ok(QueryResult {
        response_text,
        tool_calls_made,
        input_tokens,
        output_tokens,
    })
}

/// Extracts tool call information from rig's message history.
fn extract_tool_calls_from_history(
    messages: &Option<Vec<rig::completion::Message>>,
) -> Vec<ToolCallInfo> {
    let Some(msgs) = messages else {
        return Vec::new();
    };

    let mut tool_calls = Vec::new();
    for msg in msgs {
        if let rig::completion::Message::Assistant { content, .. } = msg {
            for item in content.iter() {
                if let rig::message::AssistantContent::ToolCall(tc) = item {
                    tool_calls.push(ToolCallInfo {
                        id: tc.id.clone(),
                        name: tc.function.name.clone(),
                        arguments: tc.function.arguments.clone(),
                    });
                }
            }
        }
    }
    tool_calls
}
