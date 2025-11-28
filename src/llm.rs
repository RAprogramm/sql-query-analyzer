//! LLM provider integrations for AI-powered query analysis.
//!
//! This module provides a unified interface for interacting with multiple LLM
//! providers. It handles authentication, request formatting, response parsing,
//! and automatic retry with exponential backoff.
//!
//! # Supported Providers
//!
//! | Provider | Endpoint | Authentication |
//! |----------|----------|----------------|
//! | OpenAI | `api.openai.com` | Bearer token |
//! | Anthropic | `api.anthropic.com` | x-api-key header |
//! | Ollama | Local (configurable) | None |
//!
//! # Retry Behavior
//!
//! The client automatically retries on transient errors:
//! - Connection timeouts
//! - Rate limiting (429)
//! - Server errors (5xx)
//!
//! Retry delays use exponential backoff with configurable parameters.
//!
//! # Example
//!
//! ```
//! use sql_query_analyzer::{
//!     config::RetryConfig,
//!     llm::{LlmClient, LlmProvider}
//! };
//!
//! let provider = LlmProvider::Ollama {
//!     base_url: "http://localhost:11434".into(),
//!     model:    "llama3.2".into()
//! };
//!
//! let client = LlmClient::with_retry_config(provider, RetryConfig::default());
//! ```

use std::time::Duration;

use serde::{Deserialize, Serialize};
use tokio::time::sleep;

use crate::{
    config::RetryConfig,
    error::{AppResult, http_error, llm_api_error}
};

/// LLM provider configuration with authentication credentials.
#[derive(Debug, Clone)]
pub enum LlmProvider {
    /// OpenAI API (GPT-4, GPT-3.5, etc.)
    OpenAI {
        /// API key (sk-...)
        api_key: String,
        /// Model identifier (e.g., "gpt-4", "gpt-3.5-turbo")
        model:   String
    },
    /// Anthropic API (Claude models)
    Anthropic {
        /// API key
        api_key: String,
        /// Model identifier (e.g., "claude-sonnet-4-20250514")
        model:   String
    },
    /// Local Ollama instance
    Ollama {
        /// Base URL (e.g., "http://localhost:11434")
        base_url: String,
        /// Model name (e.g., "llama3.2", "codellama")
        model:    String
    }
}

/// HTTP client for LLM API communication with retry support.
///
/// Handles provider-specific request formatting and response parsing.
/// Automatically retries transient failures with exponential backoff.
pub struct LlmClient {
    provider:     LlmProvider,
    client:       reqwest::Client,
    retry_config: RetryConfig
}

#[derive(Serialize)]
struct OpenAIRequest {
    model:    String,
    messages: Vec<OpenAIRequestMessage>
}

#[derive(Serialize)]
struct OpenAIRequestMessage {
    role:    String,
    content: String
}

#[derive(Deserialize)]
struct OpenAIResponse {
    choices: Vec<OpenAIChoice>
}

#[derive(Deserialize)]
struct OpenAIChoice {
    message: OpenAIResponseMessage
}

#[derive(Deserialize)]
struct OpenAIResponseMessage {
    content: String
}

#[derive(Serialize)]
struct AnthropicRequest {
    model:      String,
    max_tokens: u32,
    messages:   Vec<AnthropicMessage>
}

#[derive(Serialize)]
struct AnthropicMessage {
    role:    String,
    content: String
}

#[derive(Deserialize)]
struct AnthropicResponse {
    content: Vec<AnthropicContent>
}

#[derive(Deserialize)]
struct AnthropicContent {
    text: String
}

#[derive(Serialize)]
struct OllamaRequest {
    model:  String,
    prompt: String,
    stream: bool
}

#[derive(Deserialize)]
struct OllamaResponse {
    response: String
}

impl LlmClient {
    /// Create new LLM client with default retry configuration
    #[allow(dead_code)]
    pub fn new(provider: LlmProvider) -> Self {
        Self::with_retry_config(provider, RetryConfig::default())
    }

    /// Create new LLM client with custom retry configuration
    pub fn with_retry_config(provider: LlmProvider, retry_config: RetryConfig) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(120))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());
        Self {
            provider,
            client,
            retry_config
        }
    }

    /// Analyze SQL queries using LLM with automatic retry
    pub async fn analyze(&self, schema_summary: &str, queries_summary: &str) -> AppResult<String> {
        let prompt = format!(
            "You are a database performance expert. Analyze the following SQL queries \
             for potential performance issues, especially regarding index usage.\n\n\
             {schema}\n\n{queries}\n\n\
             For each query, identify:\n\
             1. Whether existing indexes can be used effectively\n\
             2. Missing indexes that would improve performance\n\
             3. Full table scans or inefficient operations\n\
             4. Suggestions for query optimization\n\
             Provide specific, actionable recommendations.",
            schema = schema_summary,
            queries = queries_summary
        );
        self.call_with_retry(&prompt).await
    }

    async fn call_with_retry(&self, prompt: &str) -> AppResult<String> {
        let mut last_error = None;
        let mut delay = self.retry_config.initial_delay_ms;
        for attempt in 0..=self.retry_config.max_retries {
            if attempt > 0 {
                eprintln!(
                    "Retrying LLM request (attempt {}/{}), waiting {}ms...",
                    attempt + 1,
                    self.retry_config.max_retries + 1,
                    delay
                );
                sleep(Duration::from_millis(delay)).await;
                delay = ((delay as f64 * self.retry_config.backoff_factor) as u64)
                    .min(self.retry_config.max_delay_ms);
            }
            match self.call_provider(prompt).await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    if self.is_retryable_error(&e) {
                        last_error = Some(e);
                        continue;
                    }
                    return Err(e);
                }
            }
        }
        Err(last_error.unwrap_or_else(|| llm_api_error("All retry attempts failed")))
    }

    fn is_retryable_error(&self, error: &masterror::AppError) -> bool {
        let msg = error.to_string().to_lowercase();
        msg.contains("timeout")
            || msg.contains("connection")
            || msg.contains("429")
            || msg.contains("rate limit")
            || msg.contains("500")
            || msg.contains("502")
            || msg.contains("503")
            || msg.contains("504")
    }

    async fn call_provider(&self, prompt: &str) -> AppResult<String> {
        match &self.provider {
            LlmProvider::OpenAI {
                api_key,
                model
            } => self.call_openai(api_key, model, prompt).await,
            LlmProvider::Anthropic {
                api_key,
                model
            } => self.call_anthropic(api_key, model, prompt).await,
            LlmProvider::Ollama {
                base_url,
                model
            } => self.call_ollama(base_url, model, prompt).await
        }
    }

    async fn call_openai(&self, api_key: &str, model: &str, prompt: &str) -> AppResult<String> {
        let request = OpenAIRequest {
            model:    model.to_string(),
            messages: vec![OpenAIRequestMessage {
                role:    String::from("user"),
                content: prompt.to_string()
            }]
        };
        let response = self
            .client
            .post("https://api.openai.com/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", api_key))
            .json(&request)
            .send()
            .await
            .map_err(http_error)?;
        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(llm_api_error(format!(
                "OpenAI API error {}: {}",
                status, text
            )));
        }
        let result: OpenAIResponse = response.json().await.map_err(http_error)?;
        result
            .choices
            .first()
            .map(|c| c.message.content.clone())
            .ok_or_else(|| llm_api_error("Empty response from OpenAI"))
    }

    async fn call_anthropic(&self, api_key: &str, model: &str, prompt: &str) -> AppResult<String> {
        let request = AnthropicRequest {
            model:      model.to_string(),
            max_tokens: 4096,
            messages:   vec![AnthropicMessage {
                role:    String::from("user"),
                content: prompt.to_string()
            }]
        };
        let response = self
            .client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", api_key)
            .header("anthropic-version", "2023-06-01")
            .json(&request)
            .send()
            .await
            .map_err(http_error)?;
        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(llm_api_error(format!(
                "Anthropic API error {}: {}",
                status, text
            )));
        }
        let result: AnthropicResponse = response.json().await.map_err(http_error)?;
        result
            .content
            .first()
            .map(|c| c.text.clone())
            .ok_or_else(|| llm_api_error("Empty response from Anthropic"))
    }

    async fn call_ollama(&self, base_url: &str, model: &str, prompt: &str) -> AppResult<String> {
        let request = OllamaRequest {
            model:  model.to_string(),
            prompt: prompt.to_string(),
            stream: false
        };
        let url = format!("{}/api/generate", base_url.trim_end_matches('/'));
        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(http_error)?;
        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(llm_api_error(format!(
                "Ollama API error {}: {}",
                status, text
            )));
        }
        let result: OllamaResponse = response.json().await.map_err(http_error)?;
        Ok(result.response)
    }
}
