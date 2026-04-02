use anyhow::Result;
use rig::client::CompletionClient;
use rig::providers::llamafile;

/// Default llamafile server URL.
const DEFAULT_BASE_URL: &str = "http://localhost:8080";

/// Wrapper around the `rig::providers::llamafile::Client` for local llamafile access.
pub struct LlamafileClient {
    client: llamafile::Client,
}

impl LlamafileClient {
    /// Creates a new llamafile client.
    ///
    /// Uses `LLAMAFILE_API_BASE_URL` if set, otherwise defaults to `http://localhost:8080`.
    pub fn from_env() -> Result<Self> {
        let base_url =
            std::env::var("LLAMAFILE_API_BASE_URL").unwrap_or_else(|_| DEFAULT_BASE_URL.to_string());
        let client = llamafile::Client::from_url(&base_url);
        Ok(Self { client })
    }

    /// Returns a completion model handle for the specified model ID.
    pub fn completion_model(&self, model_id: &str) -> llamafile::CompletionModel {
        CompletionClient::completion_model(&self.client, model_id)
    }
}
