use std::fmt;
use std::path::Path;

use glob::Pattern;

use crate::config::settings::PermissionSettings;

/// The result of checking a permission rule against an operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PermissionDecision {
    /// The operation is explicitly allowed.
    Allow,
    /// The operation is explicitly denied, with a reason.
    Deny(String),
    /// No rule matched; the user should be asked.
    Ask(String),
}

impl fmt::Display for PermissionDecision {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PermissionDecision::Allow => write!(f, "allowed"),
            PermissionDecision::Deny(reason) => write!(f, "denied: {}", reason),
            PermissionDecision::Ask(description) => write!(f, "ask: {}", description),
        }
    }
}

/// A set of glob-based rules that govern read, write, and command execution.
#[derive(Debug, Clone, Default)]
pub struct PermissionRules {
    pub allow_read: Vec<String>,
    pub deny_read: Vec<String>,
    pub allow_write: Vec<String>,
    pub deny_write: Vec<String>,
    pub allow_run: Vec<String>,
    pub deny_run: Vec<String>,
}

impl PermissionRules {
    /// Build permission rules from the user's settings.
    pub fn from_settings(settings: &PermissionSettings) -> Self {
        PermissionRules {
            allow_read: settings.allow_read.clone().unwrap_or_default(),
            deny_read: settings.deny_read.clone().unwrap_or_default(),
            allow_write: settings.allow_write.clone().unwrap_or_default(),
            deny_write: settings.deny_write.clone().unwrap_or_default(),
            allow_run: settings.allow_run.clone().unwrap_or_default(),
            deny_run: settings.deny_run.clone().unwrap_or_default(),
        }
    }

    /// Check whether a file read is allowed, denied, or needs user confirmation.
    pub fn check_read(&self, path: &Path) -> PermissionDecision {
        check_path_rules(path, &self.deny_read, &self.allow_read, "read")
    }

    /// Check whether a file write is allowed, denied, or needs user confirmation.
    pub fn check_write(&self, path: &Path) -> PermissionDecision {
        check_path_rules(path, &self.deny_write, &self.allow_write, "write")
    }

    /// Check whether running a command is allowed, denied, or needs user confirmation.
    pub fn check_run(&self, command: &str) -> PermissionDecision {
        // Check deny rules first (deny takes precedence).
        for pattern_str in &self.deny_run {
            if command_matches(command, pattern_str) {
                return PermissionDecision::Deny(format!(
                    "command '{}' matches deny pattern '{}'",
                    command, pattern_str
                ));
            }
        }

        // Check allow rules.
        for pattern_str in &self.allow_run {
            if command_matches(command, pattern_str) {
                return PermissionDecision::Allow;
            }
        }

        // No rule matched.
        PermissionDecision::Ask(format!("run command: {}", command))
    }
}

/// Check a file path against deny and allow glob patterns.
/// Deny rules take precedence over allow rules. If nothing matches, return Ask.
fn check_path_rules(
    path: &Path,
    deny_patterns: &[String],
    allow_patterns: &[String],
    operation: &str,
) -> PermissionDecision {
    let path_str = path.to_string_lossy();

    // Check deny rules first.
    for pattern_str in deny_patterns {
        if let Ok(pattern) = Pattern::new(pattern_str) {
            if pattern.matches(&path_str) || pattern.matches(path_str.as_ref()) {
                return PermissionDecision::Deny(format!(
                    "{} of '{}' matches deny pattern '{}'",
                    operation,
                    path.display(),
                    pattern_str
                ));
            }
        }
    }

    // Check allow rules.
    for pattern_str in allow_patterns {
        if let Ok(pattern) = Pattern::new(pattern_str) {
            if pattern.matches(&path_str) || pattern.matches(path_str.as_ref()) {
                return PermissionDecision::Allow;
            }
        }
    }

    // No rule matched.
    PermissionDecision::Ask(format!("{} file: {}", operation, path.display()))
}

/// Check whether a command string matches a pattern.
/// The pattern can be a glob or a simple prefix/substring.
fn command_matches(command: &str, pattern: &str) -> bool {
    // Try glob matching first.
    if let Ok(glob) = Pattern::new(pattern) {
        if glob.matches(command) {
            return true;
        }
    }

    // Also check if the base command (first word) matches the pattern exactly,
    // or if the command starts with the pattern.
    let base_command = command.split_whitespace().next().unwrap_or("");
    base_command == pattern || command.starts_with(pattern)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deny_takes_precedence() {
        let rules = PermissionRules {
            allow_read: vec!["**/*".to_string()],
            deny_read: vec!["*.secret".to_string()],
            ..Default::default()
        };
        let result = rules.check_read(Path::new("config.secret"));
        assert!(matches!(result, PermissionDecision::Deny(_)));
    }

    #[test]
    fn test_allow_rule_matches() {
        let rules = PermissionRules {
            allow_read: vec!["src/**/*.rs".to_string()],
            ..Default::default()
        };
        let result = rules.check_read(Path::new("src/main.rs"));
        assert_eq!(result, PermissionDecision::Allow);
    }

    #[test]
    fn test_no_rule_returns_ask() {
        let rules = PermissionRules::default();
        let result = rules.check_read(Path::new("some/random/file.txt"));
        assert!(matches!(result, PermissionDecision::Ask(_)));
    }

    #[test]
    fn test_check_run_deny() {
        let rules = PermissionRules {
            deny_run: vec!["rm".to_string()],
            ..Default::default()
        };
        let result = rules.check_run("rm -rf /tmp/stuff");
        assert!(matches!(result, PermissionDecision::Deny(_)));
    }

    #[test]
    fn test_check_run_allow() {
        let rules = PermissionRules {
            allow_run: vec!["ls".to_string()],
            ..Default::default()
        };
        let result = rules.check_run("ls -la");
        assert_eq!(result, PermissionDecision::Allow);
    }

    #[test]
    fn test_from_settings() {
        let settings = PermissionSettings {
            allow_read: Some(vec!["*.rs".to_string()]),
            deny_read: None,
            allow_write: None,
            deny_write: Some(vec!["/etc/*".to_string()]),
            allow_run: None,
            deny_run: None,
        };
        let rules = PermissionRules::from_settings(&settings);
        assert_eq!(rules.allow_read, vec!["*.rs".to_string()]);
        assert!(rules.deny_read.is_empty());
        assert_eq!(rules.deny_write, vec!["/etc/*".to_string()]);
    }
}
