// SPDX-FileCopyrightText: 2025 RAprogramm
// SPDX-License-Identifier: MIT

use std::{
    env::{current_dir, remove_var, set_current_dir, set_var, var_os},
    ffi::OsString,
    fs::{create_dir_all, write},
    path::{Path, PathBuf},
    sync::Mutex
};

static ENV_LOCK: Mutex<()> = Mutex::new(());

/// Acquire the environment lock, recovering if a panicking test poisoned it.
/// Recovery is safe only because every guarded test restores everything it
/// changed through a drop guard ([`EnvRestore`], [`HomeGuard`], [`CwdGuard`]),
/// which runs even when the test panics.
fn env_guard() -> std::sync::MutexGuard<'static, ()> {
    ENV_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

/// Restores the listed environment variables when the guard is dropped.
/// Declare it after the lock guard so restoration runs before the lock is
/// released, even when the test panics.
struct EnvRestore {
    saved: Vec<(&'static str, Option<OsString>)>
}

impl EnvRestore {
    fn new(vars: &[&'static str]) -> Self {
        Self {
            saved: vars.iter().map(|name| (*name, var_os(name))).collect()
        }
    }
}

impl Drop for EnvRestore {
    fn drop(&mut self) {
        unsafe {
            for (name, value) in &self.saved {
                match value {
                    Some(v) => set_var(name, v),
                    None => remove_var(name)
                }
            }
        }
    }
}

/// Points `HOME` at a temporary directory and restores the previous value on
/// drop, even when the wrapped closure panics.
struct HomeGuard {
    saved: Option<OsString>
}

impl HomeGuard {
    fn new(home: &Path) -> Self {
        let saved = var_os("HOME");
        unsafe { set_var("HOME", home) };
        Self {
            saved
        }
    }
}

impl Drop for HomeGuard {
    fn drop(&mut self) {
        unsafe {
            match &self.saved {
                Some(v) => set_var("HOME", v),
                None => remove_var("HOME")
            }
        }
    }
}

/// Moves the process into `dir` and restores the previous directory on drop,
/// even when the surrounding test panics.
struct CwdGuard {
    saved: PathBuf
}

impl CwdGuard {
    fn to(dir: &Path) -> Self {
        let saved = current_dir().expect("current directory is readable");
        set_current_dir(dir).expect("test can change the working directory");
        Self {
            saved
        }
    }
}

impl Drop for CwdGuard {
    fn drop(&mut self) {
        let _ = set_current_dir(&self.saved);
    }
}

use sql_query_analyzer::config::{Config, RulesConfig};

#[test]
fn test_default_config() {
    let config = Config::default();
    assert!(config.llm.api_key.is_none());
    assert!(config.llm.provider.is_none());
    assert_eq!(config.rules.disabled, [] as [std::string::String; 0]);
}

#[test]
fn test_default_retry_config() {
    let config = Config::default();
    assert_eq!(config.retry.max_retries, 3);
    assert_eq!(config.retry.initial_delay_ms, 1000);
    assert_eq!(config.retry.backoff_factor, 2);
}

#[test]
fn test_default_rules_config() {
    let config = RulesConfig::default();
    assert_eq!(config.disabled, [] as [std::string::String; 0]);
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
    assert_eq!(config.backoff_factor, 2);
}

#[test]
fn test_config_load() {
    let _guard = env_guard();
    let result = Config::load();
    assert!(result.is_ok());
}

#[test]
fn test_config_debug() {
    let config = Config::default();
    let debug = format!("{config:?}");
    assert!(debug.contains("Config"));
}

#[test]
fn test_llm_config_debug() {
    use sql_query_analyzer::config::LlmConfig;
    let config = LlmConfig::default();
    let debug = format!("{config:?}");
    assert!(debug.contains("LlmConfig"));
}

#[test]
fn test_retry_config_debug() {
    use sql_query_analyzer::config::RetryConfig;
    let config = RetryConfig::default();
    let debug = format!("{config:?}");
    assert!(debug.contains("RetryConfig"));
}

#[test]
fn test_rules_config_debug() {
    let config = RulesConfig::default();
    let debug = format!("{config:?}");
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
    let _guard = env_guard();
    let _restore = EnvRestore::new(&["LLM_API_KEY", "LLM_PROVIDER", "LLM_MODEL", "OLLAMA_URL"]);
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
}

fn with_home<T>(home: &std::path::Path, f: impl FnOnce() -> T) -> T {
    let _guard = HomeGuard::new(home);
    f()
}

#[test]
fn test_load_home_config_file() {
    let _guard = env_guard();
    let _restore = EnvRestore::new(&["LLM_MODEL", "LLM_PROVIDER"]);
    unsafe {
        remove_var("LLM_MODEL");
        remove_var("LLM_PROVIDER");
    }
    let home = tempfile::tempdir().unwrap();
    let dir = home.path().join(".config").join("sql-analyzer");
    create_dir_all(&dir).unwrap();
    write(
        dir.join("config.toml"),
        "[llm]\nprovider = \"home-provider\"\nmodel = \"home-model\"\n"
    )
    .unwrap();
    let config = with_home(home.path(), Config::load).unwrap();
    assert_eq!(config.llm.provider, Some("home-provider".to_string()));
    assert_eq!(config.llm.model, Some("home-model".to_string()));
}

#[test]
fn test_load_invalid_home_config_errors() {
    let _guard = env_guard();
    let home = tempfile::tempdir().unwrap();
    let dir = home.path().join(".config").join("sql-analyzer");
    create_dir_all(&dir).unwrap();
    write(dir.join("config.toml"), "not = [valid toml").unwrap();
    let err = with_home(home.path(), Config::load).unwrap_err();
    assert!(err.to_string().contains("Invalid config file"));
}

#[test]
fn test_load_local_config_file_valid_and_invalid() {
    let _guard = env_guard();
    let dir = tempfile::tempdir().unwrap();
    let local = dir.path().join(".sql-analyzer.toml");
    write(&local, "[rules]\ndisabled = [\"STYLE001\"]\n").unwrap();
    let _cwd = CwdGuard::to(dir.path());
    let valid = Config::load();
    write(&local, "broken [").unwrap();
    let invalid = Config::load();
    assert_eq!(valid.unwrap().rules.disabled, vec!["STYLE001".to_string()]);
    assert!(
        invalid
            .unwrap_err()
            .to_string()
            .contains("Invalid config file")
    );
}
