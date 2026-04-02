use std::fs;

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::config::paths;
use crate::engine::messages::Message;

/// Metadata about a saved session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionInfo {
    pub id: Uuid,
    pub created_at: DateTime<Utc>,
    pub last_message_preview: String,
}

/// Serializable session envelope.
#[derive(Debug, Serialize, Deserialize)]
struct SessionFile {
    id: Uuid,
    created_at: DateTime<Utc>,
    messages: Vec<Message>,
}

/// Saves a session's messages to `~/.loom/sessions/<id>.json`.
pub fn save_session(session_id: &Uuid, messages: &[Message]) -> Result<()> {
    paths::ensure_dirs()?;
    let path = paths::sessions_dir().join(format!("{session_id}.json"));

    let envelope = SessionFile {
        id: *session_id,
        created_at: Utc::now(),
        messages: messages.to_vec(),
    };

    let json = serde_json::to_string_pretty(&envelope)
        .context("failed to serialize session")?;
    fs::write(&path, json)
        .with_context(|| format!("failed to write session file: {}", path.display()))?;
    Ok(())
}

/// Loads a previously saved session by its ID.
pub fn load_session(session_id: &Uuid) -> Result<Vec<Message>> {
    let path = paths::sessions_dir().join(format!("{session_id}.json"));
    let contents = fs::read_to_string(&path)
        .with_context(|| format!("failed to read session file: {}", path.display()))?;
    let envelope: SessionFile = serde_json::from_str(&contents)
        .with_context(|| format!("failed to parse session file: {}", path.display()))?;
    Ok(envelope.messages)
}

/// Lists all saved sessions with metadata, ordered by filename.
pub fn list_sessions() -> Result<Vec<SessionInfo>> {
    let dir = paths::sessions_dir();
    if !dir.exists() {
        return Ok(Vec::new());
    }

    let mut sessions = Vec::new();
    let entries = fs::read_dir(&dir)
        .with_context(|| format!("failed to read sessions directory: {}", dir.display()))?;

    for entry in entries {
        let entry = entry.context("failed to read directory entry")?;
        let path = entry.path();

        if path.extension().is_some_and(|ext| ext == "json") {
            if let Ok(contents) = fs::read_to_string(&path) {
                if let Ok(envelope) = serde_json::from_str::<SessionFile>(&contents) {
                    let preview = envelope
                        .messages
                        .last()
                        .and_then(|m| m.text_content().map(|s| s.to_string()))
                        .map(|text| {
                            if text.len() > 100 {
                                format!("{}...", &text[..100])
                            } else {
                                text
                            }
                        })
                        .unwrap_or_default();

                    sessions.push(SessionInfo {
                        id: envelope.id,
                        created_at: envelope.created_at,
                        last_message_preview: preview,
                    });
                }
            }
        }
    }

    sessions.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    Ok(sessions)
}
