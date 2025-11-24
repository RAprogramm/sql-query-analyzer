//! Configuration loading and management.
//!
//! Configuration is loaded from multiple sources with the following precedence
//! (highest to lowest):
//!
//! 1. Command-line arguments
//! 2. Environment variables
//! 3. `.sql-analyzer.toml` in current directory
//! 4. `~/.config/sql-analyzer/config.toml`
//! 5. Default values
//!
//! # Configuration File Format
//!
//! ```toml
//! [llm]
//! provider = "ollama"          # openai, anthropic, ollama
//! model = "llama3.2"
//! api_key = "sk-..."           # or use LLM_API_KEY env var
//! ollama_url = "http://localhost:11434"
//!
//! [retry]
//! max_retries = 3
//! initial_delay_ms = 1000
//! max_delay_ms = 30000
//! backoff_factor = 2.0
//!
//! [rules]
//! disabled = ["STYLE001", "PERF011"]
//!
//! [rules.severity]
//! PERF001 = "error"
//! SCHEMA001 = "info"
//! ```
//!
//! # Environment Variables
//!
//! | Variable | Description |
//! |----------|-------------|
//! | `LLM_API_KEY` | API key for OpenAI/Anthropic |
//! | `LLM_PROVIDER` | Provider name |
//! | `LLM_MODEL` | Model identifier |
//! | `OLLAMA_URL` | Ollama base URL |

use std::{collections::HashMap, env, fs, path::PathBuf};

use serde::Deserialize;

use crate::error::{AppResult, config_error};

/// Application configuration
#[derive(Debug, Clone, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub llm:   LlmConfig,
    #[serde(default)]
    pub retry: RetryConfig,
    #[serde(default)]
    pub rules: RulesConfig
}

/// Rules configuration
#[derive(Debug, Clone, Deserialize, Default)]
pub struct RulesConfig {
    /// Disabled rule IDs
    #[serde(default)]
    pub disabled: Vec<String>,
    /// Severity overrides (rule_id -> severity)
    #[serde(default)]
    pub severity: HashMap<String, String>
}

/// LLM provider configuration
#[derive(Debug, Clone, Deserialize)]
pub struct LlmConfig {
    pub provider:   Option<String>,
    pub api_key:    Option<String>,
    pub model:      Option<String>,
    pub ollama_url: Option<String>
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            provider:   None,
            api_key:    None,
            model:      None,
            ollama_url: Some(String::from("http://localhost:11434"))
        }
    }
}

/// Retry configuration for LLM requests
#[derive(Debug, Clone, Deserialize)]
pub struct RetryConfig {
    pub max_retries:      u32,
    pub initial_delay_ms: u64,
    pub max_delay_ms:     u64,
    pub backoff_factor:   f64
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries:      3,
            initial_delay_ms: 1000,
            max_delay_ms:     30000,
            backoff_factor:   2.0
        }
    }
}

impl Config {
    /// Load configuration from file and environment
    ///
    /// Priority (highest to lowest):
    /// 1. Environment variables
    /// 2. Config file in current directory (.sql-analyzer.toml)
    /// 3. Config file in home directory (~/.config/sql-analyzer/config.toml)
    /// 4. Default values
    pub fn load() -> AppResult<Self> {
        let mut config = Self::default();

        // Try to load from home directory config
        if let Some(home) = env::var_os("HOME") {
            let home_config = PathBuf::from(home)
                .join(".config")
                .join("sql-analyzer")
                .join("config.toml");

            if home_config.exists() {
                let content = fs::read_to_string(&home_config)
                    .map_err(|e| config_error(format!("Failed to read config file: {}", e)))?;
                config = toml::from_str(&content)
                    .map_err(|e| config_error(format!("Invalid config file: {}", e)))?;
            }
        }

        // Try to load from current directory config (overrides home config)
        let local_config = PathBuf::from(".sql-analyzer.toml");
        if local_config.exists() {
            let content = fs::read_to_string(&local_config)
                .map_err(|e| config_error(format!("Failed to read config file: {}", e)))?;
            config = toml::from_str(&content)
                .map_err(|e| config_error(format!("Invalid config file: {}", e)))?;
        }

        // Override with environment variables
        if let Ok(api_key) = env::var("LLM_API_KEY") {
            config.llm.api_key = Some(api_key);
        }

        if let Ok(provider) = env::var("LLM_PROVIDER") {
            config.llm.provider = Some(provider);
        }

        if let Ok(model) = env::var("LLM_MODEL") {
            config.llm.model = Some(model);
        }

        if let Ok(url) = env::var("OLLAMA_URL") {
            config.llm.ollama_url = Some(url);
        }

        Ok(config)
    }
}
