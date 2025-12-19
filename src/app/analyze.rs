//! Core analysis execution logic.
//!
//! This module contains the main `run_analyze` function that orchestrates
//! the complete SQL analysis pipeline, including schema parsing, query
//! analysis, static rule checking, and optional LLM-powered analysis.

use std::{fs::read_to_string, time::Duration};

use indicatif::{ProgressBar, ProgressStyle};

use super::{
    convert::convert_dialect,
    helpers::{
        build_llm_provider, calculate_exit_code, create_output_options, get_effective_model,
        get_effective_ollama_url, has_llm_access, parse_queries_cached, read_queries_input
    },
    types::{AnalyzeParams, AnalyzeResult, DryRunInfo}
};
use crate::{
    config::Config,
    error::{AppResult, file_read_error},
    llm::LlmClient,
    output::{format_analysis_result, format_queries_summary, format_static_analysis},
    rules::RuleRunner,
    schema::Schema
};

/// Executes the complete SQL analysis pipeline.
///
/// This function orchestrates the entire analysis workflow:
///
/// 1. **Schema Parsing**: Reads and parses the schema file
/// 2. **Query Parsing**: Reads queries (from file or stdin) and parses them
/// 3. **Static Analysis**: Runs all enabled rules against the queries
/// 4. **LLM Analysis** (optional): Sends schema and queries to LLM for analysis
///
/// # Arguments
///
/// * `params` - Analysis parameters including file paths and options
/// * `config` - Application configuration with rule settings and LLM config
///
/// # Returns
///
/// An `AnalyzeResult` containing:
/// - Exit code based on violation severity
/// - Formatted static analysis output
/// - Optional LLM analysis output
/// - Optional dry-run information
///
/// # Errors
///
/// Returns an error if:
/// - Schema or query files cannot be read
/// - SQL parsing fails
/// - LLM API call fails (when LLM is enabled)
///
/// # Example
///
/// ```no_run
/// use sql_query_analyzer::{
///     app::{AnalyzeParams, run_analyze},
///     cli::{Dialect, Format, Provider},
///     config::Config
/// };
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
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
///
/// let config = Config::default();
/// let result = run_analyze(params, config).await?;
/// println!("Exit code: {}", result.exit_code);
/// # Ok(())
/// # }
/// ```
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
