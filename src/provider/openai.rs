use anyhow::Result;
use rig::client::CompletionClient;
use rig::providers::openai;

/// Wrapper around the `rig::providers::openai::Client` for OpenAI API access.
///
/// Uses the OpenAI Responses API by default (the rig-core 0.34 default).
pub struct OpenAIClient {
    client: openai::Client,
}

impl OpenAIClient {
    /// Creates a new OpenAI client from the `OPENAI_API_KEY` environment variable.
    pub fn from_env() -> Result<Self> {
        let api_key = std::env::var("OPENAI_API_KEY").map_err(|_| {
            anyhow::anyhow!(
                "OPENAI_API_KEY environment variable is not set.\n\
                 Set it to your OpenAI API key:\n\n  \
                 export OPENAI_API_KEY=\"your-api-key\""
            )
        })?;
        Self::new(&api_key)
    }

    /// Creates a new OpenAI client with the given API key.
    pub fn new(api_key: &str) -> Result<Self> {
        let mut builder = openai::Client::builder().api_key(api_key);
        if let Ok(base_url) = std::env::var("OPENAI_BASE_URL") {
            builder = builder.base_url(&base_url);
        }
        let client = builder
            .build()
            .map_err(|e| anyhow::anyhow!("failed to create OpenAI client: {e}"))?;
        Ok(Self { client })
    }

    /// Returns a completion model handle for the specified model ID.
    pub fn completion_model(
        &self,
        model_id: &str,
    ) -> openai::responses_api::ResponsesCompletionModel {
        CompletionClient::completion_model(&self.client, model_id)
    }
}
