use std::time::Duration;

/// Format a USD cost for display.
///
/// Always shows two decimal places: "$0.03", "$1.24", "$12.50".
pub fn format_cost(usd: f64) -> String {
    format!("${:.2}", usd)
}

/// Format a duration for human-readable display.
///
/// - Under 1 minute: "2.3s"
/// - Under 1 hour: "1m 45s"
/// - 1 hour or more: "1h 2m"
pub fn format_duration(duration: Duration) -> String {
    let total_secs = duration.as_secs_f64();

    if total_secs < 60.0 {
        format!("{:.1}s", total_secs)
    } else if total_secs < 3600.0 {
        let minutes = (total_secs / 60.0).floor() as u64;
        let seconds = (total_secs % 60.0).floor() as u64;
        if seconds == 0 {
            format!("{}m", minutes)
        } else {
            format!("{}m {}s", minutes, seconds)
        }
    } else {
        let hours = (total_secs / 3600.0).floor() as u64;
        let minutes = ((total_secs % 3600.0) / 60.0).floor() as u64;
        if minutes == 0 {
            format!("{}h", hours)
        } else {
            format!("{}h {}m", hours, minutes)
        }
    }
}

/// Truncate a string to a maximum length, appending "..." if truncated.
///
/// Handles multi-byte characters safely.
pub fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        return s.to_string();
    }

    if max_len <= 3 {
        return s.chars().take(max_len).collect();
    }

    let target = max_len - 3;
    let safe_end = floor_char_boundary(s, target);
    format!("{}...", &s[..safe_end])
}

/// Remove ANSI escape sequences from a string.
///
/// Uses the `strip-ansi-escapes` crate for reliable removal.
pub fn strip_ansi(s: &str) -> String {
    let stripped = strip_ansi_escapes::strip(s);
    String::from_utf8_lossy(&stripped).into_owned()
}

/// Find the largest byte index <= `index` that is a valid char boundary.
fn floor_char_boundary(s: &str, index: usize) -> usize {
    if index >= s.len() {
        return s.len();
    }
    let mut i = index;
    while i > 0 && !s.is_char_boundary(i) {
        i -= 1;
    }
    i
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_cost() {
        assert_eq!(format_cost(0.0), "$0.00");
        assert_eq!(format_cost(0.03), "$0.03");
        assert_eq!(format_cost(1.24), "$1.24");
        assert_eq!(format_cost(12.5), "$12.50");
        assert_eq!(format_cost(100.0), "$100.00");
    }

    #[test]
    fn test_format_duration_seconds() {
        assert_eq!(format_duration(Duration::from_secs_f64(0.5)), "0.5s");
        assert_eq!(format_duration(Duration::from_secs_f64(2.3)), "2.3s");
        assert_eq!(format_duration(Duration::from_secs_f64(59.9)), "59.9s");
    }

    #[test]
    fn test_format_duration_minutes() {
        assert_eq!(format_duration(Duration::from_secs(60)), "1m");
        assert_eq!(format_duration(Duration::from_secs(105)), "1m 45s");
        assert_eq!(format_duration(Duration::from_secs(600)), "10m");
    }

    #[test]
    fn test_format_duration_hours() {
        assert_eq!(format_duration(Duration::from_secs(3600)), "1h");
        assert_eq!(format_duration(Duration::from_secs(3720)), "1h 2m");
        assert_eq!(format_duration(Duration::from_secs(7200)), "2h");
    }

    #[test]
    fn test_truncate_string_no_truncation() {
        assert_eq!(truncate_string("hello", 10), "hello");
    }

    #[test]
    fn test_truncate_string_with_truncation() {
        assert_eq!(truncate_string("hello world", 8), "hello...");
    }

    #[test]
    fn test_truncate_string_very_short() {
        assert_eq!(truncate_string("hello", 2), "he");
    }

    #[test]
    fn test_strip_ansi() {
        assert_eq!(strip_ansi("hello"), "hello");
        assert_eq!(strip_ansi("\x1b[31mred\x1b[0m"), "red");
        assert_eq!(strip_ansi("\x1b[1;32mbold green\x1b[0m"), "bold green");
    }

    #[test]
    fn test_truncate_multibyte() {
        // Emoji is multi-byte; make sure we don't panic.
        let s = "hello \u{1f600} world";
        let result = truncate_string(s, 10);
        assert!(result.len() <= 13); // 10 + "..."
    }
}
