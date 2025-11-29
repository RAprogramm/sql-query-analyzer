// SPDX-FileCopyrightText: 2025 RAprogramm
// SPDX-License-Identifier: MIT

use std::env::{remove_var, set_var};

use sql_query_analyzer::config::{Config, RulesConfig};

#[test]
fn test_default_config() {
    let config = Config::default();
    assert!(config.llm.api_key.is_none());
    assert!(config.llm.provider.is_none());
    assert!(config.rules.disabled.is_empty());
}

#[test]
fn test_default_retry_config() {
    let config = Config::default();
    assert_eq!(config.retry.max_retries, 3);
    assert_eq!(config.retry.initial_delay_ms, 1000);
    assert_eq!(config.retry.backoff_factor, 2.0);
}

#[test]
fn test_default_rules_config() {
    let config = RulesConfig::default();
    assert!(config.disabled.is_empty());
    assert!(config.severity.is_empty());
}

#[test]
fn test_rules_config_with_disabled() {
    let config = RulesConfig {
        disabled: vec!["PERF001".to_string(), "STYLE001".to_string()],
        ..Default::default()
    };
    assert_eq!(config.disabled.len(), 2);
    assert!(config.disabled.contains(&"PERF001".to_string()));
}

#[test]
fn test_rules_config_with_severity() {
    let mut severity = std::collections::HashMap::new();
    severity.insert("PERF001".to_string(), "error".to_string());
    let config = RulesConfig {
        disabled: vec![],
        severity
    };
    assert_eq!(config.severity.get("PERF001").unwrap(), "error");
}

#[test]
fn test_llm_config_default() {
    use sql_query_analyzer::config::LlmConfig;
    let config = LlmConfig::default();
    assert!(config.provider.is_none());
    assert!(config.api_key.is_none());
    assert!(config.model.is_none());
    assert_eq!(
        config.ollama_url,
        Some("http://localhost:11434".to_string())
    );
}

#[test]
fn test_retry_config_default() {
    use sql_query_analyzer::config::RetryConfig;
    let config = RetryConfig::default();
    assert_eq!(config.max_retries, 3);
    assert_eq!(config.initial_delay_ms, 1000);
    assert_eq!(config.max_delay_ms, 30000);
    assert_eq!(config.backoff_factor, 2.0);
}

#[test]
fn test_config_load() {
    let result = Config::load();
    assert!(result.is_ok());
}

#[test]
fn test_config_debug() {
    let config = Config::default();
    let debug = format!("{:?}", config);
    assert!(debug.contains("Config"));
}

#[test]
fn test_llm_config_debug() {
    use sql_query_analyzer::config::LlmConfig;
    let config = LlmConfig::default();
    let debug = format!("{:?}", config);
    assert!(debug.contains("LlmConfig"));
}

#[test]
fn test_retry_config_debug() {
    use sql_query_analyzer::config::RetryConfig;
    let config = RetryConfig::default();
    let debug = format!("{:?}", config);
    assert!(debug.contains("RetryConfig"));
}

#[test]
fn test_rules_config_debug() {
    let config = RulesConfig::default();
    let debug = format!("{:?}", config);
    assert!(debug.contains("RulesConfig"));
}

#[test]
fn test_config_clone() {
    let config = Config::default();
    let cloned = config.clone();
    assert_eq!(cloned.retry.max_retries, config.retry.max_retries);
}

#[test]
fn test_llm_config_clone() {
    use sql_query_analyzer::config::LlmConfig;
    let config = LlmConfig::default();
    let cloned = config.clone();
    assert_eq!(cloned.ollama_url, config.ollama_url);
}

#[test]
fn test_retry_config_clone() {
    use sql_query_analyzer::config::RetryConfig;
    let config = RetryConfig::default();
    let cloned = config.clone();
    assert_eq!(cloned.max_retries, config.max_retries);
}

#[test]
fn test_rules_config_clone() {
    let config = RulesConfig::default();
    let cloned = config.clone();
    assert_eq!(cloned.disabled.len(), config.disabled.len());
}

#[test]
fn test_config_load_with_env_vars() {
    unsafe {
        set_var("LLM_API_KEY", "test-key-12345");
        set_var("LLM_PROVIDER", "openai");
        set_var("LLM_MODEL", "gpt-4");
        set_var("OLLAMA_URL", "http://custom:11434");
    }

    let config = Config::load().unwrap();

    assert_eq!(config.llm.api_key, Some("test-key-12345".to_string()));
    assert_eq!(config.llm.provider, Some("openai".to_string()));
    assert_eq!(config.llm.model, Some("gpt-4".to_string()));
    assert_eq!(
        config.llm.ollama_url,
        Some("http://custom:11434".to_string())
    );

    unsafe {
        remove_var("LLM_API_KEY");
        remove_var("LLM_PROVIDER");
        remove_var("LLM_MODEL");
        remove_var("OLLAMA_URL");
    }
}
