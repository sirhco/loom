use anyhow::Result;
use rig::client::CompletionClient;
use rig::providers::cohere;

/// Wrapper around the `rig::providers::cohere::Client` for Cohere API access.
pub struct CohereClient {
    client: cohere::Client,
}

impl CohereClient {
    /// Creates a new Cohere client from the `COHERE_API_KEY` environment variable.
    pub fn from_env() -> Result<Self> {
        let api_key = std::env::var("COHERE_API_KEY").map_err(|_| {
            anyhow::anyhow!(
                "COHERE_API_KEY environment variable is not set.\n\
                 Set it to your Cohere API key:\n\n  \
                 export COHERE_API_KEY=\"your-api-key\""
            )
        })?;
        Self::new(&api_key)
    }

    /// Creates a new Cohere client with the given API key.
    pub fn new(api_key: &str) -> Result<Self> {
        let client = cohere::Client::new(api_key)
            .map_err(|e| anyhow::anyhow!("failed to create Cohere client: {e}"))?;
        Ok(Self { client })
    }

    /// Returns a completion model handle for the specified model ID.
    pub fn completion_model(&self, model_id: &str) -> cohere::CompletionModel {
        CompletionClient::completion_model(&self.client, model_id)
    }
}
