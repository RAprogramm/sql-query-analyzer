//! Application types for CLI commands.
//!
//! This module defines the core data structures used throughout the CLI
//! application, including command parameters, analysis results, and
//! execution outputs.

use crate::cli::{Dialect, Format, Provider};

/// Parameters for the analyze command.
///
/// Contains all configuration options passed from the CLI to control
/// the analysis behavior, including file paths, LLM settings, and
/// output preferences.
///
/// # Example
///
/// ```
/// use sql_query_analyzer::{
///     app::AnalyzeParams,
///     cli::{Dialect, Format, Provider}
/// };
///
/// let params = AnalyzeParams {
///     schema_path:   "schema.sql".to_string(),
///     queries_path:  "queries.sql".to_string(),
///     provider:      Provider::Ollama,
///     api_key:       None,
///     model:         None,
///     ollama_url:    "http://localhost:11434".to_string(),
///     dialect:       Dialect::Generic,
///     output_format: Format::Text,
///     verbose:       false,
///     dry_run:       false,
///     no_color:      false
/// };
/// ```
#[derive(Debug, Clone)]
pub struct AnalyzeParams {
    /// Path to the SQL schema file containing table definitions.
    pub schema_path:   String,
    /// Path to queries file or "-" for stdin input.
    pub queries_path:  String,
    /// LLM provider for AI-powered analysis.
    pub provider:      Provider,
    /// API key for cloud LLM providers (OpenAI, Anthropic).
    pub api_key:       Option<String>,
    /// Model name to use for LLM analysis.
    pub model:         Option<String>,
    /// Base URL for Ollama server.
    pub ollama_url:    String,
    /// SQL dialect for parsing.
    pub dialect:       Dialect,
    /// Output format for results.
    pub output_format: Format,
    /// Enable verbose output with additional details.
    pub verbose:       bool,
    /// Dry run mode - show what would be sent to LLM.
    pub dry_run:       bool,
    /// Disable colored terminal output.
    pub no_color:      bool
}

/// Result of analysis containing all outputs.
///
/// Encapsulates the complete analysis result including static analysis
/// output, optional LLM analysis, and dry run information.
///
/// # Fields
///
/// * `exit_code` - Process exit code (0=success, 1=warnings, 2=errors)
/// * `static_output` - Formatted static analysis results
/// * `llm_output` - Optional LLM analysis results
/// * `dry_run_info` - Present when running in dry-run mode
#[derive(Debug, Clone)]
pub struct AnalyzeResult {
    /// Exit code based on violation severity (0, 1, or 2).
    pub exit_code:     i32,
    /// Formatted static analysis output.
    pub static_output: String,
    /// Optional LLM analysis output.
    pub llm_output:    Option<String>,
    /// Dry run information if in dry-run mode.
    pub dry_run_info:  Option<DryRunInfo>
}

/// Information shown during dry run mode.
///
/// Contains the summaries that would be sent to the LLM for analysis,
/// allowing users to preview the data before making API calls.
#[derive(Debug, Clone)]
pub struct DryRunInfo {
    /// Schema summary in human-readable format.
    pub schema_summary:  String,
    /// Queries summary in human-readable format.
    pub queries_summary: String
}

/// Output from CLI command execution.
///
/// Represents the final output ready for display, including the exit
/// code and all lines to be printed to stdout.
///
/// # Example
///
/// ```
/// use sql_query_analyzer::app::CommandOutput;
///
/// let output = CommandOutput {
///     exit_code: 0,
///     stdout:    vec!["Analysis complete.".to_string()]
/// };
/// ```
#[derive(Debug, Clone)]
pub struct CommandOutput {
    /// Exit code for the process (0=success, 1=warnings, 2=errors).
    pub exit_code: i32,
    /// Lines to print to stdout.
    pub stdout:    Vec<String>
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analyze_params_debug() {
        let params = AnalyzeParams {
            schema_path:   "schema.sql".to_string(),
            queries_path:  "queries.sql".to_string(),
            provider:      Provider::Ollama,
            api_key:       None,
            model:         None,
            ollama_url:    "http://localhost:11434".to_string(),
            dialect:       Dialect::Generic,
            output_format: Format::Text,
            verbose:       false,
            dry_run:       false,
            no_color:      false
        };
        assert!(format!("{:?}", params).contains("AnalyzeParams"));
    }

    #[test]
    fn test_analyze_params_clone() {
        let params = AnalyzeParams {
            schema_path:   "schema.sql".to_string(),
            queries_path:  "queries.sql".to_string(),
            provider:      Provider::Ollama,
            api_key:       None,
            model:         None,
            ollama_url:    "http://localhost:11434".to_string(),
            dialect:       Dialect::Generic,
            output_format: Format::Text,
            verbose:       false,
            dry_run:       false,
            no_color:      false
        };
        let cloned = params.clone();
        assert_eq!(cloned.schema_path, params.schema_path);
    }

    #[test]
    fn test_analyze_result_debug() {
        let result = AnalyzeResult {
            exit_code:     0,
            static_output: "output".to_string(),
            llm_output:    None,
            dry_run_info:  None
        };
        assert!(format!("{:?}", result).contains("AnalyzeResult"));
    }

    #[test]
    fn test_dry_run_info_debug() {
        let info = DryRunInfo {
            schema_summary:  "schema".to_string(),
            queries_summary: "queries".to_string()
        };
        assert!(format!("{:?}", info).contains("DryRunInfo"));
    }

    #[test]
    fn test_command_output_debug() {
        let output = CommandOutput {
            exit_code: 0,
            stdout:    vec!["line1".to_string()]
        };
        assert!(format!("{:?}", output).contains("CommandOutput"));
    }

    #[test]
    fn test_command_output_clone() {
        let output = CommandOutput {
            exit_code: 1,
            stdout:    vec!["error".to_string()]
        };
        let cloned = output.clone();
        assert_eq!(cloned.exit_code, 1);
        assert_eq!(cloned.stdout.len(), 1);
    }
}
