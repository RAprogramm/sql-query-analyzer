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
