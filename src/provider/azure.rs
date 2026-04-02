use anyhow::Result;
use rig::client::CompletionClient;
use rig::providers::azure;

/// Wrapper around the `rig::providers::azure::Client` for Azure OpenAI access.
pub struct AzureClient {
    client: azure::Client,
}

impl AzureClient {
    /// Creates a new Azure OpenAI client from environment variables.
    ///
    /// Requires `AZURE_API_KEY` (or `AZURE_TOKEN`), `AZURE_API_VERSION`, and `AZURE_ENDPOINT`.
    pub fn from_env() -> Result<Self> {
        let auth = if let Ok(api_key) = std::env::var("AZURE_API_KEY") {
            azure::AzureOpenAIAuth::ApiKey(api_key)
        } else if let Ok(token) = std::env::var("AZURE_TOKEN") {
            azure::AzureOpenAIAuth::Token(token)
        } else {
            return Err(anyhow::anyhow!(
                "Neither AZURE_API_KEY nor AZURE_TOKEN is set.\n\
                 Set one of these environment variables for Azure OpenAI access."
            ));
        };

        let api_version = std::env::var("AZURE_API_VERSION").map_err(|_| {
            anyhow::anyhow!("AZURE_API_VERSION environment variable is not set.")
        })?;
        let endpoint = std::env::var("AZURE_ENDPOINT").map_err(|_| {
            anyhow::anyhow!("AZURE_ENDPOINT environment variable is not set.")
        })?;

        let client = azure::Client::builder()
            .api_key(auth)
            .azure_endpoint(endpoint)
            .api_version(&api_version)
            .build()
            .map_err(|e| anyhow::anyhow!("failed to create Azure OpenAI client: {e}"))?;

        Ok(Self { client })
    }

    /// Returns a completion model handle for the specified model/deployment ID.
    pub fn completion_model(&self, model_id: &str) -> azure::CompletionModel {
        CompletionClient::completion_model(&self.client, model_id)
    }
}
