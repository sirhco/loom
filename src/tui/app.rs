use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Result;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use tokio::sync::{mpsc, RwLock};
use tui_textarea::TextArea;

use crate::cli::commands::plan::save_plan;
use crate::cli::commands::{all_commands, dispatch_command, CommandResult};
use crate::engine::query_engine::QueryEngine;
use crate::state::app_state::{AppState, PlanMode};
use crate::tools::registry::ToolRegistry;
use crate::tui::completion::{build_file_index, filter_slash_commands, fuzzy_match_files};
use crate::tui::event::{spawn_event_reader, spawn_tick, AppEvent};
use crate::tui::file_refs::expand_file_references;
use crate::tui::input::create_textarea;
use crate::tui::message_buffer::{DisplayMessage, MessageBuffer};
use crate::tui::renderer;
use crate::tui::widgets::completion_popup::{CompletionState, CompletionTrigger};
use crate::tui::widgets::spinner_line::SpinnerState;

/// The current interaction mode of the TUI.
pub enum AppMode {
    /// User can type in the textarea.
    Input,
    /// A query is running; input is disabled.
    Processing,
    /// User is scrolling through message history.
    Scrolling,
}

/// The main TUI application state.
pub struct TuiApp {
    pub mode: AppMode,
    pub state: Arc<RwLock<AppState>>,
    pub messages: MessageBuffer,
    pub textarea: TextArea<'static>,
    pub completion: Option<CompletionState>,
    pub spinner: SpinnerState,
    pub should_quit: bool,
    pub model_name: String,
    pub provider_name: String,
    pub total_tokens: u64,
    pub total_cost: f64,
    pub plan_mode: PlanMode,
    pub file_index: Vec<String>,
    pub commands: Vec<String>,
    pub cwd: PathBuf,
    event_tx: mpsc::UnboundedSender<AppEvent>,
    /// Engine slot: `Some` when idle, `None` when a query is in flight.
    engine: Option<QueryEngine>,
    /// Set to true when user presses Ctrl+C during Processing.
    /// The next QueryComplete/QueryError will discard the response.
    cancel_requested: bool,
}

/// Runs the interactive TUI loop.
///
/// Takes the `QueryEngine` by value. When a query is spawned to a background
/// task the engine is moved out of `TuiApp`; on completion it is moved back.
pub async fn run_interactive(
    engine: QueryEngine,
    state: Arc<RwLock<AppState>>,
    provider_name: &str,
) -> Result<()> {
    // Install a panic hook that restores the terminal before printing the panic.
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        ratatui::restore();
        original_hook(info);
    }));

    // Enable bracketed paste so multi-line pastes arrive as a single Event::Paste
    crossterm::execute!(
        std::io::stdout(),
        crossterm::event::EnableBracketedPaste
    ).ok();

    let mut terminal = ratatui::init();

    // Event channel
    let (event_tx, mut event_rx) = mpsc::unbounded_channel();

    // Start event readers
    spawn_event_reader(event_tx.clone());
    spawn_tick(event_tx.clone(), 80);

    // Read initial state
    let (cwd, model, plan_mode) = {
        let s = state.read().await;
        (s.cwd.clone(), s.model.clone(), s.plan_mode.clone())
    };

    let commands: Vec<String> = all_commands()
        .iter()
        .map(|c| format!("/{}", c.name))
        .collect();
    let file_index = build_file_index(&cwd);

    // Set the progress event sender on the engine so tool calls emit events
    let mut engine = engine;
    engine.set_progress_tx(event_tx.clone());

    let mut app = TuiApp {
        mode: AppMode::Input,
        state: state.clone(),
        messages: MessageBuffer::new(),
        textarea: create_textarea(),
        completion: None,
        spinner: SpinnerState::new(),
        should_quit: false,
        model_name: model.clone(),
        provider_name: provider_name.to_string(),
        total_tokens: 0,
        total_cost: 0.0,
        plan_mode,
        file_index,
        commands,
        cwd,
        event_tx: event_tx.clone(),
        engine: Some(engine),
        cancel_requested: false,
    };

    // Welcome banner with ASCII art logo
    let terminal_width = terminal.size().map(|s| s.width).unwrap_or(80);
    let version = env!("CARGO_PKG_VERSION");
    let cwd_short = app.cwd.file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| app.cwd.display().to_string());
    app.messages.push(
        DisplayMessage::System(format!(
r#"
       __                         /   /\
      /  /\                      /   /  \
     /  /  \       ____         /   / /\ \      ____
    /  / /\ \     /    \       /   / /  \ \    /    \
   /  / /  \ \   /  /\  \     /   / /    \ \  /  /\  \
  /  /_/____\ \ /  /  \  \   /   / /      \ \/  /  \  \
 /____________//__/____\__\ /___/_/        \__/__/____\__\

   L            O            O             M    v{version}
 _________________________________________________________
|                                                         |
|              >> DON'T LOSE THE THREAD. <<               |
|_________________________________________________________|

   {provider_name} · {model}
   {cwd_short}

   /help for commands    /plan for planning mode
   /model to switch      Ctrl+D to quit"#,
            provider_name = app.provider_name,
            model = app.model_name,
        )),
        terminal_width,
    );

    // Main event loop
    loop {
        terminal.draw(|frame| renderer::draw(frame, &app))?;

        match event_rx.recv().await {
            Some(AppEvent::Terminal(event)) => {
                handle_terminal_event(&mut app, event).await;
            }
            Some(AppEvent::QueryComplete { response, engine }) => {
                handle_query_complete(&mut app, response, engine, &terminal);
            }
            Some(AppEvent::QueryError { error, engine }) => {
                handle_query_error(&mut app, error, engine, &terminal);
            }
            Some(AppEvent::ToolCallStarted { name, args_preview }) => {
                app.spinner.update_message(&format!("running {name}..."));
                let width = terminal.size().map(|s| s.width).unwrap_or(80);
                app.messages.push(
                    DisplayMessage::ToolCall { name: name.clone(), args: args_preview },
                    width,
                );
            }
            Some(AppEvent::ToolCallCompleted { name, result_preview, is_error }) => {
                app.spinner.update_message("thinking...");
                let width = terminal.size().map(|s| s.width).unwrap_or(80);
                app.messages.push(
                    DisplayMessage::ToolResult {
                        name,
                        content: result_preview,
                        is_error,
                    },
                    width,
                );
            }
            Some(AppEvent::TurnProgress { turn, max_turns }) => {
                app.spinner.set_detail(&format!("turn {}/{}", turn, max_turns));
            }
            Some(AppEvent::Tick) => {
                app.spinner.tick();
            }
            None => break,
        }

        if app.should_quit {
            break;
        }
    }

    crossterm::execute!(
        std::io::stdout(),
        crossterm::event::DisableBracketedPaste
    ).ok();
    ratatui::restore();
    Ok(())
}

/// Routes terminal events based on the current app mode.
async fn handle_terminal_event(app: &mut TuiApp, event: Event) {
    match event {
        Event::Key(key) => {
            handle_key_event(app, key).await;
        }
        Event::Paste(text) => {
            // Bracketed paste: insert all pasted text into the textarea at once
            if matches!(app.mode, AppMode::Input) {
                for ch in text.chars() {
                    if ch == '\n' {
                        // Pasted newlines become actual newlines in the textarea
                        app.textarea.input(tui_textarea::Input {
                            key: tui_textarea::Key::Enter,
                            ctrl: false,
                            alt: false,
                            shift: false,
                        });
                    } else {
                        app.textarea.input(tui_textarea::Input {
                            key: tui_textarea::Key::Char(ch),
                            ctrl: false,
                            alt: false,
                            shift: false,
                        });
                    }
                }
            }
        }
        Event::Resize(w, _h) => {
            app.messages.re_render(w);
        }
        _ => {}
    }
}

/// Handles a single key event.
async fn handle_key_event(app: &mut TuiApp, key: KeyEvent) {
    // Ctrl+D: quit
    if key.code == KeyCode::Char('d') && key.modifiers.contains(KeyModifiers::CONTROL) {
        app.should_quit = true;
        return;
    }
    // Ctrl+C: cancel current operation
    if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
        match app.mode {
            AppMode::Processing => {
                // Cancel the in-flight query. We can't abort the task, but we
                // flag it so the result is discarded when it arrives.
                app.cancel_requested = true;
                app.spinner.stop();
                app.mode = AppMode::Input;
                app.messages.push(
                    DisplayMessage::System("Request cancelled. Waiting for engine to return...".to_string()),
                    80,
                );
            }
            _ => {
                app.textarea = create_textarea();
                app.completion = None;
                app.mode = AppMode::Input;
            }
        }
        return;
    }

    match app.mode {
        AppMode::Processing => {
            // Allow scrolling while processing, block everything else
            match key.code {
                KeyCode::PageUp => app.messages.scroll_up(10),
                KeyCode::PageDown => app.messages.scroll_down(10),
                KeyCode::Up if key.modifiers.contains(KeyModifiers::SHIFT) => {
                    app.messages.scroll_up(1);
                }
                KeyCode::Down if key.modifiers.contains(KeyModifiers::SHIFT) => {
                    app.messages.scroll_down(1);
                }
                KeyCode::End => app.messages.scroll_to_bottom(),
                _ => {}
            }
        }
        AppMode::Scrolling => {
            handle_scrolling_key(app, key);
        }
        AppMode::Input => {
            handle_input_key(app, key).await;
        }
    }
}

/// Handles keys in scrolling mode.
fn handle_scrolling_key(app: &mut TuiApp, key: KeyEvent) {
    match key.code {
        KeyCode::Up | KeyCode::Char('k') => {
            app.messages.scroll_up(1);
        }
        KeyCode::Down | KeyCode::Char('j') => {
            app.messages.scroll_down(1);
        }
        KeyCode::PageUp => {
            app.messages.scroll_up(10);
        }
        KeyCode::PageDown => {
            app.messages.scroll_down(10);
        }
        KeyCode::Esc | KeyCode::Char('q') => {
            app.messages.scroll_to_bottom();
            app.mode = AppMode::Input;
        }
        _ => {}
    }
}

/// Handles keys in input mode.
async fn handle_input_key(app: &mut TuiApp, key: KeyEvent) {
    // If completion popup is active, handle navigation keys first
    if app.completion.is_some() {
        match key.code {
            KeyCode::Up => {
                if let Some(ref mut completion) = app.completion {
                    completion.move_up();
                }
                return;
            }
            KeyCode::Down => {
                if let Some(ref mut completion) = app.completion {
                    completion.move_down();
                }
                return;
            }
            KeyCode::Tab => {
                accept_completion(app);
                return;
            }
            KeyCode::Enter => {
                // For Model/PlanAction popups, Enter accepts (they're choosers, not autocomplete)
                // For Slash/AtFile, Enter submits the text as-is (Tab is for accepting)
                let is_chooser = app.completion.as_ref()
                    .is_some_and(|c| matches!(c.trigger, CompletionTrigger::Model | CompletionTrigger::PlanAction));
                if is_chooser {
                    accept_completion(app);
                } else {
                    app.completion = None;
                    submit_input(app).await;
                }
                return;
            }
            KeyCode::Esc => {
                app.completion = None;
                return;
            }
            _ => {
                // Close completion and fall through to normal input handling
                app.completion = None;
            }
        }
    }

    // Enter without shift: submit input
    if key.code == KeyCode::Enter && !key.modifiers.contains(KeyModifiers::SHIFT) {
        submit_input(app).await;
        return;
    }

    // PageUp: enter scrolling mode
    if key.code == KeyCode::PageUp {
        app.mode = AppMode::Scrolling;
        app.messages.scroll_up(10);
        return;
    }

    // Pass the key event to the textarea
    app.textarea.input(crossterm::event::Event::Key(key));

    // After input, check if we should trigger completion
    update_completion(app);
}

/// Accepts the currently selected completion item and inserts it.
fn accept_completion(app: &mut TuiApp) {
    let selected = app
        .completion
        .as_ref()
        .and_then(|c| c.selected_value().map(|s| s.to_string()));
    let trigger = app
        .completion
        .as_ref()
        .map(|c| c.trigger.clone());

    if let (Some(value), Some(trigger)) = (selected, trigger) {
        match trigger {
            CompletionTrigger::PlanAction => {
                // Plan action selection
                app.completion = None;
                if value.starts_with("Execute") {
                    spawn_plan_execution(app);
                } else if value.starts_with("Edit") {
                    // Let the user type changes — pre-fill the textarea with a prompt
                    app.textarea = create_textarea();
                    let hint = "Please make these changes to the plan: ";
                    for ch in hint.chars() {
                        app.textarea.input(tui_textarea::Input {
                            key: tui_textarea::Key::Char(ch),
                            ctrl: false,
                            alt: false,
                            shift: false,
                        });
                    }
                    app.messages.push(
                        DisplayMessage::System(
                            "Type your changes and press Enter. The AI will revise the plan."
                                .to_string(),
                        ),
                        80,
                    );
                    // Stay in Researching mode so the input goes back to the AI
                } else if value.starts_with("Show") {
                    let plan_text = app.state.try_read()
                        .ok()
                        .and_then(|s| s.current_plan.as_ref().map(|p| p.content.clone()))
                        .unwrap_or_else(|| "No plan available.".to_string());
                    app.messages.push(DisplayMessage::System(plan_text), 80);
                    // Re-show the popup so user can choose next action
                    show_plan_action_popup(app);
                } else if value.starts_with("Cancel") {
                    // Cancel the plan
                    if let Some(ref mut engine) = app.engine {
                        if let Ok(s) = app.state.try_read() {
                            if let Some(ref prompt) = s.saved_system_prompt {
                                engine.set_system_prompt(prompt.clone());
                            }
                        }
                    }
                    if let Ok(mut s) = app.state.try_write() {
                        s.plan_mode = PlanMode::Off;
                        s.current_plan = None;
                        s.saved_system_prompt = None;
                    }
                    app.plan_mode = PlanMode::Off;
                    app.messages.push(
                        DisplayMessage::System("Plan cancelled. Returned to normal mode.".to_string()),
                        80,
                    );
                }
                return;
            }
            CompletionTrigger::Model => {
                // Model selection — don't insert text, switch the model
                // The value is a selector label; extract the model ID from it
                let model_id = if let Some(gm) = crate::provider::models::GeminiModel::from_selector_label(&value) {
                    gm.model_id().to_string()
                } else {
                    value.clone()
                };
                if let Some(ref mut engine) = app.engine {
                    engine.set_model(&model_id);
                }
                {
                    let state = app.state.clone();
                    if let Ok(mut s) = state.try_write() {
                        s.model = model_id.clone();
                    }
                }
                app.model_name = model_id.clone();
                let msg = format!("Switched to model: {model_id}");
                app.messages.push(DisplayMessage::System(msg), 80);
                app.completion = None;
                return;
            }
            _ => {
                // Get the current text before replacing
                let current_text = app.textarea.lines().join("\n");

                let new_text = match trigger {
                    CompletionTrigger::Slash => {
                        // Replace the entire input (slash commands are always at the start)
                        format!("{value} ")
                    }
                    CompletionTrigger::AtFile => {
                        // Replace just the @partial with @selected, keeping text before the @
                        if let Some(at_pos) = current_text.rfind('@') {
                            let before = &current_text[..at_pos];
                            format!("{before}@{value} ")
                        } else {
                            format!("@{value} ")
                        }
                    }
                    CompletionTrigger::Model | CompletionTrigger::PlanAction => unreachable!(),
                };

                // Reset textarea and type in the new text
                app.textarea = create_textarea();
                for ch in new_text.chars() {
                    app.textarea.input(tui_textarea::Input {
                        key: tui_textarea::Key::Char(ch),
                        ctrl: false,
                        alt: false,
                        shift: false,
                    });
                }
            }
        }
    }
    app.completion = None;
}

/// Checks the current textarea content and triggers completion if appropriate.
fn update_completion(app: &mut TuiApp) {
    let lines = app.textarea.lines();
    if lines.is_empty() {
        app.completion = None;
        return;
    }

    let text: String = lines.join("\n");
    let trimmed = text.trim_start();

    // Slash command completion: input starts with "/" and has no spaces
    if trimmed.starts_with('/') && !trimmed.contains(' ') {
        let candidates = filter_slash_commands(&app.commands, trimmed);
        if !candidates.is_empty() && trimmed.len() > 1 {
            let filter = trimmed[1..].to_string();
            app.completion = Some(CompletionState {
                trigger: CompletionTrigger::Slash,
                filter,
                candidates,
                selected: 0,
            });
        } else {
            app.completion = None;
        }
        return;
    }

    // @ file completion
    if let Some(at_pos) = text.rfind('@') {
        let is_valid_trigger = at_pos == 0
            || text.as_bytes().get(at_pos - 1) == Some(&b' ')
            || text.as_bytes().get(at_pos - 1) == Some(&b'\n');
        if is_valid_trigger {
            let partial = &text[at_pos + 1..];
            if !partial.contains(' ') && !partial.is_empty() {
                let candidates = fuzzy_match_files(&app.file_index, partial);
                if !candidates.is_empty() {
                    app.completion = Some(CompletionState {
                        trigger: CompletionTrigger::AtFile,
                        filter: partial.to_string(),
                        candidates,
                        selected: 0,
                    });
                    return;
                }
            }
        }
    }

    app.completion = None;
}

/// Extracts text from the textarea, processes it, and dispatches it.
async fn submit_input(app: &mut TuiApp) {
    let text: String = app.textarea.lines().join("\n");
    let input = text.trim().to_string();

    if input.is_empty() {
        return;
    }

    // Reset the textarea
    app.textarea = create_textarea();
    app.completion = None;

    // Show the user message in the buffer
    app.messages.push(DisplayMessage::User(input.clone()), 80);

    // Handle "exit"/"quit" bare words
    if input == "exit" || input == "quit" {
        app.should_quit = true;
        return;
    }

    // Handle /model with no args as interactive popup
    if input == "/model" {
        let candidates: Vec<String> = crate::provider::models::GeminiModel::all()
            .iter()
            .map(|m| m.selector_label())
            .collect();
        app.completion = Some(CompletionState {
            trigger: CompletionTrigger::Model,
            filter: String::new(),
            candidates,
            selected: 0,
        });
        return;
    }

    // Handle slash commands (synchronous dispatch — engine stays in slot)
    if input.starts_with('/') {
        handle_slash_command(app, &input).await;
        return;
    }

    // Handle shell commands (! prefix)
    if input.starts_with('!') {
        handle_shell_command(app, &input);
        return;
    }

    // Normal query or plan research
    let plan_mode = app.plan_mode.clone();
    match plan_mode {
        PlanMode::Researching => {
            spawn_plan_research(app, &input);
        }
        PlanMode::PlanReady => {
            // If the user typed follow-up text (e.g. a URL, clarification),
            // continue the research conversation with their input.
            // The plan action popup only shows on empty input or /plan commands.
            {
                let mut s_guard = app.state.try_write();
                if let Ok(ref mut s) = s_guard {
                    s.plan_mode = PlanMode::Researching;
                }
            }
            app.plan_mode = PlanMode::Researching;
            spawn_plan_research(app, &input);
        }
        PlanMode::Off => {
            spawn_query(app, &input);
        }
    }
}

/// Dispatches a slash command synchronously.
async fn handle_slash_command(app: &mut TuiApp, input: &str) {
    let engine = match app.engine.as_mut() {
        Some(e) => e,
        None => {
            app.messages.push(
                DisplayMessage::Error("Engine is busy with a query. Please wait.".to_string()),
                80,
            );
            return;
        }
    };

    match dispatch_command(input, engine, &app.state).await {
        Ok(CommandResult::Exit) => {
            app.should_quit = true;
        }
        Ok(CommandResult::Continue) => {
            refresh_state_from_appstate(app).await;
        }
        Ok(CommandResult::Message(msg)) => {
            if msg == "__PLAN_EXECUTE__" {
                // Signal from /plan done — spawn execution as background task
                spawn_plan_execution(app);
            } else {
                app.messages.push(DisplayMessage::System(msg), 80);
                refresh_state_from_appstate(app).await;
            }
        }
        Err(err) => {
            app.messages
                .push(DisplayMessage::Error(format!("{err:#}")), 80);
        }
    }
}

/// Handles a shell command (! prefix).
fn handle_shell_command(app: &mut TuiApp, input: &str) {
    let shell_cmd = input.trim_start_matches('!').trim();
    if shell_cmd.is_empty() {
        app.messages
            .push(DisplayMessage::Error("Empty shell command.".to_string()), 80);
        return;
    }

    match std::process::Command::new("sh")
        .arg("-c")
        .arg(shell_cmd)
        .output()
    {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            let mut result = String::new();
            if !stdout.is_empty() {
                result.push_str(&stdout);
            }
            if !stderr.is_empty() {
                if !result.is_empty() {
                    result.push('\n');
                }
                result.push_str(&stderr);
            }
            if result.is_empty() {
                result = "(no output)".to_string();
            }
            app.messages.push(DisplayMessage::System(result), 80);
        }
        Err(err) => {
            app.messages.push(
                DisplayMessage::Error(format!("Failed to run command: {err}")),
                80,
            );
        }
    }
}

/// Spawns an async query task, taking the engine out of the app.
fn spawn_query(app: &mut TuiApp, input: &str) {
    let engine = match app.engine.take() {
        Some(e) => e,
        None => {
            app.messages.push(
                DisplayMessage::Error("Engine is busy with a query. Please wait.".to_string()),
                80,
            );
            return;
        }
    };

    app.mode = AppMode::Processing;
    app.spinner.start("Thinking...");

    let expanded_input = expand_input(input, &app.cwd, &mut app.messages);
    let tx = app.event_tx.clone();
    let state = app.state.clone();

    tokio::spawn(async move {
        let mut engine = engine;
        match engine.query(&expanded_input).await {
            Ok(response) => {
                update_cost_state(&engine, &state).await;
                let _ = tx.send(AppEvent::QueryComplete { response, engine });
            }
            Err(err) => {
                let _ = tx.send(AppEvent::QueryError {
                    error: format!("{err:#}"),
                    engine,
                });
            }
        }
    });
}

/// Spawns an async plan research task.
fn spawn_plan_research(app: &mut TuiApp, input: &str) {
    let engine = match app.engine.take() {
        Some(e) => e,
        None => {
            app.messages.push(
                DisplayMessage::Error("Engine is busy with a query. Please wait.".to_string()),
                80,
            );
            return;
        }
    };

    app.mode = AppMode::Processing;
    app.spinner.start("Researching...");

    let expanded_input = expand_input(input, &app.cwd, &mut app.messages);
    let tx = app.event_tx.clone();
    let state = app.state.clone();

    tokio::spawn(async move {
        let mut engine = engine;
        let tool_registry = ToolRegistry::new_read_only(state.clone());
        match engine
            .query_with_registry(&expanded_input, tool_registry)
            .await
        {
            Ok(response) => {
                // Save the latest response as a draft plan, but stay in
                // Researching mode so the user can continue the conversation.
                // The user transitions to PlanReady explicitly via /plan done.
                let plan = match save_plan("plan", &response) {
                    Ok(plan) => plan,
                    Err(_) => crate::state::app_state::Plan {
                        name: "plan".to_string(),
                        content: response.clone(),
                        file_path: std::path::PathBuf::new(),
                        created_at: chrono::Utc::now(),
                    },
                };
                {
                    let mut s = state.write().await;
                    s.current_plan = Some(plan);
                    // Stay in Researching — don't auto-transition to PlanReady
                }
                update_cost_state(&engine, &state).await;
                let _ = tx.send(AppEvent::QueryComplete { response, engine });
            }
            Err(err) => {
                let _ = tx.send(AppEvent::QueryError {
                    error: format!("{err:#}"),
                    engine,
                });
            }
        }
    });
}

/// Shows the plan action popup with options for the user to choose.
fn show_plan_action_popup(app: &mut TuiApp) {
    let candidates = vec![
        "Execute plan".to_string(),
        "Edit plan — provide changes".to_string(),
        "Show the plan".to_string(),
        "Cancel plan".to_string(),
    ];
    app.completion = Some(CompletionState {
        trigger: CompletionTrigger::PlanAction,
        filter: String::new(),
        candidates,
        selected: 0,
    });
}

/// Spawns the plan execution as a background task (like spawn_query).
fn spawn_plan_execution(app: &mut TuiApp) {
    // Get the plan content and saved prompt
    let (plan_content, saved_prompt) = {
        match app.state.try_read() {
            Ok(s) => (
                s.current_plan.as_ref().map(|p| p.content.clone()),
                s.saved_system_prompt.clone(),
            ),
            Err(_) => (None, None),
        }
    };

    let plan_content = match plan_content {
        Some(c) => c,
        None => {
            app.messages.push(
                DisplayMessage::Error("No plan content to execute.".to_string()),
                80,
            );
            return;
        }
    };

    let mut engine = match app.engine.take() {
        Some(e) => e,
        None => {
            app.messages.push(
                DisplayMessage::Error("Engine is busy with a query. Please wait.".to_string()),
                80,
            );
            return;
        }
    };

    // Restore the normal system prompt before executing
    if let Some(prompt) = saved_prompt {
        engine.set_system_prompt(prompt);
    }

    // Reset plan mode
    if let Ok(mut s) = app.state.try_write() {
        s.plan_mode = PlanMode::Off;
        s.saved_system_prompt = None;
    }
    app.plan_mode = PlanMode::Off;

    app.mode = AppMode::Processing;
    app.spinner.start("Executing plan...");
    app.messages.push(
        DisplayMessage::System("Executing plan...".to_string()),
        80,
    );

    let execution_prompt = format!(
        "Execute the following implementation plan. Follow each step carefully.\n\n{plan_content}"
    );

    let tx = app.event_tx.clone();
    let state = app.state.clone();

    tokio::spawn(async move {
        match engine.query(&execution_prompt).await {
            Ok(response) => {
                update_cost_state(&engine, &state).await;
                let _ = tx.send(AppEvent::QueryComplete { response, engine });
            }
            Err(err) => {
                let _ = tx.send(AppEvent::QueryError {
                    error: format!("{err:#}"),
                    engine,
                });
            }
        }
    });
}

/// Expands @file references in the input text.
fn expand_input(input: &str, cwd: &PathBuf, messages: &mut MessageBuffer) -> String {
    match expand_file_references(input, cwd) {
        Ok((expanded, refs)) => {
            if !refs.is_empty() {
                messages.push(
                    DisplayMessage::System(format!(
                        "Including {} file(s) as context",
                        refs.len()
                    )),
                    80,
                );
            }
            expanded
        }
        Err(err) => {
            messages.push(
                DisplayMessage::Error(format!("Failed to read file reference: {err}")),
                80,
            );
            input.to_string()
        }
    }
}

/// Updates the cost tracking fields in AppState from the engine's cost tracker.
async fn update_cost_state(engine: &QueryEngine, state: &Arc<RwLock<AppState>>) {
    let tracker = engine.cost_tracker();
    let mut s = state.write().await;
    s.total_input_tokens = tracker.total_input_tokens();
    s.total_output_tokens = tracker.total_output_tokens();
    s.total_cost_usd = tracker.total_cost();
}

/// Handles a successful query completion — puts the engine back.
fn handle_query_complete(
    app: &mut TuiApp,
    response: String,
    engine: QueryEngine,
    terminal: &ratatui::DefaultTerminal,
) {
    app.spinner.stop();
    app.mode = AppMode::Input;
    app.engine = Some(engine);

    // If the user cancelled, discard the response
    if app.cancel_requested {
        app.cancel_requested = false;
        app.messages.push(
            DisplayMessage::System("Cancelled — response discarded.".to_string()),
            80,
        );
        // Still refresh state (cost tracking etc.)
        if let Ok(s) = app.state.try_read() {
            app.total_tokens = s.total_input_tokens + s.total_output_tokens;
            app.total_cost = s.total_cost_usd;
            app.plan_mode = s.plan_mode.clone();
            app.model_name = s.model.clone();
        }
        return;
    }

    let terminal_width = terminal.size().map(|s| s.width).unwrap_or(80);
    app.messages
        .push(DisplayMessage::Assistant(response), terminal_width);

    // Refresh local copies from shared state
    if let Ok(s) = app.state.try_read() {
        app.total_tokens = s.total_input_tokens + s.total_output_tokens;
        app.total_cost = s.total_cost_usd;
        app.plan_mode = s.plan_mode.clone();
        app.model_name = s.model.clone();
    }

    // After plan research completes, auto-show the action popup
    // so the user can execute, edit, show, or cancel
    if app.plan_mode == PlanMode::Researching {
        // Check if a plan was saved (meaning the AI produced something)
        let has_plan = app.state.try_read()
            .ok()
            .and_then(|s| s.current_plan.as_ref().map(|_| true))
            .unwrap_or(false);
        if has_plan {
            show_plan_action_popup(app);
        }
    }
}

/// Handles a query error — puts the engine back.
fn handle_query_error(
    app: &mut TuiApp,
    error: String,
    engine: QueryEngine,
    terminal: &ratatui::DefaultTerminal,
) {
    app.spinner.stop();
    app.mode = AppMode::Input;
    app.engine = Some(engine);

    // If cancelled, suppress the error
    if app.cancel_requested {
        app.cancel_requested = false;
        app.messages.push(
            DisplayMessage::System("Cancelled.".to_string()),
            80,
        );
        return;
    }

    let terminal_width = terminal.size().map(|s| s.width).unwrap_or(80);
    app.messages
        .push(DisplayMessage::Error(error), terminal_width);
}

/// Refreshes local app fields from the shared AppState.
async fn refresh_state_from_appstate(app: &mut TuiApp) {
    let s = app.state.read().await;
    app.total_tokens = s.total_input_tokens + s.total_output_tokens;
    app.total_cost = s.total_cost_usd;
    app.plan_mode = s.plan_mode.clone();
    app.model_name = s.model.clone();
}
