use std::path::PathBuf;

use clap::Parser;

use crate::permissions::mode::PermissionMode;

/// Which provider backend to use.
#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum ProviderChoice {
    /// Direct Gemini API (requires GEMINI_API_KEY)
    #[value(name = "gemini")]
    Gemini,
    /// Vertex AI (requires GOOGLE_CLOUD_PROJECT)
    #[value(name = "vertex")]
    Vertex,
    /// Anthropic (requires ANTHROPIC_API_KEY)
    #[value(name = "anthropic")]
    Anthropic,
    /// OpenAI (requires OPENAI_API_KEY)
    #[value(name = "openai")]
    OpenAI,
    /// Azure OpenAI (requires AZURE_API_KEY + AZURE_ENDPOINT + AZURE_API_VERSION)
    #[value(name = "azure")]
    Azure,
    /// Cohere (requires COHERE_API_KEY)
    #[value(name = "cohere")]
    Cohere,
    /// DeepSeek (requires DEEPSEEK_API_KEY)
    #[value(name = "deepseek")]
    DeepSeek,
    /// Galadriel (requires GALADRIEL_API_KEY)
    #[value(name = "galadriel")]
    Galadriel,
    /// Groq (requires GROQ_API_KEY)
    #[value(name = "groq")]
    Groq,
    /// Hugging Face (requires HUGGINGFACE_API_KEY)
    #[value(name = "huggingface")]
    HuggingFace,
    /// Hyperbolic (requires HYPERBOLIC_API_KEY)
    #[value(name = "hyperbolic")]
    Hyperbolic,
    /// Llamafile local server (set LLAMAFILE_API_BASE_URL or default localhost:8080)
    #[value(name = "llamafile")]
    Llamafile,
    /// Mira (requires MIRA_API_KEY)
    #[value(name = "mira")]
    Mira,
    /// Mistral (requires MISTRAL_API_KEY)
    #[value(name = "mistral")]
    Mistral,
    /// Moonshot (requires MOONSHOT_API_KEY)
    #[value(name = "moonshot")]
    Moonshot,
    /// Ollama local server (set OLLAMA_API_BASE_URL or default localhost:11434)
    #[value(name = "ollama")]
    Ollama,
    /// OpenRouter (requires OPENROUTER_API_KEY)
    #[value(name = "openrouter")]
    OpenRouter,
    /// Perplexity (requires PERPLEXITY_API_KEY)
    #[value(name = "perplexity")]
    Perplexity,
    /// Together AI (requires TOGETHER_API_KEY)
    #[value(name = "together")]
    Together,
    /// xAI / Grok (requires XAI_API_KEY)
    #[value(name = "xai")]
    XAI,
}

#[derive(Parser, Debug)]
#[command(
    name = "loom",
    about = "AI-powered coding assistant with multi-provider support",
    version,
    after_help = "Environment variables:\n  \
        GEMINI_API_KEY          Gemini API key (for --provider gemini)\n  \
        GOOGLE_CLOUD_PROJECT    GCP project ID (for --provider vertex)\n  \
        GOOGLE_CLOUD_LOCATION   GCP region (default: us-central1)\n  \
        OPENAI_API_KEY          OpenAI API key (for --provider openai)\n  \
        ANTHROPIC_API_KEY       Anthropic API key (for --provider anthropic)\n  \
        GROQ_API_KEY            Groq API key (for --provider groq)\n  \
        LOOM_LOG                Log level filter (e.g. debug, info)"
)]
pub struct CliArgs {
    /// Initial prompt (non-interactive mode if provided)
    pub prompt: Option<String>,

    /// Model to use for completions (provider-specific, e.g. gemini-2.5-pro, gpt-4o, claude-sonnet-4-20250514)
    #[arg(long, short = 'm', default_value = "gemini-2.5-pro")]
    pub model: String,

    /// Provider backend
    #[arg(long, default_value = "gemini")]
    pub provider: ProviderChoice,

    /// API key (overrides provider-specific env var)
    #[arg(long)]
    pub api_key: Option<String>,

    /// Permission mode
    #[arg(long, default_value = "default")]
    pub permission_mode: PermissionMode,

    /// Enable verbose output
    #[arg(long, short = 'v')]
    pub verbose: bool,

    /// Auto-approve safe operations
    #[arg(long)]
    pub auto_approve: bool,

    /// Working directory
    #[arg(long, short = 'd')]
    pub directory: Option<PathBuf>,

    /// Maximum tokens for model output
    #[arg(long)]
    pub max_tokens: Option<u64>,

    /// Custom system prompt (overrides default)
    #[arg(long)]
    pub system_prompt: Option<String>,

    /// Append text to the system prompt
    #[arg(long)]
    pub append_system_prompt: Option<String>,

    /// Print the system prompt and exit
    #[arg(long)]
    pub dump_system_prompt: bool,

    /// Dangerously skip all permission checks
    #[arg(long)]
    pub dangerously_skip_permissions: bool,
}
