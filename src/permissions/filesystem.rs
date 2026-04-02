use std::path::Path;

use crate::permissions::mode::PermissionMode;
use crate::permissions::rules::{PermissionDecision, PermissionRules};

/// The type of file operation being performed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileOperation {
    Read,
    Write,
    Create,
    Delete,
}

impl std::fmt::Display for FileOperation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FileOperation::Read => write!(f, "read"),
            FileOperation::Write => write!(f, "write"),
            FileOperation::Create => write!(f, "create"),
            FileOperation::Delete => write!(f, "delete"),
        }
    }
}

/// Check whether a file operation should be allowed based on the permission
/// mode and the configured rules.
pub fn check_file_permission(
    path: &Path,
    operation: FileOperation,
    rules: &PermissionRules,
    mode: PermissionMode,
) -> PermissionDecision {
    // Bypass mode: always allow everything.
    if mode == PermissionMode::Bypass {
        return PermissionDecision::Allow;
    }

    // Check deny rules first (deny always takes priority regardless of mode).
    let rule_check = match operation {
        FileOperation::Read => rules.check_read(path),
        FileOperation::Write | FileOperation::Create | FileOperation::Delete => {
            rules.check_write(path)
        }
    };

    // If an explicit deny rule matched, honor it in all modes.
    if let PermissionDecision::Deny(reason) = &rule_check {
        return PermissionDecision::Deny(reason.clone());
    }

    // If an explicit allow rule matched, honor it.
    if rule_check == PermissionDecision::Allow {
        return PermissionDecision::Allow;
    }

    // No explicit rule matched. Apply mode-based defaults.
    match mode {
        PermissionMode::Bypass => {
            // Already handled above, but for completeness.
            PermissionDecision::Allow
        }
        PermissionMode::Auto => match operation {
            FileOperation::Read => PermissionDecision::Allow,
            FileOperation::Write | FileOperation::Create | FileOperation::Delete => {
                PermissionDecision::Ask(format!("{} file: {}", operation, path.display()))
            }
        },
        PermissionMode::Default => match operation {
            FileOperation::Read => PermissionDecision::Allow,
            FileOperation::Write | FileOperation::Create | FileOperation::Delete => {
                PermissionDecision::Ask(format!("{} file: {}", operation, path.display()))
            }
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn empty_rules() -> PermissionRules {
        PermissionRules::default()
    }

    #[test]
    fn test_bypass_always_allows() {
        let rules = PermissionRules {
            deny_write: vec!["/etc/*".to_string()],
            ..Default::default()
        };
        let result = check_file_permission(
            Path::new("/etc/passwd"),
            FileOperation::Write,
            &rules,
            PermissionMode::Bypass,
        );
        assert_eq!(result, PermissionDecision::Allow);
    }

    #[test]
    fn test_deny_rule_overrides_auto() {
        let rules = PermissionRules {
            deny_read: vec!["*.secret".to_string()],
            ..Default::default()
        };
        let result = check_file_permission(
            Path::new("api.secret"),
            FileOperation::Read,
            &rules,
            PermissionMode::Auto,
        );
        assert!(matches!(result, PermissionDecision::Deny(_)));
    }

    #[test]
    fn test_default_allows_reads() {
        let result = check_file_permission(
            Path::new("src/main.rs"),
            FileOperation::Read,
            &empty_rules(),
            PermissionMode::Default,
        );
        assert_eq!(result, PermissionDecision::Allow);
    }

    #[test]
    fn test_default_asks_for_writes() {
        let result = check_file_permission(
            Path::new("src/main.rs"),
            FileOperation::Write,
            &empty_rules(),
            PermissionMode::Default,
        );
        assert!(matches!(result, PermissionDecision::Ask(_)));
    }

    #[test]
    fn test_auto_allows_reads() {
        let result = check_file_permission(
            Path::new("src/main.rs"),
            FileOperation::Read,
            &empty_rules(),
            PermissionMode::Auto,
        );
        assert_eq!(result, PermissionDecision::Allow);
    }

    #[test]
    fn test_auto_asks_for_deletes() {
        let result = check_file_permission(
            Path::new("src/main.rs"),
            FileOperation::Delete,
            &empty_rules(),
            PermissionMode::Auto,
        );
        assert!(matches!(result, PermissionDecision::Ask(_)));
    }

    #[test]
    fn test_explicit_allow_rule() {
        let rules = PermissionRules {
            allow_write: vec!["src/**/*.rs".to_string()],
            ..Default::default()
        };
        let result = check_file_permission(
            Path::new("src/main.rs"),
            FileOperation::Write,
            &rules,
            PermissionMode::Default,
        );
        assert_eq!(result, PermissionDecision::Allow);
    }
}
