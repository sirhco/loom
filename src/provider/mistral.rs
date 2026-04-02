use anyhow::Result;
use rig::client::CompletionClient;
use rig::providers::mistral;

/// Wrapper around the `rig::providers::mistral::Client` for Mistral API access.
pub struct MistralClient {
    client: mistral::Client,
}

impl MistralClient {
    /// Creates a new Mistral client from the `MISTRAL_API_KEY` environment variable.
    pub fn from_env() -> Result<Self> {
        let api_key = std::env::var("MISTRAL_API_KEY").map_err(|_| {
            anyhow::anyhow!(
                "MISTRAL_API_KEY environment variable is not set.\n\
                 Set it to your Mistral API key:\n\n  \
                 export MISTRAL_API_KEY=\"your-api-key\""
            )
        })?;
        Self::new(&api_key)
    }

    /// Creates a new Mistral client with the given API key.
    pub fn new(api_key: &str) -> Result<Self> {
        let client = mistral::Client::new(api_key)
            .map_err(|e| anyhow::anyhow!("failed to create Mistral client: {e}"))?;
        Ok(Self { client })
    }

    /// Returns a completion model handle for the specified model ID.
    pub fn completion_model(&self, model_id: &str) -> mistral::CompletionModel {
        CompletionClient::completion_model(&self.client, model_id)
    }
}
