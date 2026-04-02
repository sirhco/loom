use anyhow::Result;
use rig::client::CompletionClient;
use rig::providers::galadriel;

/// Wrapper around the `rig::providers::galadriel::Client` for Galadriel API access.
pub struct GaladrielClient {
    client: galadriel::Client,
}

impl GaladrielClient {
    /// Creates a new Galadriel client from the `GALADRIEL_API_KEY` environment variable.
    pub fn from_env() -> Result<Self> {
        let api_key = std::env::var("GALADRIEL_API_KEY").map_err(|_| {
            anyhow::anyhow!(
                "GALADRIEL_API_KEY environment variable is not set.\n\
                 Set it to your Galadriel API key:\n\n  \
                 export GALADRIEL_API_KEY=\"your-api-key\""
            )
        })?;
        Self::new(&api_key)
    }

    /// Creates a new Galadriel client with the given API key.
    pub fn new(api_key: &str) -> Result<Self> {
        let client = galadriel::Client::builder()
            .api_key(api_key)
            .build()
            .map_err(|e| anyhow::anyhow!("failed to create Galadriel client: {e}"))?;
        Ok(Self { client })
    }

    /// Returns a completion model handle for the specified model ID.
    pub fn completion_model(&self, model_id: &str) -> galadriel::CompletionModel {
        CompletionClient::completion_model(&self.client, model_id)
    }
}
