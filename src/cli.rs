use std::path::PathBuf;

use clap::{Parser, Subcommand, ValueEnum};

/// SQL Query Analyzer - Analyze SQL queries for optimization using LLM
#[derive(Parser, Debug)]
#[command(name = "sql-query-analyzer")]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Analyze SQL queries against schema
    Analyze {
        /// Path to SQL schema file
        #[arg(short, long)]
        schema: PathBuf,

        /// Path to SQL queries file (use - for stdin)
        #[arg(short, long)]
        queries: PathBuf,

        /// LLM provider to use
        #[arg(short, long, value_enum, default_value = "ollama")]
        provider: Provider,

        /// API key for OpenAI or Anthropic
        #[arg(short, long, env = "LLM_API_KEY")]
        api_key: Option<String>,

        /// Model name
        #[arg(short, long)]
        model: Option<String>,

        /// Ollama base URL
        #[arg(long, default_value = "http://localhost:11434")]
        ollama_url: String,

        /// SQL dialect for parsing
        #[arg(long, value_enum, default_value = "generic")]
        dialect: Dialect,

        /// Output format
        #[arg(short = 'f', long, value_enum, default_value = "text")]
        output_format: Format,

        /// Enable verbose output with complexity scores
        #[arg(short, long)]
        verbose: bool,

        /// Show what would be sent to LLM without making API call
        #[arg(long)]
        dry_run: bool,

        /// Disable colored output
        #[arg(long)]
        no_color: bool
    }
}

#[derive(Debug, Clone, ValueEnum)]
pub enum Provider {
    OpenAI,
    Anthropic,
    Ollama
}

impl Provider {
    /// Get default model for provider
    pub fn default_model(&self) -> &str {
        match self {
            Self::OpenAI => "gpt-4",
            Self::Anthropic => "claude-sonnet-4-20250514",
            Self::Ollama => "llama3.2"
        }
    }
}

#[derive(Debug, Clone, ValueEnum)]
pub enum Dialect {
    Generic,
    Mysql,
    Postgresql,
    Sqlite,
    Clickhouse
}

#[derive(Debug, Clone, ValueEnum)]
pub enum Format {
    Text,
    Json,
    Yaml,
    Sarif
}
