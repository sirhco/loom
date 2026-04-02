use std::sync::Arc;

use rig::completion::ToolDefinition;
use rig::tool::Tool;
use serde::Deserialize;
use tokio::sync::RwLock;

use crate::state::app_state::AppState;

/// Tool for listing all tasks.
#[derive(Clone)]
pub struct TaskListTool {
    pub state: Arc<RwLock<AppState>>,
}

/// Arguments for the TaskListTool (none required).
#[derive(Debug, Clone, Deserialize)]
pub struct TaskListArgs {}

/// Error type for TaskListTool.
#[derive(Debug, thiserror::Error)]
pub enum TaskListError {
    #[error("Failed to list tasks: {0}")]
    ListFailed(String),
}

impl Tool for TaskListTool {
    const NAME: &'static str = "task_list";

    type Error = TaskListError;
    type Args = TaskListArgs;
    type Output = String;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: "task_list".to_string(),
            description: "List all tasks in the current session with their status.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
        }
    }

    async fn call(&self, _args: Self::Args) -> Result<Self::Output, Self::Error> {
        let state = self.state.read().await;
        let tasks = &state.tasks;

        if tasks.is_empty() {
            return Ok("No tasks found.".to_string());
        }

        let mut output = format!("Tasks ({} total):\n\n", tasks.len());

        for task in tasks {
            output.push_str(&format!(
                "  #{} [{}] {}\n    {}\n    Created: {} | Updated: {}\n\n",
                task.id,
                format!("{:?}", task.status).to_lowercase(),
                task.title,
                task.description,
                task.created_at.format("%Y-%m-%d %H:%M:%S"),
                task.updated_at.format("%Y-%m-%d %H:%M:%S"),
            ));
        }

        Ok(output)
    }
}
