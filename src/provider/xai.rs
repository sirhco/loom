use anyhow::Result;
use rig::client::CompletionClient;
use rig::providers::xai;

/// Wrapper around the `rig::providers::xai::Client` for xAI / Grok API access.
pub struct XAIClient {
    client: xai::Client,
}

impl XAIClient {
    /// Creates a new xAI client from the `XAI_API_KEY` environment variable.
    pub fn from_env() -> Result<Self> {
        let api_key = std::env::var("XAI_API_KEY").map_err(|_| {
            anyhow::anyhow!(
                "XAI_API_KEY environment variable is not set.\n\
                 Set it to your xAI API key:\n\n  \
                 export XAI_API_KEY=\"your-api-key\""
            )
        })?;
        Self::new(&api_key)
    }

    /// Creates a new xAI client with the given API key.
    pub fn new(api_key: &str) -> Result<Self> {
        let client = xai::Client::new(api_key)
            .map_err(|e| anyhow::anyhow!("failed to create xAI client: {e}"))?;
        Ok(Self { client })
    }

    /// Returns a completion model handle for the specified model ID.
    pub fn completion_model(&self, model_id: &str) -> xai::completion::CompletionModel {
        CompletionClient::completion_model(&self.client, model_id)
    }
}
