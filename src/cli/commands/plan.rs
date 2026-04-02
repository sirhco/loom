use std::sync::Arc;

use anyhow::Result;
use tokio::sync::RwLock;

use crate::cli::commands::CommandResult;
use crate::config::paths;
use crate::engine::context::build_planning_prompt;
use crate::engine::query_engine::QueryEngine;
use crate::state::app_state::{AppState, Plan, PlanMode};
use crate::tui::renderer;

/// Subcommands for `/plan`.
enum PlanSubcommand {
    Start,
    Done,
    Cancel,
    Show,
    List,
}

fn parse_subcommand(args: &str) -> PlanSubcommand {
    match args.trim().to_lowercase().as_str() {
        "" | "start" => PlanSubcommand::Start,
        "done" | "approve" | "execute" => PlanSubcommand::Done,
        "cancel" | "abort" => PlanSubcommand::Cancel,
        "show" | "view" => PlanSubcommand::Show,
        "list" | "ls" => PlanSubcommand::List,
        _ => PlanSubcommand::Start,
    }
}

/// Executes the `/plan` command.
pub async fn execute(
    args: &str,
    engine: &mut QueryEngine,
    state: &Arc<RwLock<AppState>>,
) -> Result<CommandResult> {
    let subcmd = parse_subcommand(args);

    match subcmd {
        PlanSubcommand::Start => start_planning(engine, state).await,
        PlanSubcommand::Done => execute_plan(engine, state).await,
        PlanSubcommand::Cancel => cancel_plan(engine, state).await,
        PlanSubcommand::Show => show_plan(state).await,
        PlanSubcommand::List => list_plans(),
    }
}

async fn start_planning(
    engine: &mut QueryEngine,
    state: &Arc<RwLock<AppState>>,
) -> Result<CommandResult> {
    let (cwd, model, current_mode) = {
        let s = state.read().await;
        (s.cwd.clone(), s.model.clone(), s.plan_mode.clone())
    };

    if current_mode != PlanMode::Off {
        return Ok(CommandResult::Message(
            "Already in planning mode. Continue the conversation, or:\n  \
             /plan done    — execute the current plan\n  \
             /plan show    — review the current plan\n  \
             /plan cancel  — exit planning mode".to_string(),
        ));
    }

    // Save current system prompt for restoration
    let saved_prompt = engine.system_prompt().to_string();
    {
        let mut s = state.write().await;
        s.plan_mode = PlanMode::Researching;
        s.saved_system_prompt = Some(saved_prompt);
    }

    // Swap to planning system prompt
    let planning_prompt = build_planning_prompt(&cwd, &model).await;
    engine.set_system_prompt(planning_prompt);

    Ok(CommandResult::Message(
        "Planning mode active. Describe what you want to build.\n\
         The AI will research the codebase and produce an implementation plan.\n\
         Use /plan cancel to exit planning mode."
            .to_string(),
    ))
}

async fn execute_plan(
    _engine: &mut QueryEngine,
    state: &Arc<RwLock<AppState>>,
) -> Result<CommandResult> {
    let plan_mode = {
        let s = state.read().await;
        s.plan_mode.clone()
    };

    if plan_mode == PlanMode::Off {
        return Ok(CommandResult::Message(
            "No plan to execute. Use /plan to start planning.".to_string(),
        ));
    }

    // Check if there's actually a plan saved
    let has_plan = {
        let s = state.read().await;
        s.current_plan.is_some()
    };
    if !has_plan {
        return Ok(CommandResult::Message(
            "No plan content yet. Continue the conversation to generate a plan first.".to_string(),
        ));
    }

    // Signal that execution should happen — the TUI layer (app.rs) will
    // handle this by spawning the execution as a background task.
    // Return a message that triggers the plan action popup.
    Ok(CommandResult::Message(
        "__PLAN_EXECUTE__".to_string(),
    ))
}

async fn cancel_plan(
    engine: &mut QueryEngine,
    state: &Arc<RwLock<AppState>>,
) -> Result<CommandResult> {
    let saved_prompt = {
        let s = state.read().await;
        if s.plan_mode == PlanMode::Off {
            return Ok(CommandResult::Message(
                "Not in planning mode.".to_string(),
            ));
        }
        s.saved_system_prompt.clone()
    };

    // Restore system prompt
    if let Some(prompt) = saved_prompt {
        engine.set_system_prompt(prompt);
    }

    // Reset state
    {
        let mut s = state.write().await;
        s.plan_mode = PlanMode::Off;
        s.current_plan = None;
        s.saved_system_prompt = None;
    }

    Ok(CommandResult::Message(
        "Planning cancelled. Returned to normal mode.".to_string(),
    ))
}

async fn show_plan(state: &Arc<RwLock<AppState>>) -> Result<CommandResult> {
    let s = state.read().await;
    match &s.current_plan {
        Some(plan) => {
            renderer::render_plan(&plan.content);
            Ok(CommandResult::Continue)
        }
        None => Ok(CommandResult::Message(
            "No plan available. Use /plan to start planning.".to_string(),
        )),
    }
}

fn list_plans() -> Result<CommandResult> {
    let plans_dir = paths::plans_dir();
    if !plans_dir.exists() {
        return Ok(CommandResult::Message("No saved plans.".to_string()));
    }

    let mut entries: Vec<_> = std::fs::read_dir(&plans_dir)?
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path()
                .extension()
                .is_some_and(|ext| ext == "md")
        })
        .collect();

    if entries.is_empty() {
        return Ok(CommandResult::Message("No saved plans.".to_string()));
    }

    entries.sort_by_key(|e| std::cmp::Reverse(e.metadata().ok().and_then(|m| m.modified().ok())));

    let mut output = String::from("Saved plans:\n");
    for entry in entries.iter().take(20) {
        let name = entry.file_name().to_string_lossy().to_string();
        output.push_str(&format!("  {name}\n"));
    }

    Ok(CommandResult::Message(output))
}

/// Saves a plan to disk and returns the Plan struct.
pub fn save_plan(name: &str, content: &str) -> Result<Plan> {
    let plans_dir = paths::plans_dir();
    std::fs::create_dir_all(&plans_dir)?;

    let timestamp = chrono::Utc::now().format("%Y%m%d-%H%M%S");
    let slug: String = name
        .chars()
        .map(|c| if c.is_alphanumeric() { c.to_ascii_lowercase() } else { '-' })
        .take(30)
        .collect();
    let filename = format!("{timestamp}-{slug}.md");
    let file_path = plans_dir.join(&filename);

    std::fs::write(&file_path, content)?;

    Ok(Plan {
        name: name.to_string(),
        content: content.to_string(),
        file_path,
        created_at: chrono::Utc::now(),
    })
}
