use rig::completion::ToolDefinition;
use rig::tool::Tool;
use serde::{Deserialize, Serialize};

/// Tool for spawning sub-agents (placeholder implementation).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentTool;

/// Arguments for the AgentTool.
#[derive(Debug, Clone, Deserialize)]
pub struct AgentArgs {
    /// The prompt to send to the sub-agent.
    pub prompt: String,
    /// A description of what the sub-agent should do.
    pub description: Option<String>,
    /// The model to use for the sub-agent.
    pub model: Option<String>,
}

/// Error type for AgentTool.
#[derive(Debug, thiserror::Error)]
pub enum AgentError {
    #[error("Sub-agent error: {0}")]
    SubAgentError(String),
}

impl Tool for AgentTool {
    const NAME: &'static str = "agent";

    type Error = AgentError;
    type Args = AgentArgs;
    type Output = String;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: "agent".to_string(),
            description: "Spawn a sub-agent to handle a complex task independently. The sub-agent gets its own context and can use tools. Use for parallel or delegated work.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "prompt": {
                        "type": "string",
                        "description": "The prompt/instructions for the sub-agent"
                    },
                    "description": {
                        "type": "string",
                        "description": "A brief description of what the sub-agent should accomplish"
                    },
                    "model": {
                        "type": "string",
                        "description": "The model to use for the sub-agent (default: same as parent)"
                    }
                },
                "required": ["prompt"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        // Placeholder: sub-agent execution is not yet implemented.
        // This will be filled in with actual sub-agent logic that:
        // 1. Creates a new agent with its own tool set
        // 2. Sends the prompt to the sub-agent
        // 3. Collects and returns the sub-agent's response
        let description = args
            .description
            .as_deref()
            .unwrap_or("(no description provided)");
        let model = args.model.as_deref().unwrap_or("(default model)");

        Ok(format!(
            "Sub-agent execution is not yet implemented.\n\
             Prompt: \"{}\"\n\
             Description: {}\n\
             Model: {}\n\n\
             Please perform the requested task directly instead of delegating to a sub-agent.",
            args.prompt, description, model
        ))
    }
}
