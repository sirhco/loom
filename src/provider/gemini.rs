use anyhow::Result;
use rig::client::CompletionClient;
use rig::providers::gemini;

/// Wrapper around the `rig::providers::gemini::Client` for direct Gemini API access.
pub struct GeminiClient {
    client: gemini::Client,
}

impl GeminiClient {
    /// Creates a new Gemini client with the given API key.
    pub fn new(api_key: &str) -> Result<Self> {
        let client = gemini::Client::new(api_key)
            .map_err(|e| anyhow::anyhow!("failed to create Gemini client: {e}"))?;
        Ok(Self { client })
    }

    /// Creates a new Gemini client from the `GEMINI_API_KEY` environment variable.
    pub fn from_env() -> Result<Self> {
        let api_key = std::env::var("GEMINI_API_KEY").map_err(|_| {
            anyhow::anyhow!(
                "GEMINI_API_KEY environment variable is not set.\n\
                 Set it to your Gemini API key:\n\n  \
                 export GEMINI_API_KEY=\"your-api-key\"\n\n\
                 Get an API key at https://aistudio.google.com/apikey"
            )
        })?;
        Self::new(&api_key)
    }

    /// Returns a completion model handle for the specified model ID.
    pub fn completion_model(
        &self,
        model_id: &str,
    ) -> gemini::completion::CompletionModel {
        CompletionClient::completion_model(&self.client, model_id)
    }
}
