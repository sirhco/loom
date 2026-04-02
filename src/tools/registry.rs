use std::sync::Arc;

use anyhow::Result;
use rig::agent::AgentBuilder;
use rig::completion::CompletionModel;
use rig::tool::ToolDyn;
use tokio::sync::RwLock;

use crate::state::app_state::AppState;

use super::agent::AgentTool;
use super::ask_user::AskUserTool;
use super::bash::BashTool;
use super::file_edit::FileEditTool;
use super::file_read::FileReadTool;
use super::file_write::FileWriteTool;
use super::glob::GlobTool;
use super::grep::GrepTool;
use super::tasks::create::TaskCreateTool;
use super::tasks::get::TaskGetTool;
use super::tasks::list::TaskListTool;
use super::tasks::update::TaskUpdateTool;
use super::web_fetch::WebFetchTool;
use super::web_search::WebSearchTool;

/// Metadata about a registered tool (for display and logging purposes).
#[derive(Debug, Clone)]
pub struct ToolInfo {
    pub name: String,
    pub description: String,
}

/// Registry of tools available to the AI agent.
///
/// Wraps tool instances and provides methods to build a rig agent
/// with all registered tools.
pub struct ToolRegistry {
    tools: Vec<Box<dyn ToolDyn>>,
    tool_infos: Vec<ToolInfo>,
}

impl ToolRegistry {
    /// Creates a new tool registry with all default tools.
    ///
    /// Task tools require shared state for reading/writing tasks.
    pub fn new(state: Arc<RwLock<AppState>>) -> Self {
        let mut registry = Self {
            tools: Vec::new(),
            tool_infos: Vec::new(),
        };

        // File system tools
        registry.register(BashTool, "Execute shell commands");
        registry.register(FileReadTool, "Read file contents");
        registry.register(FileWriteTool, "Write/create files");
        registry.register(FileEditTool, "Search and replace in files");
        registry.register(GlobTool, "Find files by pattern");
        registry.register(GrepTool, "Search file contents");

        // Web tools
        registry.register(WebFetchTool, "Fetch URL content");
        registry.register(WebSearchTool, "Search the web");

        // Interactive tools
        registry.register(AskUserTool, "Ask the user a question");

        // Agent tool
        registry.register(AgentTool, "Spawn a sub-agent");

        // Task tools (with shared state)
        registry.register(
            TaskCreateTool { state: state.clone() },
            "Create a new task",
        );
        registry.register(
            TaskListTool { state: state.clone() },
            "List all tasks",
        );
        registry.register(
            TaskUpdateTool { state: state.clone() },
            "Update task status",
        );
        registry.register(
            TaskGetTool { state: state.clone() },
            "Get task details",
        );

        registry
    }

    /// Creates a read-only tool registry for planning mode.
    ///
    /// Only includes tools that read/search — no write, edit, or agent tools.
    pub fn new_read_only(state: Arc<RwLock<AppState>>) -> Self {
        let _ = state; // State reserved for future read-only task tools
        let mut registry = Self {
            tools: Vec::new(),
            tool_infos: Vec::new(),
        };

        registry.register(BashTool, "Execute shell commands (read-only in plan mode)");
        registry.register(FileReadTool, "Read file contents");
        registry.register(GlobTool, "Find files by pattern");
        registry.register(GrepTool, "Search file contents");
        registry.register(AskUserTool, "Ask the user a question");
        registry.register(WebFetchTool, "Fetch URL content");

        registry
    }

    /// Register a tool with the registry.
    fn register<T: ToolDyn + 'static>(&mut self, tool: T, description: &str) {
        let name = tool.name();
        self.tool_infos.push(ToolInfo {
            name: name.clone(),
            description: description.to_string(),
        });
        self.tools.push(Box::new(tool));
    }

    /// Generates a description string of all registered tools for inclusion in system prompts.
    pub fn describe_all(&self) -> String {
        let mut desc = String::from("Available tools:\n");
        for info in &self.tool_infos {
            desc.push_str(&format!("  - {}: {}\n", info.name, info.description));
        }
        desc
    }

    /// Builds a rig agent with the system prompt and all registered tools.
    pub fn build_agent<M: CompletionModel>(
        self,
        model: M,
        system_prompt: &str,
    ) -> Result<rig::agent::Agent<M>> {
        let agent = AgentBuilder::new(model)
            .preamble(system_prompt)
            .tools(self.tools)
            .build();
        Ok(agent)
    }

    /// Returns the list of registered tool infos.
    pub fn tool_infos(&self) -> &[ToolInfo] {
        &self.tool_infos
    }

    /// Returns the number of registered tools.
    pub fn len(&self) -> usize {
        self.tools.len()
    }

    /// Returns true if no tools are registered.
    pub fn is_empty(&self) -> bool {
        self.tools.is_empty()
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        // Default registry with no state - task tools won't be available.
        Self {
            tools: Vec::new(),
            tool_infos: Vec::new(),
        }
    }
}
