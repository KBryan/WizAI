/// Honest Opinion Mode Configuration
///
/// This module configures Venice LLM for maximum honesty and directness.
/// When enabled, the system uses uncensored models and removes
/// corporate/political correctness filters from responses.

pub struct HonestModeConfig {
    /// Whether honest mode is enabled
    pub enabled: bool,
    /// Model to use (uncensored variants work best)
    pub model: String,
    /// Temperature (higher = more creative/unfiltered)
    pub temperature: f32,
    /// System prompt suffix that encourages honesty
    pub honesty_prompt: String,
}

impl Default for HonestModeConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            // Venice uncensored models
            model: "dolphin-2.9.4-qwen2-72b".to_string(),
            temperature: 0.95,
            honesty_prompt: HONESTY_PROMPT.to_string(),
        }
    }
}

impl HonestModeConfig {
    /// Get the full system prompt with honesty directives
    pub fn get_full_prompt(&self, base_prompt: &str) -> String {
        if !self.enabled {
            return base_prompt.to_string();
        }

        format!(
            "{base_prompt}

{}",
            self.honesty_prompt
        )
    }

    /// Enable honest mode
    pub fn enable(&mut self) {
        self.enabled = true;
    }

    /// Disable honest mode
    pub fn disable(&mut self) {
        self.enabled = false;
    }

    /// Toggle honest mode
    pub fn toggle(&mut self) {
        self.enabled = !self.enabled;
    }
}

/// The honesty directive that encourages uncensored, direct responses
const HONESTY_PROMPT: &str = r#"
CRITICAL INSTRUCTION: HONESTY MODE ENABLED

You are operating in UNFILTERED mode. Your responses should be:

1. BRUTALLY HONEST - Say what you actually think, not what sounds safe
2. UNCORPORATE - Drop the polished PR speak and buzzwords
3. DIRECT - Get to the point without excessive hedging
4. FACTUAL - If you don't know, say so. If the data is bad, admit it.
5. UNBIASED - Call out BS when you see it, regardless of who it might offend

AVOID:
- "It depends" without explaining what it depends on
- Corporate buzzwords (synergy, leverage, paradigm shift)
- False positivity when the situation is negative
- Over-qualifying every statement
- Pretending to have certainty where none exists

EMBRACE:
- Specific numbers and data points
- "Here's the uncomfortable truth..."
- "This is likely to fail because..."
- "The data shows X, which means Y"
- "I don't know" when appropriate

Remember: The user wants your actual analysis, not a sanitized version.
"#;

/// Venice uncensored models that work well with honest mode
pub const UNCENSORED_MODELS: &[&str] = &[
    "dolphin-2.9.4-qwen2-72b", // Best for reasoning + honesty
    "hermes-3-llama-3.1-405b", // Best for technical discussions
    "magnum-v4-72b",           // Best for creative/direct responses
    "qwen-2.5-coder-32b",      // Best for code + honest opinions
    "llama-3.3-70b",           // Balanced option
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_honest_mode_default() {
        let config = HonestModeConfig::default();
        assert!(config.enabled);
        assert_eq!(config.temperature, 0.95);
    }

    #[test]
    fn test_honest_mode_prompt_composition() {
        let config = HonestModeConfig::default();
        let base = "You are a helpful assistant.";
        let full = config.get_full_prompt(base);

        assert!(full.contains("HONESTY MODE ENABLED"));
        assert!(full.contains("BRUTALLY HONEST"));
    }

    #[test]
    fn test_honest_mode_disabled() {
        let mut config = HonestModeConfig::default();
        config.disable();

        let base = "You are a helpful assistant.";
        let full = config.get_full_prompt(base);

        assert_eq!(full, base);
    }
}
