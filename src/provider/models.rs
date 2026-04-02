use std::fmt;
use std::str::FromStr;

use clap::ValueEnum;

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum GeminiModel {
    /// Gemini 2.5 Pro
    #[value(name = "gemini-2.5-pro")]
    Gemini25Pro,
    /// Gemini 2.5 Flash
    #[value(name = "gemini-2.5-flash")]
    Gemini25Flash,
    /// Gemini 2.5 Flash-Lite
    #[value(name = "gemini-2.5-flash-lite")]
    Gemini25FlashLite,
    /// Gemini 3.1 Pro Preview
    #[value(name = "gemini-3.1-pro-preview")]
    Gemini31ProPreview,
    /// Gemini 3.1 Flash-Lite Preview
    #[value(name = "gemini-3.1-flash-lite-preview")]
    Gemini31FlashLitePreview,
}

impl GeminiModel {
    /// Returns all available model variants.
    pub fn all() -> &'static [GeminiModel] {
        &[
            GeminiModel::Gemini25Pro,
            GeminiModel::Gemini25Flash,
            GeminiModel::Gemini25FlashLite,
            GeminiModel::Gemini31ProPreview,
            GeminiModel::Gemini31FlashLitePreview,
        ]
    }

    /// Returns a display string suitable for the model selector popup.
    pub fn selector_label(&self) -> String {
        if self.has_tool_calling_issues() {
            format!("{} ({}) [limited tool support]", self.display_name(), self.model_id())
        } else {
            format!("{} ({})", self.display_name(), self.model_id())
        }
    }

    /// Parses a selector label back into a GeminiModel.
    pub fn from_selector_label(label: &str) -> Option<GeminiModel> {
        GeminiModel::all()
            .iter()
            .find(|m| label.contains(m.model_id()))
            .copied()
    }
}

impl Default for GeminiModel {
    fn default() -> Self {
        Self::Gemini25Pro
    }
}

impl GeminiModel {
    /// Returns the API model identifier string.
    pub fn model_id(&self) -> &str {
        match self {
            Self::Gemini25Pro => "gemini-2.5-pro",
            Self::Gemini25Flash => "gemini-2.5-flash",
            Self::Gemini25FlashLite => "gemini-2.5-flash-lite",
            Self::Gemini31ProPreview => "gemini-3.1-pro-preview",
            Self::Gemini31FlashLitePreview => "gemini-3.1-flash-lite-preview",
        }
    }

    /// Returns a human-readable display name.
    pub fn display_name(&self) -> &str {
        match self {
            Self::Gemini25Pro => "Gemini 2.5 Pro",
            Self::Gemini25Flash => "Gemini 2.5 Flash",
            Self::Gemini25FlashLite => "Gemini 2.5 Flash-Lite",
            Self::Gemini31ProPreview => "Gemini 3.1 Pro Preview",
            Self::Gemini31FlashLitePreview => "Gemini 3.1 Flash-Lite Preview",
        }
    }

    /// Returns the context window size in tokens.
    pub fn context_window(&self) -> usize {
        match self {
            Self::Gemini25Pro => 1_000_000,
            Self::Gemini25Flash => 1_000_000,
            Self::Gemini25FlashLite => 1_000_000,
            Self::Gemini31ProPreview => 1_000_000,
            Self::Gemini31FlashLitePreview => 1_000_000,
        }
    }

    /// Returns the maximum output tokens.
    pub fn max_output_tokens(&self) -> usize {
        match self {
            Self::Gemini25Pro => 65_536,
            Self::Gemini25Flash => 65_536,
            Self::Gemini25FlashLite => 65_536,
            Self::Gemini31ProPreview => 65_536,
            Self::Gemini31FlashLitePreview => 65_536,
        }
    }

    /// Returns approximate input cost per million tokens in USD.
    pub fn input_cost_per_million(&self) -> f64 {
        match self {
            Self::Gemini25Pro => 1.25,
            Self::Gemini25Flash => 0.15,
            Self::Gemini25FlashLite => 0.075,
            Self::Gemini31ProPreview => 1.50,
            Self::Gemini31FlashLitePreview => 0.10,
        }
    }

    /// Returns approximate output cost per million tokens in USD.
    pub fn output_cost_per_million(&self) -> f64 {
        match self {
            Self::Gemini25Pro => 10.00,
            Self::Gemini25Flash => 0.60,
            Self::Gemini25FlashLite => 0.30,
            Self::Gemini31ProPreview => 12.00,
            Self::Gemini31FlashLitePreview => 0.40,
        }
    }

    /// Whether this model supports extended thinking / chain-of-thought.
    pub fn supports_thinking(&self) -> bool {
        match self {
            Self::Gemini25Pro | Self::Gemini31ProPreview => true,
            Self::Gemini25Flash | Self::Gemini25FlashLite | Self::Gemini31FlashLitePreview => false,
        }
    }

    /// Whether this model has known compatibility issues with rig-vertexai.
    ///
    /// Gemini 3.1 Preview models require thought_signature support in function
    /// calling which rig-vertexai 0.3.x does not yet implement.
    pub fn has_tool_calling_issues(&self) -> bool {
        matches!(
            self,
            Self::Gemini31ProPreview | Self::Gemini31FlashLitePreview
        )
    }
}

impl fmt::Display for GeminiModel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.model_id())
    }
}

impl FromStr for GeminiModel {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "gemini-2.5-pro" => Ok(Self::Gemini25Pro),
            "gemini-2.5-flash" => Ok(Self::Gemini25Flash),
            "gemini-2.5-flash-lite" => Ok(Self::Gemini25FlashLite),
            "gemini-3.1-pro-preview" => Ok(Self::Gemini31ProPreview),
            "gemini-3.1-flash-lite-preview" => Ok(Self::Gemini31FlashLitePreview),
            _ => Err(format!(
                "unknown model '{}'; expected one of: gemini-2.5-pro, gemini-2.5-flash, \
                 gemini-2.5-flash-lite, gemini-3.1-pro-preview, gemini-3.1-flash-lite-preview",
                s
            )),
        }
    }
}
