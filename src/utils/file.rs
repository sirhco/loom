use std::fs;
use std::io::Read;
use std::path::Path;

use anyhow::{Context, Result};

/// Check whether a file is binary by looking for null bytes in the first 8KB.
pub fn is_binary(path: &Path) -> Result<bool> {
    let mut file = fs::File::open(path)
        .with_context(|| format!("failed to open file: {}", path.display()))?;

    let mut buffer = [0u8; 8192];
    let bytes_read = file
        .read(&mut buffer)
        .with_context(|| format!("failed to read file: {}", path.display()))?;

    Ok(buffer[..bytes_read].contains(&0))
}

/// Detect the encoding of raw bytes.
///
/// Returns `"utf-8"` if the data contains no null bytes, otherwise `"binary"`.
pub fn detect_encoding(data: &[u8]) -> &str {
    if data.contains(&0) {
        "binary"
    } else {
        "utf-8"
    }
}

/// Read a file and format it with line numbers.
///
/// Each line is formatted as `  N\tcontent` where N is the line number
/// (1-indexed), starting from `offset` and reading up to `limit` lines.
/// If `limit` is 0, all remaining lines are read.
pub fn read_file_with_line_numbers(path: &Path, offset: usize, limit: usize) -> Result<String> {
    let contents = fs::read_to_string(path)
        .with_context(|| format!("failed to read file: {}", path.display()))?;

    let lines: Vec<&str> = contents.lines().collect();
    let total_lines = lines.len();

    // offset is 0-indexed into the lines vector.
    let start = offset.min(total_lines);
    let end = if limit == 0 {
        total_lines
    } else {
        (start + limit).min(total_lines)
    };

    // Calculate the width needed for the largest line number.
    let max_line_num = if end > 0 { end } else { 1 };
    let width = max_line_num.to_string().len().max(3);

    let mut output = String::new();
    for (i, line) in lines[start..end].iter().enumerate() {
        let line_num = start + i + 1; // 1-indexed display
        output.push_str(&format!("{:>width$}\t{}\n", line_num, line, width = width));
    }

    Ok(output)
}

/// Truncate text to a maximum character count, appending a suffix if truncated.
pub fn truncate_output(text: &str, max_chars: usize) -> String {
    if text.len() <= max_chars {
        return text.to_string();
    }

    let suffix = "... (truncated)";
    if max_chars <= suffix.len() {
        return text[..max_chars].to_string();
    }

    let cut = max_chars - suffix.len();
    // Find a safe truncation point (don't split a multi-byte character).
    let safe_cut = floor_char_boundary(text, cut);
    format!("{}{}", &text[..safe_cut], suffix)
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
    use std::io::Write;

    #[test]
    fn test_detect_encoding_utf8() {
        assert_eq!(detect_encoding(b"hello world"), "utf-8");
    }

    #[test]
    fn test_detect_encoding_binary() {
        assert_eq!(detect_encoding(b"hello\x00world"), "binary");
    }

    #[test]
    fn test_is_binary_text_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.txt");
        fs::write(&path, "hello world\n").unwrap();
        assert!(!is_binary(&path).unwrap());
    }

    #[test]
    fn test_is_binary_binary_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.bin");
        let mut f = fs::File::create(&path).unwrap();
        f.write_all(&[0x89, 0x50, 0x4E, 0x47, 0x00, 0x0D, 0x0A]).unwrap();
        assert!(is_binary(&path).unwrap());
    }

    #[test]
    fn test_read_file_with_line_numbers() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.txt");
        fs::write(&path, "line one\nline two\nline three\n").unwrap();

        let result = read_file_with_line_numbers(&path, 0, 0).unwrap();
        assert!(result.contains("  1\tline one\n"));
        assert!(result.contains("  2\tline two\n"));
        assert!(result.contains("  3\tline three\n"));
    }

    #[test]
    fn test_read_file_with_offset_and_limit() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.txt");
        fs::write(&path, "a\nb\nc\nd\ne\n").unwrap();

        let result = read_file_with_line_numbers(&path, 1, 2).unwrap();
        assert!(result.contains("2\tb\n"));
        assert!(result.contains("3\tc\n"));
        assert!(!result.contains("1\ta"));
        assert!(!result.contains("4\td"));
    }

    #[test]
    fn test_truncate_output_no_truncation() {
        assert_eq!(truncate_output("hello", 10), "hello");
    }

    #[test]
    fn test_truncate_output_with_truncation() {
        let result = truncate_output("hello world this is a long string", 25);
        assert!(result.ends_with("... (truncated)"));
        assert!(result.len() <= 25);
    }

    #[test]
    fn test_truncate_output_very_short_max() {
        let result = truncate_output("hello world", 3);
        assert_eq!(result, "hel");
    }
}
