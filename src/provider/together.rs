use anyhow::Result;
use rig::client::CompletionClient;
use rig::providers::together;

/// Wrapper around the `rig::providers::together::Client` for Together AI API access.
pub struct TogetherClient {
    client: together::Client,
}

impl TogetherClient {
    /// Creates a new Together AI client from the `TOGETHER_API_KEY` environment variable.
    pub fn from_env() -> Result<Self> {
        let api_key = std::env::var("TOGETHER_API_KEY").map_err(|_| {
            anyhow::anyhow!(
                "TOGETHER_API_KEY environment variable is not set.\n\
                 Set it to your Together AI API key:\n\n  \
                 export TOGETHER_API_KEY=\"your-api-key\""
            )
        })?;
        Self::new(&api_key)
    }

    /// Creates a new Together AI client with the given API key.
    pub fn new(api_key: &str) -> Result<Self> {
        let client = together::Client::new(api_key)
            .map_err(|e| anyhow::anyhow!("failed to create Together AI client: {e}"))?;
        Ok(Self { client })
    }

    /// Returns a completion model handle for the specified model ID.
    pub fn completion_model(&self, model_id: &str) -> together::CompletionModel {
        CompletionClient::completion_model(&self.client, model_id)
    }
}
