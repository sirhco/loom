use std::env;

use anyhow::{Context, Result, bail};

/// GCP authentication configuration for Vertex AI.
#[derive(Debug, Clone)]
pub struct AuthConfig {
    /// Google Cloud project ID.
    pub project_id: String,
    /// Google Cloud region (e.g. "us-central1").
    pub location: String,
}

impl AuthConfig {
    /// Reads authentication configuration from environment variables.
    ///
    /// - `GOOGLE_CLOUD_PROJECT` — required, the GCP project ID.
    /// - `GOOGLE_CLOUD_LOCATION` — optional, defaults to `"us-central1"`.
    pub fn from_env() -> Result<Self> {
        let project_id = env::var("GOOGLE_CLOUD_PROJECT").context(
            "GOOGLE_CLOUD_PROJECT environment variable is not set.\n\
             Please set it to your Google Cloud project ID:\n\
             \n  export GOOGLE_CLOUD_PROJECT=\"your-project-id\"\n\
             \nYou can find your project ID at https://console.cloud.google.com/",
        )?;

        if project_id.is_empty() {
            bail!(
                "GOOGLE_CLOUD_PROJECT is set but empty. \
                 Please provide a valid Google Cloud project ID."
            );
        }

        // Default to "global" endpoint for better availability and lower latency.
        // Regional endpoints (us-central1, etc.) can be set via GOOGLE_CLOUD_LOCATION.
        let location =
            env::var("GOOGLE_CLOUD_LOCATION").unwrap_or_else(|_| "global".to_string());

        Ok(Self {
            project_id,
            location,
        })
    }
}
