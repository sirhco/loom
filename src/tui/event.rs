use crossterm::event::Event;
use tokio::sync::mpsc;

use crate::engine::query_engine::QueryEngine;

/// Application-level events consumed by the main loop.
pub enum AppEvent {
    /// Terminal input event from crossterm.
    Terminal(Event),
    /// AI query completed successfully.
    QueryComplete {
        response: String,
        engine: QueryEngine,
    },
    /// AI query failed.
    QueryError {
        error: String,
        engine: QueryEngine,
    },
    /// A tool call has started executing.
    ToolCallStarted {
        name: String,
        args_preview: String,
    },
    /// A tool call has completed.
    ToolCallCompleted {
        name: String,
        result_preview: String,
        is_error: bool,
    },
    /// Progress update: which turn of the multi-turn loop we're on.
    TurnProgress {
        turn: usize,
        max_turns: usize,
    },
    /// Tick for spinner animation.
    Tick,
}

/// Spawns a blocking task that reads crossterm terminal events and forwards them
/// to the provided channel. The task exits when the receiver is dropped.
pub fn spawn_event_reader(tx: mpsc::UnboundedSender<AppEvent>) {
    tokio::task::spawn_blocking(move || loop {
        if crossterm::event::poll(std::time::Duration::from_millis(50)).unwrap_or(false) {
            if let Ok(event) = crossterm::event::read() {
                if tx.send(AppEvent::Terminal(event)).is_err() {
                    break;
                }
            }
        }
    });
}

/// Spawns an async task that sends `AppEvent::Tick` at the given interval.
/// The task exits when the receiver is dropped.
pub fn spawn_tick(tx: mpsc::UnboundedSender<AppEvent>, interval_ms: u64) {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_millis(interval_ms));
        loop {
            interval.tick().await;
            if tx.send(AppEvent::Tick).is_err() {
                break;
            }
        }
    });
}
