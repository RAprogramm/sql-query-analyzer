mod cache;
mod cli;
mod config;
mod error;
mod llm;
mod output;
mod query;
mod schema;

use std::fs;

use clap::Parser;
use indicatif::{ProgressBar, ProgressStyle};

use crate::{
    cli::{Cli, Commands, Dialect, Format, Provider},
    config::Config,
    error::{AppResult, config_error, file_read_error},
    llm::{LlmClient, LlmProvider},
    output::{format_analysis_result, format_queries_summary, OutputFormat, OutputOptions},
    query::{parse_queries, SqlDialect},
    schema::Schema
};

#[tokio::main]
async fn main() -> AppResult<()> {
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
            let schema_sql = fs::read_to_string(&schema)
                .map_err(|e| file_read_error(&schema.display().to_string(), e))?;

            let queries_sql = fs::read_to_string(&queries)
                .map_err(|e| file_read_error(&queries.display().to_string(), e))?;

            // Convert CLI dialect to query dialect
            let sql_dialect = match dialect {
                Dialect::Generic => SqlDialect::Generic,
                Dialect::Mysql => SqlDialect::MySQL,
                Dialect::Postgresql => SqlDialect::PostgreSQL,
                Dialect::Sqlite => SqlDialect::SQLite
            };

            let parsed_schema = Schema::parse(&schema_sql)?;

            // Use cache for queries
            let parsed_queries = if let Some(cached) = cache::get_cached(&queries_sql) {
                cached
            } else {
                let queries = parse_queries(&queries_sql, sql_dialect)?;
                cache::cache_queries(&queries_sql, queries.clone());
                queries
            };

            let schema_summary = parsed_schema.to_summary();

            // Setup output options
            let output_opts = OutputOptions {
                format: match output_format {
                    Format::Text => OutputFormat::Text,
                    Format::Json => OutputFormat::Json,
                    Format::Yaml => OutputFormat::Yaml
                },
                colored: !no_color,
                verbose
            };

            let queries_summary = format_queries_summary(&parsed_queries, &output_opts);

            // Dry run mode - show what would be sent
            if dry_run {
                println!("=== DRY RUN - Would send to LLM ===\n");
                println!("Schema Summary:\n{}\n", schema_summary);
                println!("Queries Summary:\n{}", queries_summary);
                return Ok(());
            }

            // Use CLI args, then config, then defaults
            let model_name = model
                .or(config.llm.model.clone())
                .unwrap_or_else(|| provider.default_model().to_string());

            let effective_api_key = api_key.or(config.llm.api_key.clone());
            let effective_ollama_url = if ollama_url == "http://localhost:11434" {
                config.llm.ollama_url.clone().unwrap_or(ollama_url)
            } else {
                ollama_url
            };

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
            pb.set_style(
                ProgressStyle::default_spinner()
                    .template("{spinner:.green} {msg}")
                    .unwrap()
            );
            pb.set_message("Analyzing queries with LLM...");
            pb.enable_steady_tick(std::time::Duration::from_millis(100));

            let client = LlmClient::with_retry_config(llm_provider, config.retry);
            let analysis = client.analyze(&schema_summary, &queries_summary).await?;

            pb.finish_and_clear();

            let output = format_analysis_result(&parsed_queries, &analysis, &output_opts);
            println!("{}", output);
        }
    }

    Ok(())
}
