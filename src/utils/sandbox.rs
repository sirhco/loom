use std::path::{Path, PathBuf};

use thiserror::Error;

/// Errors that can occur during sandbox validation.
#[derive(Debug, Error)]
pub enum SandboxError {
    #[error("denied command: {0}")]
    DeniedCommand(String),

    #[error("path escape attempt: {0}")]
    PathEscape(String),
}

/// Configuration for the command sandbox.
#[derive(Debug, Clone, Default)]
pub struct SandboxConfig {
    /// Directories that commands are allowed to access.
    pub allowed_dirs: Vec<PathBuf>,
    /// Commands that are explicitly denied.
    pub denied_commands: Vec<String>,
}

/// Validate that a command is safe to run within the sandbox.
///
/// Checks:
/// 1. The base command is not in the denied list.
/// 2. The command does not reference paths outside the allowed directories
///    via `..` traversal or absolute paths.
pub fn validate_command(command: &str, config: &SandboxConfig) -> Result<(), SandboxError> {
    let command = command.trim();
    if command.is_empty() {
        return Ok(());
    }

    // Check each segment in a piped or chained command.
    for segment in split_command_segments(command) {
        let segment = segment.trim();
        if segment.is_empty() {
            continue;
        }

        let parts: Vec<&str> = segment.split_whitespace().collect();
        if parts.is_empty() {
            continue;
        }

        let base_cmd = extract_base_command(parts[0]);

        // Check denied commands.
        for denied in &config.denied_commands {
            if base_cmd == denied.as_str() {
                return Err(SandboxError::DeniedCommand(format!(
                    "command '{}' is denied by sandbox policy",
                    base_cmd
                )));
            }
        }

        // Check arguments for path escapes.
        for arg in &parts[1..] {
            if arg.starts_with('-') {
                continue;
            }
            check_path_escape(arg, &config.allowed_dirs)?;
        }
    }

    Ok(())
}

/// Split a command string into individual segments, splitting on `|`, `&&`, and `;`.
fn split_command_segments(command: &str) -> Vec<&str> {
    let mut segments = Vec::new();
    let mut remaining = command;

    while !remaining.is_empty() {
        // Find the next delimiter.
        let next_pipe = remaining.find('|');
        let next_and = remaining.find("&&");
        let next_semi = remaining.find(';');

        let (pos, skip) = [
            next_pipe.map(|p| (p, 1)),
            next_and.map(|p| (p, 2)),
            next_semi.map(|p| (p, 1)),
        ]
        .into_iter()
        .flatten()
        .min_by_key(|(p, _)| *p)
        .unwrap_or((remaining.len(), 0));

        let segment = &remaining[..pos];
        if !segment.trim().is_empty() {
            segments.push(segment.trim());
        }

        if pos + skip >= remaining.len() {
            break;
        }
        remaining = &remaining[pos + skip..];
    }

    segments
}

/// Extract the base command name, stripping any path prefix.
fn extract_base_command(cmd: &str) -> &str {
    cmd.rsplit('/').next().unwrap_or(cmd)
}

/// Check whether a path argument attempts to escape the sandbox.
fn check_path_escape(arg: &str, allowed_dirs: &[PathBuf]) -> Result<(), SandboxError> {
    let path = Path::new(arg);

    // Check for `..` components that could escape the working directory.
    for component in path.components() {
        if let std::path::Component::ParentDir = component {
            return Err(SandboxError::PathEscape(format!(
                "path '{}' contains '..' which may escape the sandbox",
                arg
            )));
        }
    }

    // Check absolute paths against allowed directories.
    if path.is_absolute() {
        if allowed_dirs.is_empty() {
            return Err(SandboxError::PathEscape(format!(
                "absolute path '{}' is not within any allowed directory",
                arg
            )));
        }

        let allowed = allowed_dirs.iter().any(|dir| path.starts_with(dir));
        if !allowed {
            return Err(SandboxError::PathEscape(format!(
                "absolute path '{}' is not within any allowed directory",
                arg
            )));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config_with_dir(dir: &str) -> SandboxConfig {
        SandboxConfig {
            allowed_dirs: vec![PathBuf::from(dir)],
            denied_commands: vec!["rm".to_string(), "sudo".to_string()],
        }
    }

    #[test]
    fn test_empty_command() {
        let config = SandboxConfig::default();
        assert!(validate_command("", &config).is_ok());
    }

    #[test]
    fn test_denied_command() {
        let config = config_with_dir("/tmp");
        let result = validate_command("rm -rf .", &config);
        assert!(matches!(result, Err(SandboxError::DeniedCommand(_))));
    }

    #[test]
    fn test_denied_in_pipe() {
        let config = config_with_dir("/tmp");
        let result = validate_command("ls | sudo tee file", &config);
        assert!(matches!(result, Err(SandboxError::DeniedCommand(_))));
    }

    #[test]
    fn test_path_escape_dotdot() {
        let config = config_with_dir("/home/user/project");
        let result = validate_command("cat ../../../etc/passwd", &config);
        assert!(matches!(result, Err(SandboxError::PathEscape(_))));
    }

    #[test]
    fn test_absolute_path_outside_allowed() {
        let config = config_with_dir("/home/user/project");
        let result = validate_command("cat /etc/passwd", &config);
        assert!(matches!(result, Err(SandboxError::PathEscape(_))));
    }

    #[test]
    fn test_absolute_path_inside_allowed() {
        let config = config_with_dir("/home/user/project");
        let result = validate_command("cat /home/user/project/src/main.rs", &config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_relative_path_ok() {
        let config = config_with_dir("/home/user/project");
        let result = validate_command("cat src/main.rs", &config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_safe_command_no_denied() {
        let config = SandboxConfig {
            allowed_dirs: vec![],
            denied_commands: vec![],
        };
        let result = validate_command("ls -la", &config);
        assert!(result.is_ok());
    }
}
