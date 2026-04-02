pub mod anthropic;
pub mod auth;
pub mod azure;
pub mod cohere;
pub mod deepseek;
pub mod galadriel;
pub mod gemini;
pub mod groq;
pub mod huggingface;
pub mod hyperbolic;
pub mod llamafile;
pub mod mira;
pub mod mistral;
pub mod models;
pub mod moonshot;
pub mod ollama;
pub mod openai;
pub mod openrouter;
pub mod perplexity;
pub mod together;
pub mod vertex;
pub mod xai;

use crate::provider::anthropic::AnthropicClient;
use crate::provider::azure::AzureClient;
use crate::provider::cohere::CohereClient;
use crate::provider::deepseek::DeepSeekClient;
use crate::provider::galadriel::GaladrielClient;
use crate::provider::gemini::GeminiClient;
use crate::provider::groq::GroqClient;
use crate::provider::huggingface::HuggingFaceClient;
use crate::provider::hyperbolic::HyperbolicClient;
use crate::provider::llamafile::LlamafileClient;
use crate::provider::mira::MiraClient;
use crate::provider::mistral::MistralClient;
use crate::provider::moonshot::MoonshotClient;
use crate::provider::ollama::OllamaClient;
use crate::provider::openai::OpenAIClient;
use crate::provider::openrouter::OpenRouterClient;
use crate::provider::perplexity::PerplexityClient;
use crate::provider::together::TogetherClient;
use crate::provider::vertex::VertexClient;
use crate::provider::xai::XAIClient;

/// Unified provider enum supporting all rig-core providers.
pub enum Provider {
    /// Direct Gemini API access via API key.
    Gemini(GeminiClient),
    /// Vertex AI access via GCP service account credentials.
    Vertex(VertexClient),
    /// Anthropic (Claude) API.
    Anthropic(AnthropicClient),
    /// OpenAI API (Responses API).
    OpenAI(OpenAIClient),
    /// Azure OpenAI API.
    Azure(AzureClient),
    /// Cohere API.
    Cohere(CohereClient),
    /// DeepSeek API.
    DeepSeek(DeepSeekClient),
    /// Galadriel API.
    Galadriel(GaladrielClient),
    /// Groq API.
    Groq(GroqClient),
    /// Hugging Face Inference API.
    HuggingFace(HuggingFaceClient),
    /// Hyperbolic API.
    Hyperbolic(HyperbolicClient),
    /// Llamafile local server.
    Llamafile(LlamafileClient),
    /// Mira API.
    Mira(MiraClient),
    /// Mistral API.
    Mistral(MistralClient),
    /// Moonshot API.
    Moonshot(MoonshotClient),
    /// Ollama local server.
    Ollama(OllamaClient),
    /// OpenRouter API.
    OpenRouter(OpenRouterClient),
    /// Perplexity API.
    Perplexity(PerplexityClient),
    /// Together AI API.
    Together(TogetherClient),
    /// xAI / Grok API.
    XAI(XAIClient),
}

impl Provider {
    /// Returns a human-readable name for the active provider.
    pub fn name(&self) -> &'static str {
        match self {
            Provider::Gemini(_) => "Gemini API",
            Provider::Vertex(_) => "Vertex AI",
            Provider::Anthropic(_) => "Anthropic",
            Provider::OpenAI(_) => "OpenAI",
            Provider::Azure(_) => "Azure OpenAI",
            Provider::Cohere(_) => "Cohere",
            Provider::DeepSeek(_) => "DeepSeek",
            Provider::Galadriel(_) => "Galadriel",
            Provider::Groq(_) => "Groq",
            Provider::HuggingFace(_) => "Hugging Face",
            Provider::Hyperbolic(_) => "Hyperbolic",
            Provider::Llamafile(_) => "Llamafile",
            Provider::Mira(_) => "Mira",
            Provider::Mistral(_) => "Mistral",
            Provider::Moonshot(_) => "Moonshot",
            Provider::Ollama(_) => "Ollama",
            Provider::OpenRouter(_) => "OpenRouter",
            Provider::Perplexity(_) => "Perplexity",
            Provider::Together(_) => "Together AI",
            Provider::XAI(_) => "xAI",
        }
    }
}
