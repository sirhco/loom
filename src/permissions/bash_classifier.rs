use std::fmt;

/// Safety classification for a bash command.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandSafety {
    /// The command is known to be safe (read-only or informational).
    Safe,
    /// The command is known to be dangerous, with a reason.
    Dangerous(String),
    /// The command is not recognized; safety is unknown.
    Unknown,
}

impl fmt::Display for CommandSafety {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CommandSafety::Safe => write!(f, "safe"),
            CommandSafety::Dangerous(reason) => write!(f, "dangerous: {}", reason),
            CommandSafety::Unknown => write!(f, "unknown"),
        }
    }
}

/// Commands that are known to be safe (read-only / informational).
const SAFE_COMMANDS: &[&str] = &[
    "ls", "cat", "head", "tail", "echo", "pwd", "which", "whoami",
    "date", "uname", "wc", "sort", "uniq", "tr", "cut", "diff",
    "file", "stat", "du", "df", "env", "printenv", "hostname",
    "id", "groups",
];

/// Git subcommands that are safe (read-only).
const SAFE_GIT_SUBCOMMANDS: &[&str] = &[
    "status", "log", "diff", "branch", "remote", "show", "tag",
];

/// Cargo subcommands that are safe.
const SAFE_CARGO_SUBCOMMANDS: &[&str] = &[
    "check", "test", "clippy", "doc",
];

/// Safe compound commands (command + specific args).
const SAFE_COMPOUND_COMMANDS: &[(&str, &str)] = &[
    ("npm", "test"),
    ("npm", "run"),
    ("node", "--version"),
    ("python", "--version"),
    ("rustc", "--version"),
];

/// Commands that are inherently dangerous.
const DANGEROUS_COMMANDS: &[(&str, &str)] = &[
    ("rm", "removes files or directories"),
    ("rmdir", "removes directories"),
    ("chmod", "changes file permissions"),
    ("chown", "changes file ownership"),
    ("sudo", "runs with elevated privileges"),
    ("su", "switches user"),
    ("kill", "terminates processes"),
    ("killall", "terminates processes by name"),
    ("pkill", "terminates processes by pattern"),
    ("shutdown", "shuts down the system"),
    ("reboot", "reboots the system"),
    ("mkfs", "creates a filesystem (destructive)"),
    ("dd", "low-level data copy (destructive)"),
    ("ssh", "opens remote shell connection"),
    ("scp", "copies files over SSH"),
    ("docker", "container operations"),
    ("kubectl", "Kubernetes cluster operations"),
];

/// Classify a bash command string as safe, dangerous, or unknown.
///
/// For piped commands, each segment is checked individually and the most
/// dangerous classification wins.
pub fn classify_command(command: &str) -> CommandSafety {
    let command = command.trim();
    if command.is_empty() {
        return CommandSafety::Safe;
    }

    // Split on pipes and check each segment.
    let segments: Vec<&str> = command.split('|').collect();
    let mut worst = CommandSafety::Safe;

    for segment in segments {
        let classification = classify_single_segment(segment.trim());
        worst = merge_safety(worst, classification);
        // Short-circuit if already dangerous.
        if matches!(worst, CommandSafety::Dangerous(_)) {
            return worst;
        }
    }

    worst
}

/// Classify a single command segment (no pipes).
fn classify_single_segment(segment: &str) -> CommandSafety {
    let segment = segment.trim();
    if segment.is_empty() {
        return CommandSafety::Safe;
    }

    // Handle command chaining with && and ;
    for part in segment.split("&&").flat_map(|s| s.split(';')) {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        let result = classify_atomic_command(part);
        if matches!(result, CommandSafety::Dangerous(_)) {
            return result;
        }
    }

    // If no part was dangerous, classify the first part for the overall result.
    classify_atomic_command(segment.split("&&").next().unwrap_or(segment).split(';').next().unwrap_or(segment).trim())
}

/// Classify a single atomic command (no pipes, no chaining).
fn classify_atomic_command(command: &str) -> CommandSafety {
    let parts: Vec<&str> = command.split_whitespace().collect();
    if parts.is_empty() {
        return CommandSafety::Safe;
    }

    let base = extract_base_command(parts[0]);

    // Check dangerous commands first.
    for &(dangerous_cmd, reason) in DANGEROUS_COMMANDS {
        if base == dangerous_cmd {
            return CommandSafety::Dangerous(reason.to_string());
        }
    }

    // Check curl with dangerous HTTP methods.
    if base == "curl" {
        return classify_curl(&parts);
    }

    // Check wget.
    if base == "wget" {
        return CommandSafety::Dangerous("downloads files from the internet".to_string());
    }

    // Check mv to sensitive paths.
    if base == "mv" {
        return classify_mv(&parts);
    }

    // Check safe commands.
    if SAFE_COMMANDS.contains(&base) {
        return CommandSafety::Safe;
    }

    // Check git subcommands.
    if base == "git" {
        if let Some(subcommand) = parts.get(1) {
            if SAFE_GIT_SUBCOMMANDS.contains(subcommand) {
                return CommandSafety::Safe;
            }
        }
        return CommandSafety::Unknown;
    }

    // Check cargo subcommands.
    if base == "cargo" {
        if let Some(subcommand) = parts.get(1) {
            if SAFE_CARGO_SUBCOMMANDS.contains(subcommand) {
                return CommandSafety::Safe;
            }
        }
        return CommandSafety::Unknown;
    }

    // Check safe compound commands.
    if let Some(arg) = parts.get(1) {
        for &(cmd, safe_arg) in SAFE_COMPOUND_COMMANDS {
            if base == cmd && *arg == safe_arg {
                return CommandSafety::Safe;
            }
        }
    }

    CommandSafety::Unknown
}

/// Extract the base command name, stripping any path prefix.
fn extract_base_command(cmd: &str) -> &str {
    cmd.rsplit('/').next().unwrap_or(cmd)
}

/// Classify a curl command. GET/HEAD are safe; POST/PUT/DELETE/PATCH are dangerous.
fn classify_curl(parts: &[&str]) -> CommandSafety {
    for (i, part) in parts.iter().enumerate() {
        match *part {
            "-X" | "--request" => {
                if let Some(method) = parts.get(i + 1) {
                    let method_upper = method.to_uppercase();
                    if matches!(method_upper.as_str(), "POST" | "PUT" | "DELETE" | "PATCH") {
                        return CommandSafety::Dangerous(format!(
                            "curl with {} method modifies remote data",
                            method_upper
                        ));
                    }
                }
            }
            "-d" | "--data" | "--data-raw" | "--data-binary" | "--data-urlencode" => {
                return CommandSafety::Dangerous(
                    "curl with data payload (implies POST)".to_string(),
                );
            }
            _ => {}
        }
    }
    // GET-only curl is relatively safe.
    CommandSafety::Safe
}

/// Classify an mv command. Moving to system directories is dangerous.
fn classify_mv(parts: &[&str]) -> CommandSafety {
    let sensitive_paths = ["/usr", "/bin", "/sbin", "/etc", "/var", "/boot", "/sys", "/proc"];
    for part in parts.iter().skip(1) {
        if part.starts_with('-') {
            continue;
        }
        for sensitive in &sensitive_paths {
            if part.starts_with(sensitive) {
                return CommandSafety::Dangerous(format!(
                    "mv targets sensitive system path '{}'",
                    sensitive
                ));
            }
        }
    }
    CommandSafety::Unknown
}

/// Merge two safety classifications, keeping the most dangerous.
fn merge_safety(a: CommandSafety, b: CommandSafety) -> CommandSafety {
    match (&a, &b) {
        (CommandSafety::Dangerous(_), _) => a,
        (_, CommandSafety::Dangerous(_)) => b,
        (CommandSafety::Unknown, _) => a,
        (_, CommandSafety::Unknown) => b,
        _ => CommandSafety::Safe,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_safe_commands() {
        assert_eq!(classify_command("ls -la"), CommandSafety::Safe);
        assert_eq!(classify_command("cat file.txt"), CommandSafety::Safe);
        assert_eq!(classify_command("echo hello"), CommandSafety::Safe);
        assert_eq!(classify_command("pwd"), CommandSafety::Safe);
        assert_eq!(classify_command("git status"), CommandSafety::Safe);
        assert_eq!(classify_command("git log --oneline"), CommandSafety::Safe);
        assert_eq!(classify_command("cargo test"), CommandSafety::Safe);
        assert_eq!(classify_command("node --version"), CommandSafety::Safe);
    }

    #[test]
    fn test_dangerous_commands() {
        assert!(matches!(classify_command("rm -rf /tmp"), CommandSafety::Dangerous(_)));
        assert!(matches!(classify_command("sudo apt install"), CommandSafety::Dangerous(_)));
        assert!(matches!(classify_command("chmod 777 file"), CommandSafety::Dangerous(_)));
        assert!(matches!(classify_command("kill -9 1234"), CommandSafety::Dangerous(_)));
        assert!(matches!(classify_command("docker run ubuntu"), CommandSafety::Dangerous(_)));
    }

    #[test]
    fn test_curl_classification() {
        assert_eq!(classify_command("curl https://example.com"), CommandSafety::Safe);
        assert!(matches!(
            classify_command("curl -X POST https://example.com"),
            CommandSafety::Dangerous(_)
        ));
        assert!(matches!(
            classify_command("curl -d 'data' https://example.com"),
            CommandSafety::Dangerous(_)
        ));
    }

    #[test]
    fn test_piped_commands() {
        assert_eq!(classify_command("ls | sort | head"), CommandSafety::Safe);
        assert!(matches!(
            classify_command("cat file | sudo tee /etc/something"),
            CommandSafety::Dangerous(_)
        ));
    }

    #[test]
    fn test_unknown_commands() {
        assert_eq!(classify_command("my-custom-tool --flag"), CommandSafety::Unknown);
        assert_eq!(classify_command("git push"), CommandSafety::Unknown);
    }

    #[test]
    fn test_empty_command() {
        assert_eq!(classify_command(""), CommandSafety::Safe);
        assert_eq!(classify_command("   "), CommandSafety::Safe);
    }

    #[test]
    fn test_mv_sensitive_paths() {
        assert!(matches!(
            classify_command("mv file.txt /etc/config"),
            CommandSafety::Dangerous(_)
        ));
        assert_eq!(classify_command("mv a.txt b.txt"), CommandSafety::Unknown);
    }
}
