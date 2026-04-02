use anyhow::Result;
use rig::client::CompletionClient;
use rig::providers::groq;

/// Wrapper around the `rig::providers::groq::Client` for Groq API access.
pub struct GroqClient {
    client: groq::Client,
}

impl GroqClient {
    /// Creates a new Groq client from the `GROQ_API_KEY` environment variable.
    pub fn from_env() -> Result<Self> {
        let api_key = std::env::var("GROQ_API_KEY").map_err(|_| {
            anyhow::anyhow!(
                "GROQ_API_KEY environment variable is not set.\n\
                 Set it to your Groq API key:\n\n  \
                 export GROQ_API_KEY=\"your-api-key\""
            )
        })?;
        Self::new(&api_key)
    }

    /// Creates a new Groq client with the given API key.
    pub fn new(api_key: &str) -> Result<Self> {
        let client = groq::Client::new(api_key)
            .map_err(|e| anyhow::anyhow!("failed to create Groq client: {e}"))?;
        Ok(Self { client })
    }

    /// Returns a completion model handle for the specified model ID.
    pub fn completion_model(&self, model_id: &str) -> groq::CompletionModel {
        CompletionClient::completion_model(&self.client, model_id)
    }
}
