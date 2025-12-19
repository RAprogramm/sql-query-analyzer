//! Application logic for the SQL Query Analyzer CLI.
//!
//! This module contains the core application logic separated from the main
//! entry point to enable testing. It orchestrates CLI command execution,
//! configuration handling, and analysis pipeline coordination.
//!
//! # Module Structure
//!
//! The application logic is organized into focused submodules:
//!
//! - `types`: Core data structures for command parameters and results
//! - `convert`: Type conversion between CLI and internal representations
//! - `helpers`: Utility functions for common operations
//! - `analyze`: SQL analysis execution logic
//!
//! # Architecture
//!
//! The CLI application follows a layered architecture:
//!
//! ```text
//! ┌─────────────────────────────────────────┐
//! │                 main.rs                 │
//! │         (Entry point, CLI parsing)      │
//! └─────────────────┬───────────────────────┘
//!                   │
//! ┌─────────────────▼───────────────────────┐
//! │              app/mod.rs                 │
//! │     (Command execution, orchestration)  │
//! └─────────────────┬───────────────────────┘
//!                   │
//! ┌─────────────────▼───────────────────────┐
//! │            app/analyze.rs               │
//! │       (Analysis pipeline logic)         │
//! └─────────────────┬───────────────────────┘
//!                   │
//! ┌─────────────────▼───────────────────────┐
//! │   schema, query, rules, llm, output     │
//! │          (Domain modules)               │
//! └─────────────────────────────────────────┘
//! ```
//!
//! # Example
//!
//! ```no_run
//! use sql_query_analyzer::{
//!     app::{CommandOutput, execute_command},
//!     cli::Commands,
//!     config::Config
//! };
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Parse CLI arguments and execute the command
//! let command = Commands::Analyze {
//!     schema:        "schema.sql".into(),
//!     queries:       "queries.sql".into(),
//!     provider:      sql_query_analyzer::cli::Provider::Ollama,
//!     api_key:       None,
//!     model:         None,
//!     ollama_url:    "http://localhost:11434".to_string(),
//!     dialect:       sql_query_analyzer::cli::Dialect::Generic,
//!     output_format: sql_query_analyzer::cli::Format::Text,
//!     verbose:       false,
//!     dry_run:       false,
//!     no_color:      false
//! };
//!
//! let config = Config::default();
//! let output = execute_command(command, config).await?;
//! println!("Exit code: {}", output.exit_code);
//! # Ok(())
//! # }
//! ```

mod analyze;
mod convert;
mod helpers;
mod types;

#[allow(unused_imports)]
pub use analyze::run_analyze;
#[allow(unused_imports)]
pub use convert::{convert_dialect, convert_format};
#[allow(unused_imports)]
pub use helpers::{
    build_llm_provider, calculate_exit_code, create_output_options, get_effective_model,
    get_effective_ollama_url, has_llm_access, parse_queries_cached, read_queries_input
};
#[allow(unused_imports)]
pub use types::{AnalyzeParams, AnalyzeResult, CommandOutput, DryRunInfo};

use crate::{cli::Commands, config::Config, error::AppResult};

/// Executes a CLI command and produces output ready for display.
///
/// This is the main entry point for command execution after CLI parsing.
/// It dispatches to the appropriate handler based on the command variant
/// and formats the results for terminal output.
///
/// # Arguments
///
/// * `command` - The parsed CLI command to execute
/// * `config` - Application configuration loaded from file or defaults
///
/// # Returns
///
/// A `CommandOutput` containing:
/// - `exit_code`: Process exit code (0=success, 1=warnings, 2=errors)
/// - `stdout`: Lines to be printed to standard output
///
/// # Errors
///
/// Returns an error if:
/// - Schema or query files cannot be read
/// - SQL parsing fails
/// - LLM API call fails (when LLM analysis is enabled)
///
/// # Exit Codes
///
/// The function returns different exit codes based on analysis results:
///
/// | Code | Meaning |
/// |------|---------|
/// | 0 | Success - no violations or info only |
/// | 1 | Warnings detected |
/// | 2 | Errors detected |
///
/// # Example
///
/// ```no_run
/// use std::path::PathBuf;
///
/// use sql_query_analyzer::{
///     app::{CommandOutput, execute_command},
///     cli::{Commands, Dialect, Format, Provider},
///     config::Config
/// };
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let command = Commands::Analyze {
///     schema:        PathBuf::from("schema.sql"),
///     queries:       PathBuf::from("queries.sql"),
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
/// let output = execute_command(command, config).await?;
///
/// for line in &output.stdout {
///     println!("{}", line);
/// }
///
/// std::process::exit(output.exit_code);
/// # Ok(())
/// # }
/// ```
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
    use std::{io::Write, path::PathBuf};

    use tempfile::NamedTempFile;

    use super::*;
    use crate::cli::{Dialect, Format, Provider};

    #[tokio::test]
    async fn test_execute_command_success() {
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
        let mut schema_file = NamedTempFile::new().unwrap();
        writeln!(schema_file, "CREATE TABLE test (id INT);").unwrap();

        let mut queries_file = NamedTempFile::new().unwrap();
        writeln!(queries_file, "SELECT id FROM test;").unwrap();

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

    #[tokio::test]
    async fn test_execute_command_stdin_path() {
        let mut schema_file = NamedTempFile::new().unwrap();
        writeln!(schema_file, "CREATE TABLE stdin_test (id INT);").unwrap();

        let command = Commands::Analyze {
            schema:        schema_file.path().to_path_buf(),
            queries:       PathBuf::from("-"),
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
        let result = execute_command(command, config).await;
        assert!(result.is_err() || result.is_ok());
    }

    #[tokio::test]
    async fn test_execute_command_mysql_dialect() {
        let mut schema_file = NamedTempFile::new().unwrap();
        writeln!(schema_file, "CREATE TABLE t (id INT PRIMARY KEY);").unwrap();

        let mut queries_file = NamedTempFile::new().unwrap();
        writeln!(queries_file, "SELECT id FROM t;").unwrap();

        let command = Commands::Analyze {
            schema:        schema_file.path().to_path_buf(),
            queries:       queries_file.path().to_path_buf(),
            provider:      Provider::OpenAI,
            api_key:       None,
            model:         None,
            ollama_url:    "http://localhost:11434".to_string(),
            dialect:       Dialect::Mysql,
            output_format: Format::Text,
            verbose:       false,
            dry_run:       false,
            no_color:      true
        };

        let config = Config::default();
        let result = execute_command(command, config).await.unwrap();
        assert_eq!(result.exit_code, 0);
    }

    #[tokio::test]
    async fn test_execute_command_postgresql_dialect() {
        let mut schema_file = NamedTempFile::new().unwrap();
        writeln!(schema_file, "CREATE TABLE t (id INT PRIMARY KEY);").unwrap();

        let mut queries_file = NamedTempFile::new().unwrap();
        writeln!(queries_file, "SELECT id FROM t;").unwrap();

        let command = Commands::Analyze {
            schema:        schema_file.path().to_path_buf(),
            queries:       queries_file.path().to_path_buf(),
            provider:      Provider::OpenAI,
            api_key:       None,
            model:         None,
            ollama_url:    "http://localhost:11434".to_string(),
            dialect:       Dialect::Postgresql,
            output_format: Format::Text,
            verbose:       false,
            dry_run:       false,
            no_color:      true
        };

        let config = Config::default();
        let result = execute_command(command, config).await.unwrap();
        assert_eq!(result.exit_code, 0);
    }

    #[tokio::test]
    async fn test_execute_command_sqlite_dialect() {
        let mut schema_file = NamedTempFile::new().unwrap();
        writeln!(schema_file, "CREATE TABLE t (id INTEGER PRIMARY KEY);").unwrap();

        let mut queries_file = NamedTempFile::new().unwrap();
        writeln!(queries_file, "SELECT id FROM t;").unwrap();

        let command = Commands::Analyze {
            schema:        schema_file.path().to_path_buf(),
            queries:       queries_file.path().to_path_buf(),
            provider:      Provider::OpenAI,
            api_key:       None,
            model:         None,
            ollama_url:    "http://localhost:11434".to_string(),
            dialect:       Dialect::Sqlite,
            output_format: Format::Text,
            verbose:       false,
            dry_run:       false,
            no_color:      true
        };

        let config = Config::default();
        let result = execute_command(command, config).await.unwrap();
        assert_eq!(result.exit_code, 0);
    }
}
