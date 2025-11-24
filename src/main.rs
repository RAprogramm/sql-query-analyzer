//! # SQL Query Analyzer
//!
//! Static analysis and LLM-powered optimization for SQL queries.
//!
//! `sql-query-analyzer` is a comprehensive SQL analysis tool that combines
//! fast, deterministic static analysis with optional AI-powered insights. It
//! parses SQL queries, validates them against your database schema, and
//! identifies performance issues, style violations, and security
//! vulnerabilities.
//!
//! # Architecture
//!
//! The analyzer operates in two phases:
//!
//! 1. **Static Analysis** (always runs) - A rule engine executes 18 built-in
//!    rules in parallel using [`rayon`]. Rules are categorized as Performance,
//!    Style, or Security, each with configurable severity levels.
//!
//! 2. **LLM Analysis** (optional) - When API credentials are provided, queries
//!    are sent to OpenAI, Anthropic, or a local Ollama instance for deeper
//!    semantic analysis and optimization suggestions.
//!
//! # Quick Start
//!
//! ```bash
//! # Basic static analysis
//! sql-query-analyzer analyze -s schema.sql -q queries.sql
//!
//! # CI/CD integration with SARIF output
//! sql-query-analyzer analyze -s schema.sql -q queries.sql -f sarif > results.sarif
//!
//! # Stream queries from stdin
//! echo "SELECT * FROM users" | sql-query-analyzer analyze -s schema.sql -q -
//!
//! # Enable LLM analysis
//! export LLM_API_KEY="sk-..."
//! sql-query-analyzer analyze -s schema.sql -q queries.sql --provider openai
//! ```
//!
//! # Configuration
//!
//! Configuration is loaded from (in order of precedence):
//!
//! 1. Command-line arguments
//! 2. Environment variables (`LLM_API_KEY`, `LLM_PROVIDER`, etc.)
//! 3. `.sql-analyzer.toml` in current directory
//! 4. `~/.config/sql-analyzer/config.toml`
//!
//! ## Example Configuration
//!
//! ```toml
//! [rules]
//! # Disable specific rules by ID
//! disabled = ["STYLE001", "PERF011"]
//!
//! # Override default severity levels
//! [rules.severity]
//! PERF001 = "error"    # Promote to error
//! SCHEMA001 = "info"   # Demote to info
//!
//! [llm]
//! provider = "ollama"
//! model = "codellama"
//! ollama_url = "http://localhost:11434"
//!
//! [retry]
//! max_retries = 3
//! initial_delay_ms = 1000
//! ```
//!
//! # Rules
//!
//! ## Performance Rules (PERF001-PERF011)
//!
//! | ID | Name | Description |
//! |----|------|-------------|
//! | PERF001 | Select star without limit | `SELECT *` without `LIMIT` can return unbounded rows |
//! | PERF002 | Leading wildcard | `LIKE '%value'` prevents index usage |
//! | PERF003 | OR instead of IN | Multiple `OR` conditions can be simplified to `IN` |
//! | PERF004 | Large offset | `OFFSET > 1000` causes performance degradation |
//! | PERF005 | Missing join condition | Cartesian product detected |
//! | PERF006 | Distinct with order by | Potentially redundant operations |
//! | PERF007 | Scalar subquery | N+1 query pattern detected |
//! | PERF008 | Function on column | Function calls prevent index usage |
//! | PERF009 | NOT IN with subquery | Can cause unexpected NULL behavior |
//! | PERF010 | UNION without ALL | Unnecessary deduplication overhead |
//! | PERF011 | Select without where | Full table scan on large tables |
//!
//! ## Style Rules (STYLE001-STYLE002)
//!
//! | ID | Name | Description |
//! |----|------|-------------|
//! | STYLE001 | Select star | Explicit column list preferred |
//! | STYLE002 | Missing table alias | Multi-table queries should use aliases |
//!
//! ## Security Rules (SEC001-SEC002)
//!
//! | ID | Name | Description |
//! |----|------|-------------|
//! | SEC001 | Missing WHERE in UPDATE | Potentially dangerous bulk update |
//! | SEC002 | Missing WHERE in DELETE | Potentially dangerous bulk delete |
//!
//! ## Schema-Aware Rules (SCHEMA001-SCHEMA003)
//!
//! | ID | Name | Description |
//! |----|------|-------------|
//! | SCHEMA001 | Missing index on filter | WHERE/JOIN column lacks index |
//! | SCHEMA002 | Column not in schema | Referenced column doesn't exist |
//! | SCHEMA003 | Index suggestion | ORDER BY column could benefit from index |
//!
//! # Exit Codes
//!
//! The process exit code reflects the highest severity violation found:
//!
//! - `0` - Success, no issues or only informational messages
//! - `1` - Warnings found
//! - `2` - Errors found
//!
//! # Output Formats
//!
//! - `text` - Human-readable colored output (default)
//! - `json` - Structured JSON for programmatic processing
//! - `yaml` - YAML format for configuration management
//! - `sarif` - SARIF 2.1.0 for CI/CD integration (GitHub, GitLab, etc.)
//!
//! # Modules
//!
//! - [`rules`] - Static analysis rule engine and built-in rules
//! - [`query`] - SQL parsing and query metadata extraction
//! - [`schema`] - Database schema parsing and representation
//! - [`llm`] - LLM provider integrations (OpenAI, Anthropic, Ollama)
//! - [`config`] - Configuration loading and validation
//! - [`output`] - Result formatting for various output formats
//! - [`cache`] - Query parsing cache for performance
//! - [`error`] - Error types and constructors

mod cache;
mod cli;
mod config;
mod error;
mod llm;
mod output;
mod query;
mod rules;
mod schema;

use std::{
    fs::read_to_string,
    io::{self, Read},
    process,
    time::Duration
};

use clap::Parser;
use indicatif::{ProgressBar, ProgressStyle};
use tokio::main;

use crate::{
    cache::{cache_queries, get_cached},
    cli::{Cli, Commands, Dialect, Format, Provider},
    config::Config,
    error::{AppResult, config_error, file_read_error},
    llm::{LlmClient, LlmProvider},
    output::{
        OutputFormat, OutputOptions, format_analysis_result, format_queries_summary,
        format_static_analysis
    },
    query::{SqlDialect, parse_queries},
    rules::{RuleRunner, Severity},
    schema::Schema
};

#[main]
async fn main() {
    match run().await {
        Ok(code) => process::exit(code),
        Err(e) => {
            eprintln!("Error: {}", e);
            process::exit(1);
        }
    }
}

async fn run() -> AppResult<i32> {
    let cli = Cli::parse();
    let config = Config::load()?;

    match cli.command {
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
            let schema_sql = read_to_string(&schema)
                .map_err(|e| file_read_error(&schema.display().to_string(), e))?;

            // Support stdin for queries with "-"
            let queries_sql = if queries.to_str() == Some("-") {
                let mut buffer = String::new();
                io::stdin()
                    .read_to_string(&mut buffer)
                    .map_err(|e| file_read_error("stdin", e))?;
                buffer
            } else {
                read_to_string(&queries)
                    .map_err(|e| file_read_error(&queries.display().to_string(), e))?
            };

            // Convert CLI dialect to query dialect
            let sql_dialect = match dialect {
                Dialect::Generic => SqlDialect::Generic,
                Dialect::Mysql => SqlDialect::MySQL,
                Dialect::Postgresql => SqlDialect::PostgreSQL,
                Dialect::Sqlite => SqlDialect::SQLite
            };

            let parsed_schema = Schema::parse(&schema_sql)?;

            // Use cache for queries
            let parsed_queries = if let Some(cached) = get_cached(&queries_sql) {
                cached
            } else {
                let queries = parse_queries(&queries_sql, sql_dialect)?;
                cache_queries(&queries_sql, queries.clone());
                queries
            };

            let schema_summary = parsed_schema.to_summary();

            // Setup output options
            let output_opts = OutputOptions {
                format: match output_format {
                    Format::Text => OutputFormat::Text,
                    Format::Json => OutputFormat::Json,
                    Format::Yaml => OutputFormat::Yaml,
                    Format::Sarif => OutputFormat::Sarif
                },
                colored: !no_color,
                verbose
            };

            // Run static analysis (always) with schema-aware rules
            let runner =
                RuleRunner::with_schema_and_config(parsed_schema.clone(), config.rules.clone());
            let static_report = runner.analyze(&parsed_queries);
            let static_output = format_static_analysis(&static_report, &output_opts);
            println!("{}", static_output);

            // Determine exit code based on violations
            let exit_code = if static_report
                .violations
                .iter()
                .any(|v| v.severity == Severity::Error)
            {
                2 // Errors found
            } else if static_report
                .violations
                .iter()
                .any(|v| v.severity == Severity::Warning)
            {
                1 // Warnings found
            } else {
                0 // No issues
            };

            // Dry run mode - show what would be sent to LLM
            if dry_run {
                let queries_summary = format_queries_summary(&parsed_queries, &output_opts);
                println!("=== DRY RUN - Would send to LLM ===\n");
                println!("Schema Summary:\n{}\n", schema_summary);
                println!("Queries Summary:\n{}", queries_summary);
                return Ok(exit_code);
            }

            // Check if LLM analysis is available
            let effective_api_key = api_key.or(config.llm.api_key.clone());
            let effective_ollama_url = if ollama_url == "http://localhost:11434" {
                config.llm.ollama_url.clone().unwrap_or(ollama_url)
            } else {
                ollama_url
            };

            // Only run LLM analysis if we have credentials (or using Ollama)
            let has_llm_access =
                effective_api_key.is_some() || matches!(provider, Provider::Ollama);

            if !has_llm_access {
                println!("Note: Set LLM_API_KEY for additional AI-powered analysis\n");
                return Ok(exit_code);
            }

            let model_name = model
                .or(config.llm.model.clone())
                .unwrap_or_else(|| provider.default_model().to_string());

            let llm_provider = match provider {
                Provider::OpenAI => {
                    let key = effective_api_key.ok_or_else(|| {
                        config_error("API key required for OpenAI (use --api-key or LLM_API_KEY)")
                    })?;
                    LlmProvider::OpenAI {
                        api_key: key,
                        model:   model_name
                    }
                }
                Provider::Anthropic => {
                    let key = effective_api_key.ok_or_else(|| {
                        config_error(
                            "API key required for Anthropic (use --api-key or LLM_API_KEY)"
                        )
                    })?;
                    LlmProvider::Anthropic {
                        api_key: key,
                        model:   model_name
                    }
                }
                Provider::Ollama => LlmProvider::Ollama {
                    base_url: effective_ollama_url,
                    model:    model_name
                }
            };

            // Show progress indicator
            let pb = ProgressBar::new_spinner();
            if let Ok(style) = ProgressStyle::default_spinner().template("{spinner:.green} {msg}")
            {
                pb.set_style(style);
            }
            pb.set_message("Analyzing queries with LLM...");
            pb.enable_steady_tick(Duration::from_millis(100));

            let queries_summary = format_queries_summary(&parsed_queries, &output_opts);
            let client = LlmClient::with_retry_config(llm_provider, config.retry);
            let analysis = client.analyze(&schema_summary, &queries_summary).await?;

            pb.finish_and_clear();

            let output = format_analysis_result(&parsed_queries, &analysis, &output_opts);
            println!("{}", output);

            Ok(exit_code)
        }
    }
}
