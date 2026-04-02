use std::fs;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::config::paths;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Settings {
    pub model: Option<String>,
    pub project_id: Option<String>,
    pub location: Option<String>,
    pub verbose: Option<bool>,
    pub permission_mode: Option<String>,
    #[serde(default)]
    pub permissions: PermissionSettings,
    #[serde(default)]
    pub theme: ThemeSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PermissionSettings {
    pub allow_read: Option<Vec<String>>,
    pub deny_read: Option<Vec<String>>,
    pub allow_write: Option<Vec<String>>,
    pub deny_write: Option<Vec<String>>,
    pub allow_run: Option<Vec<String>>,
    pub deny_run: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ThemeSettings {
    pub color_scheme: Option<String>,
}

impl Settings {
    /// Loads settings from `~/.loom/settings.toml`.
    /// Returns default settings if the file does not exist.
    pub fn load() -> Result<Settings> {
        let path = paths::settings_path();
        if !path.exists() {
            return Ok(Settings::default());
        }
        let contents = fs::read_to_string(&path)
            .with_context(|| format!("failed to read settings file: {}", path.display()))?;
        let settings: Settings = toml::from_str(&contents)
            .with_context(|| format!("failed to parse settings file: {}", path.display()))?;
        Ok(settings)
    }

    /// Saves the current settings to `~/.loom/settings.toml`.
    pub fn save(&self) -> Result<()> {
        paths::ensure_dirs()?;
        let path = paths::settings_path();
        let contents = toml::to_string_pretty(self)
            .context("failed to serialize settings to TOML")?;
        fs::write(&path, contents)
            .with_context(|| format!("failed to write settings file: {}", path.display()))?;
        Ok(())
    }
}
