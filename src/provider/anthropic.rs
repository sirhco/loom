use anyhow::Result;
use rig::client::CompletionClient;
use rig::providers::anthropic;

/// Wrapper around the `rig::providers::anthropic::Client` for Anthropic API access.
pub struct AnthropicClient {
    client: anthropic::Client,
}

impl AnthropicClient {
    /// Creates a new Anthropic client from the `ANTHROPIC_API_KEY` environment variable.
    pub fn from_env() -> Result<Self> {
        let api_key = std::env::var("ANTHROPIC_API_KEY").map_err(|_| {
            anyhow::anyhow!(
                "ANTHROPIC_API_KEY environment variable is not set.\n\
                 Set it to your Anthropic API key:\n\n  \
                 export ANTHROPIC_API_KEY=\"your-api-key\""
            )
        })?;
        Self::new(&api_key)
    }

    /// Creates a new Anthropic client with the given API key.
    pub fn new(api_key: &str) -> Result<Self> {
        let client = anthropic::Client::builder()
            .api_key(api_key)
            .build()
            .map_err(|e| anyhow::anyhow!("failed to create Anthropic client: {e}"))?;
        Ok(Self { client })
    }

    /// Returns a completion model handle for the specified model ID.
    pub fn completion_model(&self, model_id: &str) -> anthropic::completion::CompletionModel {
        CompletionClient::completion_model(&self.client, model_id)
    }
}
