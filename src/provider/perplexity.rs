use anyhow::Result;
use rig::client::CompletionClient;
use rig::providers::perplexity;

/// Wrapper around the `rig::providers::perplexity::Client` for Perplexity API access.
pub struct PerplexityClient {
    client: perplexity::Client,
}

impl PerplexityClient {
    /// Creates a new Perplexity client from the `PERPLEXITY_API_KEY` environment variable.
    pub fn from_env() -> Result<Self> {
        let api_key = std::env::var("PERPLEXITY_API_KEY").map_err(|_| {
            anyhow::anyhow!(
                "PERPLEXITY_API_KEY environment variable is not set.\n\
                 Set it to your Perplexity API key:\n\n  \
                 export PERPLEXITY_API_KEY=\"your-api-key\""
            )
        })?;
        Self::new(&api_key)
    }

    /// Creates a new Perplexity client with the given API key.
    pub fn new(api_key: &str) -> Result<Self> {
        let client = perplexity::Client::new(api_key)
            .map_err(|e| anyhow::anyhow!("failed to create Perplexity client: {e}"))?;
        Ok(Self { client })
    }

    /// Returns a completion model handle for the specified model ID.
    pub fn completion_model(&self, model_id: &str) -> perplexity::CompletionModel {
        CompletionClient::completion_model(&self.client, model_id)
    }
}
