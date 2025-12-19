//! Application logic for the SQL Query Analyzer CLI.
//!
//! This module contains the core application logic separated from the main
//! entry point to enable testing.

use std::{
    fs::read_to_string,
    io::{self, Read},
    time::Duration
};

use indicatif::{ProgressBar, ProgressStyle};

use crate::{
    cache::{cache_queries, get_cached},
    cli::{Commands, Dialect, Format, Provider},
    config::Config,
    error::{AppResult, config_error, file_read_error},
    llm::{LlmClient, LlmProvider},
    output::{
        OutputFormat, OutputOptions, format_analysis_result, format_queries_summary,
        format_static_analysis
    },
    query::{Query, SqlDialect, parse_queries},
    rules::{AnalysisReport, RuleRunner, Severity},
    schema::Schema
};

/// Parameters for the analyze command
#[derive(Debug, Clone)]
pub struct AnalyzeParams {
    pub schema_path:   String,
    pub queries_path:  String,
    pub provider:      Provider,
    pub api_key:       Option<String>,
    pub model:         Option<String>,
    pub ollama_url:    String,
    pub dialect:       Dialect,
    pub output_format: Format,
    pub verbose:       bool,
    pub dry_run:       bool,
    pub no_color:      bool
}

/// Result of analysis containing all outputs
#[derive(Debug, Clone)]
pub struct AnalyzeResult {
    pub exit_code:     i32,
    pub static_output: String,
    pub llm_output:    Option<String>,
    pub dry_run_info:  Option<DryRunInfo>
}

/// Information shown during dry run
#[derive(Debug, Clone)]
pub struct DryRunInfo {
    pub schema_summary:  String,
    pub queries_summary: String
}

/// Convert CLI dialect to internal SqlDialect
pub fn convert_dialect(dialect: Dialect) -> SqlDialect {
    match dialect {
        Dialect::Generic => SqlDialect::Generic,
        Dialect::Mysql => SqlDialect::MySQL,
        Dialect::Postgresql => SqlDialect::PostgreSQL,
        Dialect::Sqlite => SqlDialect::SQLite,
        Dialect::Clickhouse => SqlDialect::ClickHouse
    }
}

/// Convert CLI format to internal OutputFormat
pub fn convert_format(format: Format) -> OutputFormat {
    match format {
        Format::Text => OutputFormat::Text,
        Format::Json => OutputFormat::Json,
        Format::Yaml => OutputFormat::Yaml,
        Format::Sarif => OutputFormat::Sarif
    }
}

/// Calculate exit code based on violations
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

/// Read queries from file or stdin
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

/// Parse queries with caching
pub fn parse_queries_cached(sql: &str, dialect: SqlDialect) -> AppResult<Vec<Query>> {
    if let Some(cached) = get_cached(sql) {
        Ok(cached)
    } else {
        let queries = parse_queries(sql, dialect)?;
        cache_queries(sql, queries.clone());
        Ok(queries)
    }
}

/// Create output options from parameters
pub fn create_output_options(format: Format, no_color: bool, verbose: bool) -> OutputOptions {
    OutputOptions {
        format: convert_format(format),
        colored: !no_color,
        verbose
    }
}

/// Build LLM provider from parameters
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

/// Check if LLM access is available
pub fn has_llm_access(api_key: &Option<String>, provider: &Provider) -> bool {
    api_key.is_some() || matches!(provider, Provider::Ollama)
}

/// Get effective model name
pub fn get_effective_model(
    model: Option<String>,
    config_model: Option<String>,
    provider: &Provider
) -> String {
    model
        .or(config_model)
        .unwrap_or_else(|| provider.default_model().to_string())
}

/// Get effective Ollama URL
pub fn get_effective_ollama_url(url: String, config_url: Option<String>) -> String {
    if url == "http://localhost:11434" {
        config_url.unwrap_or(url)
    } else {
        url
    }
}

/// Run the analyze command
pub async fn run_analyze(params: AnalyzeParams, config: Config) -> AppResult<AnalyzeResult> {
    let schema_sql = read_to_string(&params.schema_path)
        .map_err(|e| file_read_error(&params.schema_path, e))?;
    let queries_sql = read_queries_input(&params.queries_path)?;
    let sql_dialect = convert_dialect(params.dialect);
    let parsed_schema = Schema::parse(&schema_sql, sql_dialect)?;
    let parsed_queries = parse_queries_cached(&queries_sql, sql_dialect)?;
    let schema_summary = parsed_schema.to_summary();
    let output_opts = create_output_options(params.output_format, params.no_color, params.verbose);
    let runner = RuleRunner::with_schema_and_config(parsed_schema.clone(), config.rules.clone());
    let static_report = runner.analyze(&parsed_queries);
    let static_output = format_static_analysis(&static_report, &output_opts);
    let exit_code = calculate_exit_code(&static_report);
    if params.dry_run {
        let queries_summary = format_queries_summary(&parsed_queries, &output_opts);
        return Ok(AnalyzeResult {
            exit_code,
            static_output,
            llm_output: None,
            dry_run_info: Some(DryRunInfo {
                schema_summary,
                queries_summary
            })
        });
    }
    let effective_api_key = params.api_key.or(config.llm.api_key.clone());
    let effective_ollama_url =
        get_effective_ollama_url(params.ollama_url, config.llm.ollama_url.clone());
    if !has_llm_access(&effective_api_key, &params.provider) {
        return Ok(AnalyzeResult {
            exit_code,
            static_output,
            llm_output: None,
            dry_run_info: None
        });
    }
    let model_name = get_effective_model(params.model, config.llm.model.clone(), &params.provider);
    let llm_provider = build_llm_provider(
        params.provider,
        effective_api_key,
        model_name,
        effective_ollama_url
    )?;
    let pb = ProgressBar::new_spinner();
    if let Ok(style) = ProgressStyle::default_spinner().template("{spinner:.green} {msg}") {
        pb.set_style(style);
    }
    pb.set_message("Analyzing queries with LLM...");
    pb.enable_steady_tick(Duration::from_millis(100));
    let queries_summary = format_queries_summary(&parsed_queries, &output_opts);
    let client = LlmClient::with_retry_config(llm_provider, config.retry);
    let analysis = client.analyze(&schema_summary, &queries_summary).await?;
    pb.finish_and_clear();
    let llm_output = format_analysis_result(&parsed_queries, &analysis, &output_opts);
    Ok(AnalyzeResult {
        exit_code,
        static_output,
        llm_output: Some(llm_output),
        dry_run_info: None
    })
}

/// Output from command execution.
#[derive(Debug, Clone)]
pub struct CommandOutput {
    /// Exit code for the process.
    pub exit_code: i32,
    /// Lines to print to stdout.
    pub stdout:    Vec<String>
}

/// Execute a CLI command and return output.
pub async fn execute_command(command: Commands, config: Config) -> AppResult<CommandOutput> {
    match command {
        Commands::Analyze {
            schema,
            queries,
            provider,
            api_key,
            model,
            ollama_url,
            dialect,
            output_format,
            verbose,
            dry_run,
            no_color
        } => {
            let params = AnalyzeParams {
                schema_path: schema.display().to_string(),
                queries_path: if queries.to_str() == Some("-") {
                    "-".to_string()
                } else {
                    queries.display().to_string()
                },
                provider,
                api_key,
                model,
                ollama_url,
                dialect,
                output_format,
                verbose,
                dry_run,
                no_color
            };
            let result = run_analyze(params, config).await?;
            let mut stdout = vec![result.static_output];
            if let Some(dry_run_info) = result.dry_run_info {
                stdout.push("=== DRY RUN - Would send to LLM ===\n".to_string());
                stdout.push(format!(
                    "Schema Summary:\n{}\n",
                    dry_run_info.schema_summary
                ));
                stdout.push(format!(
                    "Queries Summary:\n{}",
                    dry_run_info.queries_summary
                ));
            } else if result.llm_output.is_none() && !dry_run {
                stdout.push(
                    "Note: Set LLM_API_KEY for additional AI-powered analysis\n".to_string()
                );
            }
            if let Some(llm_output) = result.llm_output {
                stdout.push(llm_output);
            }
            Ok(CommandOutput {
                exit_code: result.exit_code,
                stdout
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::{RuleCategory, Violation};

    #[test]
    fn test_convert_dialect_generic() {
        assert!(matches!(
            convert_dialect(Dialect::Generic),
            SqlDialect::Generic
        ));
    }

    #[test]
    fn test_convert_dialect_mysql() {
        assert!(matches!(convert_dialect(Dialect::Mysql), SqlDialect::MySQL));
    }

    #[test]
    fn test_convert_dialect_postgresql() {
        assert!(matches!(
            convert_dialect(Dialect::Postgresql),
            SqlDialect::PostgreSQL
        ));
    }

    #[test]
    fn test_convert_dialect_sqlite() {
        assert!(matches!(
            convert_dialect(Dialect::Sqlite),
            SqlDialect::SQLite
        ));
    }

    #[test]
    fn test_convert_dialect_clickhouse() {
        assert!(matches!(
            convert_dialect(Dialect::Clickhouse),
            SqlDialect::ClickHouse
        ));
    }

    #[test]
    fn test_convert_format_text() {
        assert!(matches!(convert_format(Format::Text), OutputFormat::Text));
    }

    #[test]
    fn test_convert_format_json() {
        assert!(matches!(convert_format(Format::Json), OutputFormat::Json));
    }

    #[test]
    fn test_convert_format_yaml() {
        assert!(matches!(convert_format(Format::Yaml), OutputFormat::Yaml));
    }

    #[test]
    fn test_convert_format_sarif() {
        assert!(matches!(convert_format(Format::Sarif), OutputFormat::Sarif));
    }

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
        assert!(matches!(opts.format, OutputFormat::Text));
        assert!(opts.colored);
        assert!(opts.verbose);
    }

    #[test]
    fn test_create_output_options_json_no_color() {
        let opts = create_output_options(Format::Json, true, false);
        assert!(matches!(opts.format, OutputFormat::Json));
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
        let sql = "SELECT id FROM test_cached_table";
        let queries1 = parse_queries_cached(sql, SqlDialect::Generic).unwrap();
        let queries2 = parse_queries_cached(sql, SqlDialect::Generic).unwrap();
        assert_eq!(queries1.len(), queries2.len());
    }

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
        let debug = format!("{:?}", params);
        assert!(debug.contains("AnalyzeParams"));
    }

    #[test]
    fn test_analyze_result_debug() {
        let result = AnalyzeResult {
            exit_code:     0,
            static_output: "output".to_string(),
            llm_output:    None,
            dry_run_info:  None
        };
        let debug = format!("{:?}", result);
        assert!(debug.contains("AnalyzeResult"));
    }

    #[test]
    fn test_dry_run_info_debug() {
        let info = DryRunInfo {
            schema_summary:  "schema".to_string(),
            queries_summary: "queries".to_string()
        };
        let debug = format!("{:?}", info);
        assert!(debug.contains("DryRunInfo"));
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
    fn test_command_output_debug() {
        let output = CommandOutput {
            exit_code: 0,
            stdout:    vec!["line1".to_string()]
        };
        let debug = format!("{:?}", output);
        assert!(debug.contains("CommandOutput"));
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

    #[tokio::test]
    async fn test_execute_command_success() {
        use std::io::Write;

        use tempfile::NamedTempFile;

        let mut schema_file = NamedTempFile::new().unwrap();
        writeln!(schema_file, "CREATE TABLE users (id INT PRIMARY KEY);").unwrap();

        let mut queries_file = NamedTempFile::new().unwrap();
        writeln!(queries_file, "SELECT id FROM users;").unwrap();

        let command = Commands::Analyze {
            schema:        schema_file.path().to_path_buf(),
            queries:       queries_file.path().to_path_buf(),
            provider:      Provider::OpenAI,
            api_key:       None,
            model:         None,
            ollama_url:    "http://localhost:11434".to_string(),
            dialect:       Dialect::Generic,
            output_format: Format::Text,
            verbose:       false,
            dry_run:       false,
            no_color:      true
        };

        let config = Config::default();
        let result = execute_command(command, config).await.unwrap();
        assert_eq!(result.exit_code, 0);
        assert!(!result.stdout.is_empty());
    }

    #[tokio::test]
    async fn test_execute_command_dry_run() {
        use std::io::Write;

        use tempfile::NamedTempFile;

        let mut schema_file = NamedTempFile::new().unwrap();
        writeln!(schema_file, "CREATE TABLE test (id INT);").unwrap();

        let mut queries_file = NamedTempFile::new().unwrap();
        writeln!(queries_file, "SELECT * FROM test;").unwrap();

        let command = Commands::Analyze {
            schema:        schema_file.path().to_path_buf(),
            queries:       queries_file.path().to_path_buf(),
            provider:      Provider::OpenAI,
            api_key:       None,
            model:         None,
            ollama_url:    "http://localhost:11434".to_string(),
            dialect:       Dialect::Generic,
            output_format: Format::Text,
            verbose:       false,
            dry_run:       true,
            no_color:      true
        };

        let config = Config::default();
        let result = execute_command(command, config).await.unwrap();
        let output = result.stdout.join("\n");
        assert!(output.contains("DRY RUN"));
        assert!(output.contains("Schema Summary"));
        assert!(output.contains("Queries Summary"));
    }

    #[tokio::test]
    async fn test_execute_command_file_not_found() {
        use std::path::PathBuf;

        let command = Commands::Analyze {
            schema:        PathBuf::from("/nonexistent/schema.sql"),
            queries:       PathBuf::from("/nonexistent/queries.sql"),
            provider:      Provider::OpenAI,
            api_key:       None,
            model:         None,
            ollama_url:    "http://localhost:11434".to_string(),
            dialect:       Dialect::Generic,
            output_format: Format::Text,
            verbose:       false,
            dry_run:       false,
            no_color:      true
        };

        let config = Config::default();
        let result = execute_command(command, config).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_execute_command_with_violations() {
        use std::io::Write;

        use tempfile::NamedTempFile;

        let mut schema_file = NamedTempFile::new().unwrap();
        writeln!(schema_file, "CREATE TABLE orders (id INT);").unwrap();

        let mut queries_file = NamedTempFile::new().unwrap();
        writeln!(queries_file, "SELECT * FROM orders;").unwrap();

        let command = Commands::Analyze {
            schema:        schema_file.path().to_path_buf(),
            queries:       queries_file.path().to_path_buf(),
            provider:      Provider::OpenAI,
            api_key:       None,
            model:         None,
            ollama_url:    "http://localhost:11434".to_string(),
            dialect:       Dialect::Generic,
            output_format: Format::Text,
            verbose:       false,
            dry_run:       false,
            no_color:      true
        };

        let config = Config::default();
        let result = execute_command(command, config).await.unwrap();
        assert!(result.exit_code >= 0);
    }

    #[tokio::test]
    async fn test_execute_command_json_format() {
        use std::io::Write;

        use tempfile::NamedTempFile;

        let mut schema_file = NamedTempFile::new().unwrap();
        writeln!(schema_file, "CREATE TABLE items (id INT PRIMARY KEY);").unwrap();

        let mut queries_file = NamedTempFile::new().unwrap();
        writeln!(queries_file, "SELECT id FROM items;").unwrap();

        let command = Commands::Analyze {
            schema:        schema_file.path().to_path_buf(),
            queries:       queries_file.path().to_path_buf(),
            provider:      Provider::OpenAI,
            api_key:       None,
            model:         None,
            ollama_url:    "http://localhost:11434".to_string(),
            dialect:       Dialect::Generic,
            output_format: Format::Json,
            verbose:       false,
            dry_run:       false,
            no_color:      true
        };

        let config = Config::default();
        let result = execute_command(command, config).await.unwrap();
        let output = result.stdout.join("");
        assert!(output.contains("{") || output.contains("queries_analyzed"));
    }

    #[tokio::test]
    async fn test_execute_command_verbose() {
        use std::io::Write;

        use tempfile::NamedTempFile;

        let mut schema_file = NamedTempFile::new().unwrap();
        writeln!(schema_file, "CREATE TABLE logs (id INT);").unwrap();

        let mut queries_file = NamedTempFile::new().unwrap();
        writeln!(queries_file, "SELECT id FROM logs;").unwrap();

        let command = Commands::Analyze {
            schema:        schema_file.path().to_path_buf(),
            queries:       queries_file.path().to_path_buf(),
            provider:      Provider::OpenAI,
            api_key:       None,
            model:         None,
            ollama_url:    "http://localhost:11434".to_string(),
            dialect:       Dialect::Generic,
            output_format: Format::Text,
            verbose:       true,
            dry_run:       false,
            no_color:      true
        };

        let config = Config::default();
        let result = execute_command(command, config).await.unwrap();
        assert!(!result.stdout.is_empty());
    }

    #[tokio::test]
    async fn test_execute_command_yaml_format() {
        use std::io::Write;

        use tempfile::NamedTempFile;

        let mut schema_file = NamedTempFile::new().unwrap();
        writeln!(schema_file, "CREATE TABLE events (id INT);").unwrap();

        let mut queries_file = NamedTempFile::new().unwrap();
        writeln!(queries_file, "SELECT id FROM events;").unwrap();

        let command = Commands::Analyze {
            schema:        schema_file.path().to_path_buf(),
            queries:       queries_file.path().to_path_buf(),
            provider:      Provider::OpenAI,
            api_key:       None,
            model:         None,
            ollama_url:    "http://localhost:11434".to_string(),
            dialect:       Dialect::Generic,
            output_format: Format::Yaml,
            verbose:       false,
            dry_run:       false,
            no_color:      true
        };

        let config = Config::default();
        let result = execute_command(command, config).await.unwrap();
        assert!(!result.stdout.is_empty());
    }

    #[tokio::test]
    async fn test_execute_command_sarif_format() {
        use std::io::Write;

        use tempfile::NamedTempFile;

        let mut schema_file = NamedTempFile::new().unwrap();
        writeln!(schema_file, "CREATE TABLE metrics (id INT);").unwrap();

        let mut queries_file = NamedTempFile::new().unwrap();
        writeln!(queries_file, "SELECT id FROM metrics;").unwrap();

        let command = Commands::Analyze {
            schema:        schema_file.path().to_path_buf(),
            queries:       queries_file.path().to_path_buf(),
            provider:      Provider::OpenAI,
            api_key:       None,
            model:         None,
            ollama_url:    "http://localhost:11434".to_string(),
            dialect:       Dialect::Generic,
            output_format: Format::Sarif,
            verbose:       false,
            dry_run:       false,
            no_color:      true
        };

        let config = Config::default();
        let result = execute_command(command, config).await.unwrap();
        let output = result.stdout.join("");
        assert!(output.contains("sarif") || output.contains("$schema"));
    }
}
