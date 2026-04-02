/// Configuration for streaming responses.
///
/// Currently a placeholder — the MVP uses non-streaming completions since
/// rig-vertexai does not yet support streaming.
pub struct StreamConfig {
    pub enabled: bool,
}

impl Default for StreamConfig {
    fn default() -> Self {
        Self { enabled: false }
    }
}
