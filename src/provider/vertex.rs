use anyhow::{Result, anyhow};
use rig::client::CompletionClient;

use crate::provider::auth::AuthConfig;

/// Wrapper around the `rig_vertexai::Client` for Vertex AI interactions.
pub struct VertexClient {
    client: rig_vertexai::Client,
}

impl VertexClient {
    /// Creates a new Vertex AI client from the given authentication config.
    pub fn new(auth: &AuthConfig) -> Result<Self> {
        let client = rig_vertexai::ClientBuilder::new()
            .with_project(&auth.project_id)
            .with_location(&auth.location)
            .build()
            .map_err(|e| anyhow!("failed to create Vertex AI client: {e}"))?;

        Ok(Self { client })
    }

    /// Returns a completion model handle for the specified model ID.
    pub fn completion_model(&self, model_id: &str) -> rig_vertexai::completion::CompletionModel {
        CompletionClient::completion_model(&self.client, model_id)
    }
}
