use std::sync::Arc;

use chrono::Utc;
use rig::completion::ToolDefinition;
use rig::tool::Tool;
use serde::Deserialize;
use tokio::sync::RwLock;

use crate::state::app_state::{AppState, TaskStatus};

/// Tool for updating task status.
#[derive(Clone)]
pub struct TaskUpdateTool {
    pub state: Arc<RwLock<AppState>>,
}

/// Arguments for the TaskUpdateTool.
#[derive(Debug, Clone, Deserialize)]
pub struct TaskUpdateArgs {
    /// The task ID to update.
    pub task_id: String,
    /// The new status: "pending", "in_progress", "completed", or "failed".
    pub status: String,
}

/// Error type for TaskUpdateTool.
#[derive(Debug, thiserror::Error)]
pub enum TaskUpdateError {
    #[error("Task not found: {0}")]
    TaskNotFound(String),
    #[error("Invalid status: {0}. Must be one of: pending, in_progress, completed, failed")]
    InvalidStatus(String),
}

fn parse_status(s: &str) -> Result<TaskStatus, TaskUpdateError> {
    match s.to_lowercase().as_str() {
        "pending" => Ok(TaskStatus::Pending),
        "in_progress" | "inprogress" | "in-progress" => Ok(TaskStatus::InProgress),
        "completed" | "complete" | "done" => Ok(TaskStatus::Completed),
        "failed" | "fail" | "error" => Ok(TaskStatus::Failed),
        other => Err(TaskUpdateError::InvalidStatus(other.to_string())),
    }
}

impl Tool for TaskUpdateTool {
    const NAME: &'static str = "task_update";

    type Error = TaskUpdateError;
    type Args = TaskUpdateArgs;
    type Output = String;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: "task_update".to_string(),
            description: "Update the status of an existing task.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "task_id": {
                        "type": "string",
                        "description": "The ID of the task to update"
                    },
                    "status": {
                        "type": "string",
                        "description": "The new status: \"pending\", \"in_progress\", \"completed\", or \"failed\""
                    }
                },
                "required": ["task_id", "status"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let new_status = parse_status(&args.status)?;
        let mut state = self.state.write().await;

        let task = state
            .tasks
            .iter_mut()
            .find(|t| t.id == args.task_id)
            .ok_or_else(|| TaskUpdateError::TaskNotFound(args.task_id.clone()))?;

        let old_status = format!("{:?}", task.status).to_lowercase();
        task.status = new_status;
        task.updated_at = Utc::now();
        let new_status_str = format!("{:?}", task.status).to_lowercase();

        Ok(format!(
            "Task #{} updated: {} -> {}\n  Title: {}",
            args.task_id, old_status, new_status_str, task.title
        ))
    }
}
