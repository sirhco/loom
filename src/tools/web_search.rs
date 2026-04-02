use rig::completion::ToolDefinition;
use rig::tool::Tool;
use serde::{Deserialize, Serialize};

/// Tool for performing web searches (placeholder implementation).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSearchTool;

/// Arguments for the WebSearchTool.
#[derive(Debug, Clone, Deserialize)]
pub struct WebSearchArgs {
    /// The search query.
    pub query: String,
    /// Number of results to return (default 5).
    pub num_results: Option<usize>,
}

/// Error type for WebSearchTool.
#[derive(Debug, thiserror::Error)]
pub enum WebSearchError {
    #[error("Search failed: {0}")]
    SearchFailed(String),
}

impl Tool for WebSearchTool {
    const NAME: &'static str = "web_search";

    type Error = WebSearchError;
    type Args = WebSearchArgs;
    type Output = String;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: "web_search".to_string(),
            description: "Search the web for information. Returns search results for the given query.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "The search query"
                    },
                    "num_results": {
                        "type": "number",
                        "description": "Number of results to return (default 5)"
                    }
                },
                "required": ["query"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        // Web search API is not configured. Instead of failing, guide the AI
        // to use web_fetch with known documentation URLs for the query topic.
        Ok(format!(
            "Web search API is not configured. Use the web_fetch tool instead to look up \
             documentation directly. Based on your query \"{}\", try fetching from:\n\n\
             - Go packages: https://pkg.go.dev/{{package_path}}\n\
             - Rust crates: https://docs.rs/{{crate_name}}\n\
             - npm packages: https://www.npmjs.com/package/{{name}}\n\
             - Python packages: https://pypi.org/project/{{name}}\n\
             - GitHub repos: https://github.com/{{owner}}/{{repo}}\n\
             - GitHub search: https://github.com/search?q={{query}}&type=repositories\n\
             - Official documentation sites for the specific framework or tool\n\n\
             Use web_fetch with the most relevant URL for your query.",
            args.query
        ))
    }
}
