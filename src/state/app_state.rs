use std::path::PathBuf;
use std::time::Instant;

use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::cli::args::CliArgs;
use crate::config::settings::Settings;
use crate::permissions::mode::PermissionMode;

/// Planning mode phases.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlanMode {
    /// Normal operation — no plan active.
    Off,
    /// AI is researching the codebase with read-only tools.
    Researching,
    /// A plan has been generated and awaits user approval.
    PlanReady,
}

/// A generated implementation plan.
#[derive(Debug, Clone)]
pub struct Plan {
    pub name: String,
    pub content: String,
    pub file_path: PathBuf,
    pub created_at: DateTime<Utc>,
}

/// Central application state shared across the session lifetime.
pub struct AppState {
    pub model: String,
    pub permission_mode: PermissionMode,
    pub verbose: bool,
    pub cwd: PathBuf,
    pub session_id: Uuid,
    pub settings: Settings,
    pub total_input_tokens: u64,
    pub total_output_tokens: u64,
    pub total_cost_usd: f64,
    pub tasks: Vec<Task>,
    pub start_time: Instant,
    pub plan_mode: PlanMode,
    pub current_plan: Option<Plan>,
    pub saved_system_prompt: Option<String>,
}

/// A tracked task within the session.
pub struct Task {
    pub id: String,
    pub title: String,
    pub description: String,
    pub status: TaskStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Status of a tracked task.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
}

impl AppState {
    /// Creates a new `AppState` from CLI arguments and loaded settings.
    pub fn new(args: &CliArgs, settings: Settings) -> Self {
        let cwd = args
            .directory
            .clone()
            .unwrap_or_else(|| std::env::current_dir().expect("failed to determine working directory"));

        let model = args.model.clone();

        let verbose = args.verbose || settings.verbose.unwrap_or(false);

        let permission_mode = args.permission_mode;

        Self {
            model,
            permission_mode,
            verbose,
            cwd,
            session_id: Uuid::new_v4(),
            settings,
            total_input_tokens: 0,
            total_output_tokens: 0,
            total_cost_usd: 0.0,
            tasks: Vec::new(),
            start_time: Instant::now(),
            plan_mode: PlanMode::Off,
            current_plan: None,
            saved_system_prompt: None,
        }
    }
}
