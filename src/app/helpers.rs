//! Helper functions for CLI operations.
//!
//! This module provides utility functions used throughout the CLI application
//! for tasks such as reading input, calculating exit codes, building LLM
//! providers, and managing configuration defaults.

use std::{
    fs::read_to_string,
    io::{self, Read}
};

use super::convert::convert_format;
use crate::{
    cache::{cache_queries, get_cached},
    cli::{Format, Provider},
    error::{AppResult, config_error, file_read_error},
    llm::LlmProvider,
    output::OutputOptions,
    query::{Query, SqlDialect, parse_queries},
    rules::{AnalysisReport, Severity}
};

/// Calculates the process exit code based on violation severities.
///
/// Examines all violations in the analysis report and returns an exit
/// code reflecting the highest severity found:
/// - `0` - No violations or only informational messages
/// - `1` - At least one warning present
/// - `2` - At least one error present
///
/// # Arguments
///
/// * `report` - The analysis report containing violations
///
/// # Returns
///
/// An integer exit code (0, 1, or 2).
///
/// # Example
///
/// ```
/// use sql_query_analyzer::{app::calculate_exit_code, rules::AnalysisReport};
///
/// let report = AnalysisReport::new(1, 0);
/// assert_eq!(calculate_exit_code(&report), 0);
/// ```
pub fn calculate_exit_code(report: &AnalysisReport) -> i32 {
    if report
        .violations
        .iter()
        .any(|v| v.severity == Severity::Error)
    {
        2
    } else if report
        .violations
        .iter()
        .any(|v| v.severity == Severity::Warning)
    {
        1
    } else {
        0
    }
}

/// Reads SQL queries from a file or stdin.
///
/// Supports reading from a file path or from standard input when the
/// path is "-".
///
/// # Arguments
///
/// * `path` - File path or "-" for stdin
///
/// # Returns
///
/// The file contents as a string, or an error if reading fails.
///
/// # Errors
///
/// Returns an error if the file cannot be read or stdin fails.
pub fn read_queries_input(path: &str) -> AppResult<String> {
    if path == "-" {
        let mut buffer = String::new();
        io::stdin()
            .read_to_string(&mut buffer)
            .map_err(|e| file_read_error("stdin", e))?;
        Ok(buffer)
    } else {
        read_to_string(path).map_err(|e| file_read_error(path, e))
    }
}

/// Parses SQL queries with caching support.
///
/// Attempts to retrieve parsed queries from the cache first. If not
/// found, parses the SQL and stores the result in the cache for
/// future use.
///
/// # Arguments
///
/// * `sql` - Raw SQL string containing one or more queries
/// * `dialect` - SQL dialect for parsing
///
/// # Returns
///
/// A vector of parsed Query objects.
///
/// # Errors
///
/// Returns an error if SQL parsing fails.
pub fn parse_queries_cached(sql: &str, dialect: SqlDialect) -> AppResult<Vec<Query>> {
    if let Some(cached) = get_cached(sql) {
        Ok(cached)
    } else {
        let queries = parse_queries(sql, dialect)?;
        cache_queries(sql, queries.clone());
        Ok(queries)
    }
}

/// Creates output options from CLI parameters.
///
/// Constructs an `OutputOptions` struct from the CLI format, color,
/// and verbosity settings.
///
/// # Arguments
///
/// * `format` - Output format (text, json, yaml, sarif)
/// * `no_color` - Whether to disable colored output
/// * `verbose` - Whether to enable verbose output
///
/// # Returns
///
/// An `OutputOptions` struct configured with the given settings.
pub fn create_output_options(format: Format, no_color: bool, verbose: bool) -> OutputOptions {
    OutputOptions {
        format: convert_format(format),
        colored: !no_color,
        verbose
    }
}

/// Builds an LLM provider configuration from CLI parameters.
///
/// Constructs the appropriate `LlmProvider` variant based on the
/// selected provider type. For cloud providers (OpenAI, Anthropic),
/// an API key is required.
///
/// # Arguments
///
/// * `provider` - The LLM provider type
/// * `api_key` - Optional API key for cloud providers
/// * `model` - Model name to use
/// * `ollama_url` - Base URL for Ollama server
///
/// # Returns
///
/// An `LlmProvider` configured for the selected provider.
///
/// # Errors
///
/// Returns an error if a cloud provider is selected without an API key.
pub fn build_llm_provider(
    provider: Provider,
    api_key: Option<String>,
    model: String,
    ollama_url: String
) -> AppResult<LlmProvider> {
    match provider {
        Provider::OpenAI => {
            let key = api_key.ok_or_else(|| {
                config_error("API key required for OpenAI (use --api-key or LLM_API_KEY)")
            })?;
            Ok(LlmProvider::OpenAI {
                api_key: key,
                model
            })
        }
        Provider::Anthropic => {
            let key = api_key.ok_or_else(|| {
                config_error("API key required for Anthropic (use --api-key or LLM_API_KEY)")
            })?;
            Ok(LlmProvider::Anthropic {
                api_key: key,
                model
            })
        }
        Provider::Ollama => Ok(LlmProvider::Ollama {
            base_url: ollama_url,
            model
        })
    }
}

/// Checks if LLM access is available.
///
/// Determines whether LLM analysis can be performed based on the
/// provider type and API key availability. Ollama doesn't require
/// an API key, while cloud providers do.
///
/// # Arguments
///
/// * `api_key` - Optional API key
/// * `provider` - The LLM provider type
///
/// # Returns
///
/// `true` if LLM access is available, `false` otherwise.
pub fn has_llm_access(api_key: &Option<String>, provider: &Provider) -> bool {
    api_key.is_some() || matches!(provider, Provider::Ollama)
}

/// Gets the effective model name from available sources.
///
/// Resolves the model name in order of precedence:
/// 1. Explicitly provided model name
/// 2. Model from configuration file
/// 3. Default model for the provider
///
/// # Arguments
///
/// * `model` - Explicitly provided model name
/// * `config_model` - Model from configuration
/// * `provider` - The LLM provider (for default)
///
/// # Returns
///
/// The resolved model name.
pub fn get_effective_model(
    model: Option<String>,
    config_model: Option<String>,
    provider: &Provider
) -> String {
    model
        .or(config_model)
        .unwrap_or_else(|| provider.default_model().to_string())
}

/// Gets the effective Ollama URL from available sources.
///
/// Uses the config URL if the provided URL is the default localhost,
/// otherwise uses the explicitly provided URL.
///
/// # Arguments
///
/// * `url` - Explicitly provided URL
/// * `config_url` - URL from configuration
///
/// # Returns
///
/// The resolved Ollama URL.
pub fn get_effective_ollama_url(url: String, config_url: Option<String>) -> String {
    if url == "http://localhost:11434" {
        config_url.unwrap_or(url)
    } else {
        url
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::{AnalysisReport, RuleCategory, Violation};

    #[test]
    fn test_calculate_exit_code_no_violations() {
        let report = AnalysisReport::new(1, 1);
        assert_eq!(calculate_exit_code(&report), 0);
    }

    #[test]
    fn test_calculate_exit_code_info_only() {
        let mut report = AnalysisReport::new(1, 1);
        report.add_violation(Violation {
            rule_id:     "TEST",
            rule_name:   "Test",
            message:     "Test".to_string(),
            severity:    Severity::Info,
            category:    RuleCategory::Style,
            suggestion:  None,
            query_index: 0
        });
        assert_eq!(calculate_exit_code(&report), 0);
    }

    #[test]
    fn test_calculate_exit_code_warning() {
        let mut report = AnalysisReport::new(1, 1);
        report.add_violation(Violation {
            rule_id:     "TEST",
            rule_name:   "Test",
            message:     "Test".to_string(),
            severity:    Severity::Warning,
            category:    RuleCategory::Performance,
            suggestion:  None,
            query_index: 0
        });
        assert_eq!(calculate_exit_code(&report), 1);
    }

    #[test]
    fn test_calculate_exit_code_error() {
        let mut report = AnalysisReport::new(1, 1);
        report.add_violation(Violation {
            rule_id:     "TEST",
            rule_name:   "Test",
            message:     "Test".to_string(),
            severity:    Severity::Error,
            category:    RuleCategory::Security,
            suggestion:  None,
            query_index: 0
        });
        assert_eq!(calculate_exit_code(&report), 2);
    }

    #[test]
    fn test_calculate_exit_code_error_takes_precedence() {
        let mut report = AnalysisReport::new(1, 1);
        report.add_violation(Violation {
            rule_id:     "W1",
            rule_name:   "Warning",
            message:     "Warning".to_string(),
            severity:    Severity::Warning,
            category:    RuleCategory::Performance,
            suggestion:  None,
            query_index: 0
        });
        report.add_violation(Violation {
            rule_id:     "E1",
            rule_name:   "Error",
            message:     "Error".to_string(),
            severity:    Severity::Error,
            category:    RuleCategory::Security,
            suggestion:  None,
            query_index: 0
        });
        assert_eq!(calculate_exit_code(&report), 2);
    }

    #[test]
    fn test_has_llm_access_with_api_key() {
        assert!(has_llm_access(&Some("key".to_string()), &Provider::OpenAI));
    }

    #[test]
    fn test_has_llm_access_ollama_no_key() {
        assert!(has_llm_access(&None, &Provider::Ollama));
    }

    #[test]
    fn test_has_llm_access_openai_no_key() {
        assert!(!has_llm_access(&None, &Provider::OpenAI));
    }

    #[test]
    fn test_has_llm_access_anthropic_no_key() {
        assert!(!has_llm_access(&None, &Provider::Anthropic));
    }

    #[test]
    fn test_get_effective_model_explicit() {
        let model = get_effective_model(Some("gpt-4o".to_string()), None, &Provider::OpenAI);
        assert_eq!(model, "gpt-4o");
    }

    #[test]
    fn test_get_effective_model_from_config() {
        let model = get_effective_model(None, Some("claude-3".to_string()), &Provider::Anthropic);
        assert_eq!(model, "claude-3");
    }

    #[test]
    fn test_get_effective_model_default() {
        let model = get_effective_model(None, None, &Provider::OpenAI);
        assert_eq!(model, "gpt-4");
    }

    #[test]
    fn test_get_effective_ollama_url_explicit() {
        let url = get_effective_ollama_url(
            "http://custom:11434".to_string(),
            Some("http://other:11434".to_string())
        );
        assert_eq!(url, "http://custom:11434");
    }

    #[test]
    fn test_get_effective_ollama_url_from_config() {
        let url = get_effective_ollama_url(
            "http://localhost:11434".to_string(),
            Some("http://config:11434".to_string())
        );
        assert_eq!(url, "http://config:11434");
    }

    #[test]
    fn test_get_effective_ollama_url_default() {
        let url = get_effective_ollama_url("http://localhost:11434".to_string(), None);
        assert_eq!(url, "http://localhost:11434");
    }

    #[test]
    fn test_create_output_options_text_colored() {
        let opts = create_output_options(Format::Text, false, true);
        assert!(matches!(opts.format, crate::output::OutputFormat::Text));
        assert!(opts.colored);
        assert!(opts.verbose);
    }

    #[test]
    fn test_create_output_options_json_no_color() {
        let opts = create_output_options(Format::Json, true, false);
        assert!(matches!(opts.format, crate::output::OutputFormat::Json));
        assert!(!opts.colored);
        assert!(!opts.verbose);
    }

    #[test]
    fn test_build_llm_provider_ollama() {
        let provider = build_llm_provider(
            Provider::Ollama,
            None,
            "llama3".to_string(),
            "http://localhost:11434".to_string()
        )
        .unwrap();
        assert!(matches!(provider, LlmProvider::Ollama { .. }));
    }

    #[test]
    fn test_build_llm_provider_openai_no_key() {
        let result = build_llm_provider(
            Provider::OpenAI,
            None,
            "gpt-4".to_string(),
            "http://localhost:11434".to_string()
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_build_llm_provider_openai_with_key() {
        let provider = build_llm_provider(
            Provider::OpenAI,
            Some("sk-test".to_string()),
            "gpt-4".to_string(),
            "http://localhost:11434".to_string()
        )
        .unwrap();
        assert!(matches!(provider, LlmProvider::OpenAI { .. }));
    }

    #[test]
    fn test_build_llm_provider_anthropic_no_key() {
        let result = build_llm_provider(
            Provider::Anthropic,
            None,
            "claude-3".to_string(),
            "http://localhost:11434".to_string()
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_build_llm_provider_anthropic_with_key() {
        let provider = build_llm_provider(
            Provider::Anthropic,
            Some("sk-test".to_string()),
            "claude-3".to_string(),
            "http://localhost:11434".to_string()
        )
        .unwrap();
        assert!(matches!(provider, LlmProvider::Anthropic { .. }));
    }

    #[test]
    fn test_parse_queries_cached() {
        let sql = "SELECT id FROM test_cached_table_helpers";
        let queries1 = parse_queries_cached(sql, SqlDialect::Generic).unwrap();
        let queries2 = parse_queries_cached(sql, SqlDialect::Generic).unwrap();
        assert_eq!(queries1.len(), queries2.len());
    }
}
