pub mod clear;
pub mod commit;
pub mod compact;
pub mod config;
pub mod cost;
pub mod help;
pub mod memory;
pub mod model;
pub mod plan;
pub mod session;

use std::sync::Arc;

use anyhow::Result;
use tokio::sync::RwLock;

use crate::engine::query_engine::QueryEngine;
use crate::state::app_state::AppState;

/// Metadata for a slash command.
pub struct SlashCommand {
    pub name: &'static str,
    pub description: &'static str,
    pub usage: &'static str,
}

/// Result of executing a slash command.
pub enum CommandResult {
    /// Continue the REPL loop.
    Continue,
    /// Exit the REPL.
    Exit,
    /// Display a message and continue.
    Message(String),
}

/// Returns the list of all available slash commands.
pub fn all_commands() -> Vec<SlashCommand> {
    vec![
        SlashCommand {
            name: "help",
            description: "Show available commands",
            usage: "/help",
        },
        SlashCommand {
            name: "clear",
            description: "Clear conversation history",
            usage: "/clear",
        },
        SlashCommand {
            name: "compact",
            description: "Force context compaction",
            usage: "/compact",
        },
        SlashCommand {
            name: "config",
            description: "Display current configuration",
            usage: "/config",
        },
        SlashCommand {
            name: "cost",
            description: "Show token usage and cost summary",
            usage: "/cost",
        },
        SlashCommand {
            name: "model",
            description: "Show or switch the current model",
            usage: "/model [model-name]",
        },
        SlashCommand {
            name: "memory",
            description: "Display loaded memory files",
            usage: "/memory",
        },
        SlashCommand {
            name: "commit",
            description: "Generate a git commit with AI message",
            usage: "/commit",
        },
        SlashCommand {
            name: "session",
            description: "Session management (list, resume)",
            usage: "/session [list|resume <id>]",
        },
        SlashCommand {
            name: "plan",
            description: "Enter planning mode for structured implementation",
            usage: "/plan [start|done|cancel|show|list]",
        },
        SlashCommand {
            name: "exit",
            description: "Exit loom",
            usage: "/exit",
        },
        SlashCommand {
            name: "quit",
            description: "Exit loom",
            usage: "/quit",
        },
    ]
}

/// Parses and dispatches a slash command.
pub async fn dispatch_command(
    input: &str,
    engine: &mut QueryEngine,
    state: &Arc<RwLock<AppState>>,
) -> Result<CommandResult> {
    let trimmed = input.trim_start_matches('/');
    let mut parts = trimmed.splitn(2, ' ');
    let cmd = parts.next().unwrap_or("");
    let args = parts.next().unwrap_or("").trim();

    match cmd {
        "help" | "h" | "?" => Ok(CommandResult::Message(help::execute())),
        "clear" => Ok(CommandResult::Message(clear::execute(engine))),
        "compact" => {
            let msg = compact::execute(engine).await;
            Ok(CommandResult::Message(msg))
        }
        "config" => {
            let s = state.read().await;
            Ok(CommandResult::Message(config::execute(&s)))
        }
        "cost" => Ok(CommandResult::Message(cost::execute(engine))),
        "model" => {
            let mut s = state.write().await;
            Ok(CommandResult::Message(model::execute(args, &mut s, engine)))
        }
        "memory" => {
            let cwd = {
                let s = state.read().await;
                s.cwd.clone()
            };
            Ok(CommandResult::Message(memory::execute(&cwd)))
        }
        "commit" => {
            let cwd = {
                let s = state.read().await;
                s.cwd.clone()
            };
            let msg = commit::execute(&cwd).await;
            Ok(CommandResult::Message(msg))
        }
        "session" => Ok(CommandResult::Message(session::execute(args))),
        "plan" => plan::execute(args, engine, state).await,
        "exit" | "quit" => Ok(CommandResult::Exit),
        _ => Ok(CommandResult::Message(format!(
            "Unknown command: /{cmd}. Type /help for available commands."
        ))),
    }
}
