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
//! - [`app`] - Application logic for CLI commands

mod app;
mod cache;
mod cli;
mod config;
mod error;
mod llm;
mod output;
mod query;
mod rules;
mod schema;

use std::process;

use clap::Parser;
use tokio::main;

use crate::{
    app::{AnalyzeParams, run_analyze},
    cli::{Cli, Commands},
    config::Config,
    error::AppResult
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

            println!("{}", result.static_output);

            if let Some(dry_run_info) = result.dry_run_info {
                println!("=== DRY RUN - Would send to LLM ===\n");
                println!("Schema Summary:\n{}\n", dry_run_info.schema_summary);
                println!("Queries Summary:\n{}", dry_run_info.queries_summary);
            } else if result.llm_output.is_none() && !dry_run {
                println!("Note: Set LLM_API_KEY for additional AI-powered analysis\n");
            }

            if let Some(llm_output) = result.llm_output {
                println!("{}", llm_output);
            }

            Ok(result.exit_code)
        }
    }
}
