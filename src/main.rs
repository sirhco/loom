use std::sync::Arc;

use anyhow::Result;
use clap::Parser;
use tokio::sync::RwLock;

use loom::cli::args::{CliArgs, ProviderChoice};
use loom::config::paths;
use loom::config::settings::Settings;
use loom::engine::context::build_system_prompt;
use loom::engine::query_engine::QueryEngine;
use loom::permissions::mode::PermissionMode;
use loom::provider::Provider;
use loom::provider::anthropic::AnthropicClient;
use loom::provider::auth::AuthConfig;
use loom::provider::azure::AzureClient;
use loom::provider::cohere::CohereClient;
use loom::provider::deepseek::DeepSeekClient;
use loom::provider::galadriel::GaladrielClient;
use loom::provider::gemini::GeminiClient;
use loom::provider::groq::GroqClient;
use loom::provider::huggingface::HuggingFaceClient;
use loom::provider::hyperbolic::HyperbolicClient;
use loom::provider::llamafile::LlamafileClient;
use loom::provider::mira::MiraClient;
use loom::provider::mistral::MistralClient;
use loom::provider::moonshot::MoonshotClient;
use loom::provider::ollama::OllamaClient;
use loom::provider::openai::OpenAIClient;
use loom::provider::openrouter::OpenRouterClient;
use loom::provider::perplexity::PerplexityClient;
use loom::provider::together::TogetherClient;
use loom::provider::vertex::VertexClient;
use loom::provider::xai::XAIClient;
use loom::state::app_state::AppState;
use loom::tools::registry::ToolRegistry;
use loom::tui::app::run_interactive;
use loom::tui::renderer;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize color-eyre for nice error reports
    color_eyre::install().ok();

    // Initialize tracing from LOOM_LOG env var
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_env("LOOM_LOG")
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("warn")),
        )
        .with_target(false)
        .init();

    // Parse CLI arguments
    let args = CliArgs::parse();

    // Ensure config directories exist
    paths::ensure_dirs()?;

    // Load settings
    let settings = Settings::load().unwrap_or_else(|err| {
        tracing::warn!("Failed to load settings: {err}, using defaults");
        Settings::default()
    });

    // Set working directory
    if let Some(ref dir) = args.directory {
        std::env::set_current_dir(dir)?;
    }

    // Resolve permission mode
    let _permission_mode = if args.dangerously_skip_permissions {
        PermissionMode::Bypass
    } else if args.auto_approve {
        PermissionMode::Auto
    } else {
        args.permission_mode
    };

    let cwd = std::env::current_dir()?;

    // Handle --dump-system-prompt early (no auth needed)
    if args.dump_system_prompt {
        let system_prompt = build_system_prompt(
            &cwd,
            &args.model,
            args.system_prompt.as_deref(),
            args.append_system_prompt.as_deref(),
        )
        .await;
        println!("{system_prompt}");
        return Ok(());
    }

    // Create provider based on --provider flag
    let provider = match args.provider {
        ProviderChoice::Gemini => {
            let client = if let Some(ref api_key) = args.api_key {
                GeminiClient::new(api_key)?
            } else {
                GeminiClient::from_env()?
            };
            Provider::Gemini(client)
        }
        ProviderChoice::Vertex => {
            let auth = AuthConfig::from_env()?;
            let client = VertexClient::new(&auth)?;
            Provider::Vertex(client)
        }
        ProviderChoice::Anthropic => {
            let client = if let Some(ref api_key) = args.api_key {
                AnthropicClient::new(api_key)?
            } else {
                AnthropicClient::from_env()?
            };
            Provider::Anthropic(client)
        }
        ProviderChoice::OpenAI => {
            let client = if let Some(ref api_key) = args.api_key {
                OpenAIClient::new(api_key)?
            } else {
                OpenAIClient::from_env()?
            };
            Provider::OpenAI(client)
        }
        ProviderChoice::Azure => {
            let client = AzureClient::from_env()?;
            Provider::Azure(client)
        }
        ProviderChoice::Cohere => {
            let client = if let Some(ref api_key) = args.api_key {
                CohereClient::new(api_key)?
            } else {
                CohereClient::from_env()?
            };
            Provider::Cohere(client)
        }
        ProviderChoice::DeepSeek => {
            let client = if let Some(ref api_key) = args.api_key {
                DeepSeekClient::new(api_key)?
            } else {
                DeepSeekClient::from_env()?
            };
            Provider::DeepSeek(client)
        }
        ProviderChoice::Galadriel => {
            let client = if let Some(ref api_key) = args.api_key {
                GaladrielClient::new(api_key)?
            } else {
                GaladrielClient::from_env()?
            };
            Provider::Galadriel(client)
        }
        ProviderChoice::Groq => {
            let client = if let Some(ref api_key) = args.api_key {
                GroqClient::new(api_key)?
            } else {
                GroqClient::from_env()?
            };
            Provider::Groq(client)
        }
        ProviderChoice::HuggingFace => {
            let client = if let Some(ref api_key) = args.api_key {
                HuggingFaceClient::new(api_key)?
            } else {
                HuggingFaceClient::from_env()?
            };
            Provider::HuggingFace(client)
        }
        ProviderChoice::Hyperbolic => {
            let client = if let Some(ref api_key) = args.api_key {
                HyperbolicClient::new(api_key)?
            } else {
                HyperbolicClient::from_env()?
            };
            Provider::Hyperbolic(client)
        }
        ProviderChoice::Llamafile => {
            let client = LlamafileClient::from_env()?;
            Provider::Llamafile(client)
        }
        ProviderChoice::Mira => {
            let client = if let Some(ref api_key) = args.api_key {
                MiraClient::new(api_key)?
            } else {
                MiraClient::from_env()?
            };
            Provider::Mira(client)
        }
        ProviderChoice::Mistral => {
            let client = if let Some(ref api_key) = args.api_key {
                MistralClient::new(api_key)?
            } else {
                MistralClient::from_env()?
            };
            Provider::Mistral(client)
        }
        ProviderChoice::Moonshot => {
            let client = if let Some(ref api_key) = args.api_key {
                MoonshotClient::new(api_key)?
            } else {
                MoonshotClient::from_env()?
            };
            Provider::Moonshot(client)
        }
        ProviderChoice::Ollama => {
            let client = OllamaClient::from_env()?;
            Provider::Ollama(client)
        }
        ProviderChoice::OpenRouter => {
            let client = if let Some(ref api_key) = args.api_key {
                OpenRouterClient::new(api_key)?
            } else {
                OpenRouterClient::from_env()?
            };
            Provider::OpenRouter(client)
        }
        ProviderChoice::Perplexity => {
            let client = if let Some(ref api_key) = args.api_key {
                PerplexityClient::new(api_key)?
            } else {
                PerplexityClient::from_env()?
            };
            Provider::Perplexity(client)
        }
        ProviderChoice::Together => {
            let client = if let Some(ref api_key) = args.api_key {
                TogetherClient::new(api_key)?
            } else {
                TogetherClient::from_env()?
            };
            Provider::Together(client)
        }
        ProviderChoice::XAI => {
            let client = if let Some(ref api_key) = args.api_key {
                XAIClient::new(api_key)?
            } else {
                XAIClient::from_env()?
            };
            Provider::XAI(client)
        }
    };

    tracing::info!("Using provider: {}", provider.name());

    // Build initial state
    let state = Arc::new(RwLock::new(AppState::new(&args, settings)));

    // Build system prompt
    let system_prompt = build_system_prompt(
        &cwd,
        &args.model,
        args.system_prompt.as_deref(),
        args.append_system_prompt.as_deref(),
    )
    .await;

    // Save provider name before moving it
    let provider_name = provider.name();

    // Create tool registry
    let tool_registry = ToolRegistry::new(state.clone());

    // Create query engine
    let mut engine = QueryEngine::new(
        provider,
        args.model.clone(),
        tool_registry,
        state.clone(),
        system_prompt,
    );

    // Non-interactive mode: single prompt
    if let Some(prompt) = args.prompt {
        match engine.query(&prompt).await {
            Ok(response) => {
                renderer::render_assistant_message(&response);
            }
            Err(err) => {
                renderer::render_error(&format!("{err}"));
                std::process::exit(1);
            }
        }
        return Ok(());
    }

    // Interactive mode: TUI
    run_interactive(engine, state, provider_name).await?;

    Ok(())
}
