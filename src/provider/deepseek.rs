use anyhow::Result;
use rig::client::CompletionClient;
use rig::providers::deepseek;

/// Wrapper around the `rig::providers::deepseek::Client` for DeepSeek API access.
pub struct DeepSeekClient {
    client: deepseek::Client,
}

impl DeepSeekClient {
    /// Creates a new DeepSeek client from the `DEEPSEEK_API_KEY` environment variable.
    pub fn from_env() -> Result<Self> {
        let api_key = std::env::var("DEEPSEEK_API_KEY").map_err(|_| {
            anyhow::anyhow!(
                "DEEPSEEK_API_KEY environment variable is not set.\n\
                 Set it to your DeepSeek API key:\n\n  \
                 export DEEPSEEK_API_KEY=\"your-api-key\""
            )
        })?;
        Self::new(&api_key)
    }

    /// Creates a new DeepSeek client with the given API key.
    pub fn new(api_key: &str) -> Result<Self> {
        let client = deepseek::Client::builder()
            .api_key(api_key)
            .build()
            .map_err(|e| anyhow::anyhow!("failed to create DeepSeek client: {e}"))?;
        Ok(Self { client })
    }

    /// Returns a completion model handle for the specified model ID.
    pub fn completion_model(&self, model_id: &str) -> deepseek::CompletionModel {
        CompletionClient::completion_model(&self.client, model_id)
    }
}
