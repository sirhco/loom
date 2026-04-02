# Loom

Loom is an AI-powered coding assistant for the terminal — built in Rust with a rich interactive TUI for multi-turn conversations, tool use, and agentic workflows.

It supports **20+ AI providers** out of the box (Gemini, Vertex AI, Anthropic, OpenAI, Ollama, and more), giving you flexibility to use whichever model fits your workflow.

## Features

- **Interactive TUI** — Full terminal UI built with ratatui, featuring markdown rendering, syntax highlighting, autocomplete, and a conversation history viewer
- **Multi-provider support** — Seamlessly switch between Gemini, Vertex AI, Anthropic (Claude), OpenAI, Azure, Groq, Mistral, Ollama, DeepSeek, and many more
- **Agentic tool use** — The AI can read/write/edit files, run shell commands, search your codebase, fetch web content, and manage tasks — all in an autonomous loop
- **Permission system** — Configurable permission modes (default, auto, bypass) control which operations require approval
- **Planning mode** — A research-first workflow where the AI uses read-only tools to analyze before proposing changes
- **Cost tracking** — Real-time token usage and cost estimates displayed in the status bar
- **Context compaction** — Automatically compacts conversation history to stay within context limits during long sessions
- **Git-aware** — Understands your repository structure and respects `.gitignore` for file operations
- **Configurable** — Settings persisted in `~/.loom/settings.toml` with per-project overrides

## Installation

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (edition 2024)

### Build from source

```bash
git clone https://github.com/your-org/loom.git
cd loom
cargo build --release
```

The binary will be at `target/release/loom`.

## Quick Start

### 1. Set up a provider

Loom defaults to **Gemini**. Export an API key for your chosen provider:

```bash
# Gemini (default)
export GEMINI_API_KEY="your-key"

# Or use Vertex AI (Google Cloud)
export GOOGLE_CLOUD_PROJECT="your-project-id"
export GOOGLE_CLOUD_LOCATION="us-central1"  # optional, defaults to "global"

# Or use Anthropic
export ANTHROPIC_API_KEY="your-key"

# Or use OpenAI
export OPENAI_API_KEY="your-key"

# Or use a local model via Ollama (no key needed)
```

### 2. Launch the TUI

```bash
loom
```

### 3. Or run a one-shot query

```bash
loom --prompt "Explain the main function in src/main.rs"
```

## Usage

### CLI Options

```
loom [OPTIONS]

Options:
      --prompt <PROMPT>              Run a single query non-interactively
      --model <MODEL>                Model ID [default: gemini-2.5-pro]
      --provider <PROVIDER>          AI provider backend [default: gemini]
      --api-key <KEY>                Override the provider API key
      --permission-mode <MODE>       Permission mode: default, auto, bypass
      --auto-approve                 Auto-approve safe operations
      --directory <DIR>              Set the working directory
      --max-tokens <N>               Max output tokens
      --system-prompt <PROMPT>       Custom system prompt
      --append-system-prompt <TEXT>  Append to the default system prompt
      --dump-system-prompt           Print the system prompt and exit
      --verbose                      Verbose output
```

### Slash Commands

Inside the TUI, use slash commands to control Loom:

| Command    | Description                          |
|------------|--------------------------------------|
| `/help`    | Show available commands              |
| `/clear`   | Clear the conversation               |
| `/compact` | Compact conversation context         |
| `/config`  | View or update settings              |
| `/cost`    | Show token usage and cost summary    |
| `/commit`  | Generate a git commit                |
| `/memory`  | View or manage persistent memory     |
| `/model`   | Switch the active model              |
| `/plan`    | Enter planning mode                  |
| `/session` | Manage conversation sessions         |

### Tools

Loom's AI has access to a suite of tools it uses autonomously:

| Tool       | Description                                  |
|------------|----------------------------------------------|
| `bash`     | Execute shell commands (with timeout support) |
| `read`     | Read file contents with line numbers         |
| `write`    | Create or overwrite files                    |
| `edit`     | Search-and-replace editing in files          |
| `glob`     | Find files by glob pattern                   |
| `grep`     | Regex search across files                    |
| `web_fetch`| Fetch and process web page content           |
| `ask_user` | Ask for clarification                        |
| `task_*`   | Create, list, get, and update tasks          |

## Supported Providers

| Provider     | Env Variable(s)                                      | Example Model           |
|--------------|------------------------------------------------------|-------------------------|
| Gemini       | `GEMINI_API_KEY`                                     | `gemini-2.5-pro`        |
| Vertex AI    | `GOOGLE_CLOUD_PROJECT`, `GOOGLE_CLOUD_LOCATION`      | `gemini-2.5-pro`        |
| Anthropic    | `ANTHROPIC_API_KEY`                                  | `claude-sonnet-4-6`    |
| OpenAI       | `OPENAI_API_KEY`                                     | `gpt-4o`               |
| Azure OpenAI | `AZURE_API_KEY`, `AZURE_ENDPOINT`, `AZURE_API_VERSION`| `gpt-4o`              |
| Ollama       | *(none — runs locally)*                              | `llama3`               |
| Groq         | `GROQ_API_KEY`                                       | `llama-3.3-70b`        |
| DeepSeek     | `DEEPSEEK_API_KEY`                                   | `deepseek-chat`        |
| Mistral      | `MISTRAL_API_KEY`                                    | `mistral-large-latest` |
| OpenRouter   | `OPENROUTER_API_KEY`                                 | *(any supported model)* |
| Together     | `TOGETHER_API_KEY`                                   | *(any supported model)* |
| Perplexity   | `PERPLEXITY_API_KEY`                                 | `sonar-pro`            |
| xAI          | `XAI_API_KEY`                                        | `grok-3`               |
| HuggingFace  | `HUGGINGFACE_API_KEY`                                | *(any supported model)* |
| Cohere       | `COHERE_API_KEY`                                     | `command-r-plus`       |
| Llamafile    | *(none — runs locally)*                              | *(local model)*        |

## Configuration

Loom stores configuration in `~/.loom/`:

```
~/.loom/
├── settings.toml    # Global settings
├── sessions/        # Conversation session history
├── plans/           # Saved plans
└── LOOM.md          # Persistent memory
```

### Example `settings.toml`

```toml
model = "gemini-2.5-pro"
verbose = false
permission_mode = "default"

[permissions]
allow_read = ["**/*"]
deny_write = [".env", "*.pem"]
allow_run = ["cargo *", "git *", "npm *"]

[theme]
color_scheme = "dark"
```

## Architecture

Loom follows a modular architecture with clear separation of concerns:

```
┌─────────────────────────────────────────────┐
│                   TUI Layer                  │
│        (ratatui + crossterm + events)        │
├─────────────────────────────────────────────┤
│                 Engine Layer                 │
│  (QueryEngine → message history → AI loop)  │
├──────────────────┬──────────────────────────┤
│  Provider Layer  │       Tools Layer        │
│  (20+ backends)  │  (file, bash, grep, ...) │
├──────────────────┴──────────────────────────┤
│              State + Config                  │
│    (AppState, Settings, Permissions)         │
└─────────────────────────────────────────────┘
```

The core loop: user submits a prompt → engine sends it to the AI → AI responds with text and/or tool calls → tools execute → results feed back to the AI → loop until complete.

## Development

```bash
# Run in development
cargo run

# Run tests
cargo test

# Build release binary
cargo build --release

# Enable debug logging
LOOM_LOG=debug cargo run
```

## License

[MIT](LICENSE)
