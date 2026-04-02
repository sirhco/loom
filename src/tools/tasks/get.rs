use std::sync::Arc;

use rig::completion::ToolDefinition;
use rig::tool::Tool;
use serde::Deserialize;
use tokio::sync::RwLock;

use crate::state::app_state::AppState;

/// Tool for getting details of a specific task.
#[derive(Clone)]
pub struct TaskGetTool {
    pub state: Arc<RwLock<AppState>>,
}

/// Arguments for the TaskGetTool.
#[derive(Debug, Clone, Deserialize)]
pub struct TaskGetArgs {
    /// The task ID to retrieve.
    pub task_id: String,
}

/// Error type for TaskGetTool.
#[derive(Debug, thiserror::Error)]
pub enum TaskGetError {
    #[error("Task not found: {0}")]
    TaskNotFound(String),
}

impl Tool for TaskGetTool {
    const NAME: &'static str = "task_get";

    type Error = TaskGetError;
    type Args = TaskGetArgs;
    type Output = String;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: "task_get".to_string(),
            description: "Get the details of a specific task by ID.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "task_id": {
                        "type": "string",
                        "description": "The ID of the task to retrieve"
                    }
                },
                "required": ["task_id"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let state = self.state.read().await;

        let task = state
            .tasks
            .iter()
            .find(|t| t.id == args.task_id)
            .ok_or_else(|| TaskGetError::TaskNotFound(args.task_id.clone()))?;

        Ok(format!(
            "Task #{}\n  Title: {}\n  Description: {}\n  Status: {:?}\n  Created: {}\n  Updated: {}",
            task.id,
            task.title,
            task.description,
            task.status,
            task.created_at.format("%Y-%m-%d %H:%M:%S UTC"),
            task.updated_at.format("%Y-%m-%d %H:%M:%S UTC"),
        ))
    }
}
