use anyhow::Result;
use rig::client::{CompletionClient, Nothing};
use rig::providers::ollama;

/// Wrapper around the `rig::providers::ollama::Client` for local Ollama access.
pub struct OllamaClient {
    client: ollama::Client,
}

impl OllamaClient {
    /// Creates a new Ollama client.
    ///
    /// Uses `OLLAMA_API_BASE_URL` if set, otherwise defaults to `http://localhost:11434`.
    pub fn from_env() -> Result<Self> {
        let client = if let Ok(base_url) = std::env::var("OLLAMA_API_BASE_URL") {
            ollama::Client::builder()
                .api_key(Nothing)
                .base_url(&base_url)
                .build()
                .map_err(|e| anyhow::anyhow!("failed to create Ollama client: {e}"))?
        } else {
            ollama::Client::new(Nothing)
                .map_err(|e| anyhow::anyhow!("failed to create Ollama client: {e}"))?
        };
        Ok(Self { client })
    }

    /// Returns a completion model handle for the specified model ID.
    pub fn completion_model(&self, model_id: &str) -> ollama::CompletionModel {
        CompletionClient::completion_model(&self.client, model_id)
    }
}
