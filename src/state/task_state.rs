use anyhow::{Result, bail};
use chrono::Utc;
use uuid::Uuid;

use crate::state::app_state::{AppState, Task, TaskStatus};

/// Creates a new task and adds it to the application state.
/// Returns the generated task ID.
pub fn create_task(state: &mut AppState, title: String, description: String) -> String {
    let id = Uuid::new_v4().to_string();
    let now = Utc::now();
    let task = Task {
        id: id.clone(),
        title,
        description,
        status: TaskStatus::Pending,
        created_at: now,
        updated_at: now,
    };
    state.tasks.push(task);
    id
}

/// Updates the status of an existing task by its ID.
pub fn update_task(state: &mut AppState, id: &str, status: TaskStatus) -> Result<()> {
    let task = state
        .tasks
        .iter_mut()
        .find(|t| t.id == id);

    match task {
        Some(t) => {
            t.status = status;
            t.updated_at = Utc::now();
            Ok(())
        }
        None => bail!("task not found: {id}"),
    }
}

/// Returns a slice of all tasks in the application state.
pub fn list_tasks(state: &AppState) -> &[Task] {
    &state.tasks
}

/// Finds a task by its ID.
pub fn get_task<'a>(state: &'a AppState, id: &str) -> Option<&'a Task> {
    state.tasks.iter().find(|t| t.id == id)
}
