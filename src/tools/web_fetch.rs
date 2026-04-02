use rig::completion::ToolDefinition;
use rig::tool::Tool;
use serde::{Deserialize, Serialize};

/// Maximum response body size in bytes (100KB).
const MAX_BODY_BYTES: usize = 100 * 1024;

/// Tool for fetching URL content.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebFetchTool;

/// Arguments for the WebFetchTool.
#[derive(Debug, Clone, Deserialize)]
pub struct WebFetchArgs {
    /// The URL to fetch.
    pub url: String,
}

/// Error type for WebFetchTool.
#[derive(Debug, thiserror::Error)]
pub enum WebFetchError {
    #[error("HTTP request failed: {0}")]
    Request(String),
    #[error("Invalid URL: {0}")]
    InvalidUrl(String),
}

impl Tool for WebFetchTool {
    const NAME: &'static str = "web_fetch";

    type Error = WebFetchError;
    type Args = WebFetchArgs;
    type Output = String;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: "web_fetch".to_string(),
            description: "Fetch the content of a URL and return it as text. Useful for reading web pages, API responses, and downloading text content.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "url": {
                        "type": "string",
                        "description": "The URL to fetch"
                    }
                },
                "required": ["url"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let response = reqwest::get(&args.url)
            .await
            .map_err(|e| WebFetchError::Request(e.to_string()))?;

        let status = response.status();
        let headers = response.headers().clone();

        let content_type = headers
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("unknown")
            .to_string();

        let body = response
            .text()
            .await
            .map_err(|e| WebFetchError::Request(e.to_string()))?;

        let mut result = format!("Status: {status}\nContent-Type: {content_type}\n\n");

        if body.len() > MAX_BODY_BYTES {
            result.push_str(&body[..MAX_BODY_BYTES]);
            result.push_str("\n\n... (response truncated)");
        } else {
            result.push_str(&body);
        }

        Ok(result)
    }
}
