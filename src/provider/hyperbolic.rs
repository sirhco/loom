use anyhow::Result;
use rig::client::CompletionClient;
use rig::providers::hyperbolic;

/// Wrapper around the `rig::providers::hyperbolic::Client` for Hyperbolic API access.
pub struct HyperbolicClient {
    client: hyperbolic::Client,
}

impl HyperbolicClient {
    /// Creates a new Hyperbolic client from the `HYPERBOLIC_API_KEY` environment variable.
    pub fn from_env() -> Result<Self> {
        let api_key = std::env::var("HYPERBOLIC_API_KEY").map_err(|_| {
            anyhow::anyhow!(
                "HYPERBOLIC_API_KEY environment variable is not set.\n\
                 Set it to your Hyperbolic API key:\n\n  \
                 export HYPERBOLIC_API_KEY=\"your-api-key\""
            )
        })?;
        Self::new(&api_key)
    }

    /// Creates a new Hyperbolic client with the given API key.
    pub fn new(api_key: &str) -> Result<Self> {
        let client = hyperbolic::Client::new(api_key)
            .map_err(|e| anyhow::anyhow!("failed to create Hyperbolic client: {e}"))?;
        Ok(Self { client })
    }

    /// Returns a completion model handle for the specified model ID.
    pub fn completion_model(&self, model_id: &str) -> hyperbolic::CompletionModel {
        CompletionClient::completion_model(&self.client, model_id)
    }
}
