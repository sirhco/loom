use anyhow::Result;
use rig::client::CompletionClient;
use rig::providers::mira;

/// Wrapper around the `rig::providers::mira::Client` for Mira API access.
pub struct MiraClient {
    client: mira::Client,
}

impl MiraClient {
    /// Creates a new Mira client from the `MIRA_API_KEY` environment variable.
    pub fn from_env() -> Result<Self> {
        let api_key = std::env::var("MIRA_API_KEY").map_err(|_| {
            anyhow::anyhow!(
                "MIRA_API_KEY environment variable is not set.\n\
                 Set it to your Mira API key:\n\n  \
                 export MIRA_API_KEY=\"your-api-key\""
            )
        })?;
        Self::new(&api_key)
    }

    /// Creates a new Mira client with the given API key.
    pub fn new(api_key: &str) -> Result<Self> {
        let client = mira::Client::new(api_key)
            .map_err(|e| anyhow::anyhow!("failed to create Mira client: {e}"))?;
        Ok(Self { client })
    }

    /// Returns a completion model handle for the specified model ID.
    pub fn completion_model(&self, model_id: &str) -> mira::CompletionModel {
        CompletionClient::completion_model(&self.client, model_id)
    }
}
