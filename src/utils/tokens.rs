/// Estimate the number of tokens in a text string.
///
/// Uses the standard approximation of ~4 characters per token for English text.
pub fn estimate_tokens(text: &str) -> usize {
    if text.is_empty() {
        return 0;
    }
    // Roughly 4 characters per token for English text.
    (text.len() + 3) / 4
}

/// Estimate the total tokens across a slice of message content strings.
///
/// Adds a small per-message overhead (4 tokens) to account for role markers
/// and structural formatting in the API payload.
pub fn estimate_message_tokens(messages: &[&str]) -> usize {
    let overhead_per_message: usize = 4;
    messages
        .iter()
        .map(|content| estimate_tokens(content) + overhead_per_message)
        .sum()
}

/// Format a token count for human-readable display.
///
/// - Below 1,000: exact number (e.g. "842")
/// - Below 1,000,000: thousands (e.g. "1.2k", "45.6k")
/// - Above 1,000,000: millions (e.g. "1.2M")
pub fn format_tokens(count: usize) -> String {
    if count < 1_000 {
        format!("{}", count)
    } else if count < 1_000_000 {
        let k = count as f64 / 1_000.0;
        if k < 10.0 {
            format!("{:.1}k", k)
        } else {
            format!("{:.1}k", k)
        }
    } else {
        let m = count as f64 / 1_000_000.0;
        format!("{:.1}M", m)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_estimate_tokens() {
        assert_eq!(estimate_tokens(""), 0);
        assert_eq!(estimate_tokens("hi"), 1); // 2 chars -> ceil(2/4) = 1
        assert_eq!(estimate_tokens("hello world"), 3); // 11 chars -> ceil(11/4) = 3
        assert_eq!(estimate_tokens("abcd"), 1); // 4 chars -> 1
        assert_eq!(estimate_tokens("abcde"), 2); // 5 chars -> 2
    }

    #[test]
    fn test_estimate_message_tokens() {
        let messages = vec!["hello world", "how are you"];
        let result = estimate_message_tokens(&messages);
        // "hello world" = 3 tokens + 4 overhead = 7
        // "how are you" = 3 tokens + 4 overhead = 7
        assert_eq!(result, 14);
    }

    #[test]
    fn test_format_tokens() {
        assert_eq!(format_tokens(0), "0");
        assert_eq!(format_tokens(42), "42");
        assert_eq!(format_tokens(999), "999");
        assert_eq!(format_tokens(1_000), "1.0k");
        assert_eq!(format_tokens(1_234), "1.2k");
        assert_eq!(format_tokens(45_600), "45.6k");
        assert_eq!(format_tokens(999_999), "1000.0k");
        assert_eq!(format_tokens(1_000_000), "1.0M");
        assert_eq!(format_tokens(1_234_567), "1.2M");
    }
}
