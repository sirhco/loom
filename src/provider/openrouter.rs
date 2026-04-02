use anyhow::Result;
use rig::client::CompletionClient;
use rig::providers::openrouter;

/// Wrapper around the `rig::providers::openrouter::Client` for OpenRouter API access.
pub struct OpenRouterClient {
    client: openrouter::Client,
}

impl OpenRouterClient {
    /// Creates a new OpenRouter client from the `OPENROUTER_API_KEY` environment variable.
    pub fn from_env() -> Result<Self> {
        let api_key = std::env::var("OPENROUTER_API_KEY").map_err(|_| {
            anyhow::anyhow!(
                "OPENROUTER_API_KEY environment variable is not set.\n\
                 Set it to your OpenRouter API key:\n\n  \
                 export OPENROUTER_API_KEY=\"your-api-key\""
            )
        })?;
        Self::new(&api_key)
    }

    /// Creates a new OpenRouter client with the given API key.
    pub fn new(api_key: &str) -> Result<Self> {
        let client = openrouter::Client::new(api_key)
            .map_err(|e| anyhow::anyhow!("failed to create OpenRouter client: {e}"))?;
        Ok(Self { client })
    }

    /// Returns a completion model handle for the specified model ID.
    pub fn completion_model(&self, model_id: &str) -> openrouter::CompletionModel {
        CompletionClient::completion_model(&self.client, model_id)
    }
}
