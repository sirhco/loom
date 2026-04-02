use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};
use directories::BaseDirs;

/// Returns the base configuration directory: `~/.loom/`
pub fn config_dir() -> PathBuf {
    let base = BaseDirs::new().expect("failed to resolve home directory");
    base.home_dir().join(".loom")
}

/// Returns the path to the settings file: `~/.loom/settings.toml`
pub fn settings_path() -> PathBuf {
    config_dir().join("settings.toml")
}

/// Returns the sessions directory: `~/.loom/sessions/`
pub fn sessions_dir() -> PathBuf {
    config_dir().join("sessions")
}

/// Returns the global memory file path: `~/.loom/LOOM.md`
pub fn memory_path() -> PathBuf {
    config_dir().join("LOOM.md")
}

/// Returns the plans directory: `~/.loom/plans/`
pub fn plans_dir() -> PathBuf {
    config_dir().join("plans")
}

/// Creates all required directories if they don't already exist.
pub fn ensure_dirs() -> Result<()> {
    let dirs = [config_dir(), sessions_dir(), plans_dir()];
    for dir in &dirs {
        fs::create_dir_all(dir)
            .with_context(|| format!("failed to create directory: {}", dir.display()))?;
    }
    Ok(())
}
