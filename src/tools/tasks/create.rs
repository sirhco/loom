use std::sync::Arc;

use chrono::Utc;
use rig::completion::ToolDefinition;
use rig::tool::Tool;
use serde::Deserialize;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::state::app_state::{AppState, Task, TaskStatus};

/// Tool for creating new tasks.
#[derive(Clone)]
pub struct TaskCreateTool {
    pub state: Arc<RwLock<AppState>>,
}

/// Arguments for the TaskCreateTool.
#[derive(Debug, Clone, Deserialize)]
pub struct TaskCreateArgs {
    /// The title/subject of the task.
    pub subject: String,
    /// A detailed description of the task.
    pub description: String,
}

/// Error type for TaskCreateTool.
#[derive(Debug, thiserror::Error)]
pub enum TaskCreateError {
    #[error("Failed to create task: {0}")]
    CreateFailed(String),
}

impl Tool for TaskCreateTool {
    const NAME: &'static str = "task_create";

    type Error = TaskCreateError;
    type Args = TaskCreateArgs;
    type Output = String;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: "task_create".to_string(),
            description: "Create a new task to track work items during the session.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "subject": {
                        "type": "string",
                        "description": "The title/subject of the task"
                    },
                    "description": {
                        "type": "string",
                        "description": "A detailed description of what needs to be done"
                    }
                },
                "required": ["subject", "description"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let now = Utc::now();
        let task_id = Uuid::new_v4().to_string()[..8].to_string();

        let task = Task {
            id: task_id.clone(),
            title: args.subject.clone(),
            description: args.description.clone(),
            status: TaskStatus::Pending,
            created_at: now,
            updated_at: now,
        };

        let mut state = self.state.write().await;
        state.tasks.push(task);

        Ok(format!(
            "Task created successfully:\n  ID: {task_id}\n  Subject: {}\n  Status: Pending",
            args.subject
        ))
    }
}
