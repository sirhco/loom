use anyhow::Result;
use rig::client::CompletionClient;
use rig::providers::moonshot;

/// Wrapper around the `rig::providers::moonshot::Client` for Moonshot API access.
pub struct MoonshotClient {
    client: moonshot::Client,
}

impl MoonshotClient {
    /// Creates a new Moonshot client from the `MOONSHOT_API_KEY` environment variable.
    pub fn from_env() -> Result<Self> {
        let api_key = std::env::var("MOONSHOT_API_KEY").map_err(|_| {
            anyhow::anyhow!(
                "MOONSHOT_API_KEY environment variable is not set.\n\
                 Set it to your Moonshot API key:\n\n  \
                 export MOONSHOT_API_KEY=\"your-api-key\""
            )
        })?;
        Self::new(&api_key)
    }

    /// Creates a new Moonshot client with the given API key.
    pub fn new(api_key: &str) -> Result<Self> {
        let client = moonshot::Client::new(api_key)
            .map_err(|e| anyhow::anyhow!("failed to create Moonshot client: {e}"))?;
        Ok(Self { client })
    }

    /// Returns a completion model handle for the specified model ID.
    pub fn completion_model(&self, model_id: &str) -> moonshot::CompletionModel {
        CompletionClient::completion_model(&self.client, model_id)
    }
}
