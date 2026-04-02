use anyhow::Result;
use rig::client::CompletionClient;
use rig::providers::huggingface;

/// Wrapper around the `rig::providers::huggingface::Client` for Hugging Face API access.
pub struct HuggingFaceClient {
    client: huggingface::Client,
}

impl HuggingFaceClient {
    /// Creates a new Hugging Face client from the `HUGGINGFACE_API_KEY` environment variable.
    pub fn from_env() -> Result<Self> {
        let api_key = std::env::var("HUGGINGFACE_API_KEY").map_err(|_| {
            anyhow::anyhow!(
                "HUGGINGFACE_API_KEY environment variable is not set.\n\
                 Set it to your Hugging Face API key:\n\n  \
                 export HUGGINGFACE_API_KEY=\"your-api-key\""
            )
        })?;
        Self::new(&api_key)
    }

    /// Creates a new Hugging Face client with the given API key.
    pub fn new(api_key: &str) -> Result<Self> {
        let client = huggingface::Client::new(api_key)
            .map_err(|e| anyhow::anyhow!("failed to create Hugging Face client: {e}"))?;
        Ok(Self { client })
    }

    /// Returns a completion model handle for the specified model ID.
    pub fn completion_model(&self, model_id: &str) -> huggingface::completion::CompletionModel {
        CompletionClient::completion_model(&self.client, model_id)
    }
}
