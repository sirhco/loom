use std::path::Path;

use ignore::WalkBuilder;

/// Walks the working directory (respecting .gitignore) and builds a list of
/// relative file paths for fuzzy matching. Only indexes cwd and below.
pub fn build_file_index(cwd: &Path) -> Vec<String> {
    WalkBuilder::new(cwd)
        .max_depth(Some(8))
        .hidden(true)
        .build()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_some_and(|ft| ft.is_file()))
        .filter_map(|e| {
            e.path()
                .strip_prefix(cwd)
                .ok()
                .map(|p| p.to_string_lossy().to_string())
        })
        .collect()
}

/// Fuzzy-matches file paths from the index against a query string.
///
/// Scoring priority:
/// 1. Exact filename match
/// 2. Filename starts with query
/// 3. Filename contains query
/// 4. Full path contains query
///
/// Returns up to 20 matches sorted by relevance then path length.
pub fn fuzzy_match_files(index: &[String], query: &str) -> Vec<String> {
    if query.is_empty() {
        return Vec::new();
    }
    let query_lower = query.to_lowercase();
    let mut scored: Vec<(usize, &String)> = index
        .iter()
        .filter_map(|path| {
            let path_lower = path.to_lowercase();
            if !path_lower.contains(&query_lower) {
                return None;
            }
            let filename = path.rsplit('/').next().unwrap_or(path);
            let filename_lower = filename.to_lowercase();
            let score = if filename_lower == query_lower {
                0 // exact filename match
            } else if filename_lower.starts_with(&query_lower) {
                1 // filename starts with query
            } else if filename_lower.contains(&query_lower) {
                2 // filename contains query
            } else {
                3 // path contains query
            };
            Some((score, path))
        })
        .collect();

    scored.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.len().cmp(&b.1.len())));
    scored.into_iter().take(20).map(|(_, p)| p.clone()).collect()
}

/// Filters slash commands by a prefix string.
pub fn filter_slash_commands(commands: &[String], prefix: &str) -> Vec<String> {
    commands
        .iter()
        .filter(|c| c.starts_with(prefix))
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fuzzy_match_exact_filename() {
        let index = vec![
            "src/main.rs".to_string(),
            "src/lib.rs".to_string(),
            "tests/main_test.rs".to_string(),
        ];
        let results = fuzzy_match_files(&index, "main.rs");
        assert!(!results.is_empty());
        assert_eq!(results[0], "src/main.rs");
    }

    #[test]
    fn test_fuzzy_match_partial() {
        let index = vec![
            "src/engine/query.rs".to_string(),
            "src/engine/query_engine.rs".to_string(),
            "src/cli/args.rs".to_string(),
        ];
        let results = fuzzy_match_files(&index, "query");
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_fuzzy_match_empty_query() {
        let index = vec!["src/main.rs".to_string()];
        let results = fuzzy_match_files(&index, "");
        assert!(results.is_empty());
    }

    #[test]
    fn test_filter_slash_commands() {
        let commands = vec![
            "/help".to_string(),
            "/history".to_string(),
            "/clear".to_string(),
            "/config".to_string(),
        ];
        let results = filter_slash_commands(&commands, "/h");
        assert_eq!(results.len(), 2);
        assert!(results.contains(&"/help".to_string()));
        assert!(results.contains(&"/history".to_string()));
    }
}
