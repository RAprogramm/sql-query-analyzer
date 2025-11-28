// SPDX-FileCopyrightText: 2025 RAprogramm
// SPDX-License-Identifier: MIT

use sql_query_analyzer::cli::{Dialect, Format, Provider};

#[test]
fn test_provider_default_model_openai() {
    let provider = Provider::OpenAI;
    assert_eq!(provider.default_model(), "gpt-4");
}

#[test]
fn test_provider_default_model_anthropic() {
    let provider = Provider::Anthropic;
    assert_eq!(provider.default_model(), "claude-sonnet-4-20250514");
}

#[test]
fn test_provider_default_model_ollama() {
    let provider = Provider::Ollama;
    assert_eq!(provider.default_model(), "llama3.2");
}

#[test]
fn test_dialect_variants() {
    let _generic = Dialect::Generic;
    let _mysql = Dialect::Mysql;
    let _postgresql = Dialect::Postgresql;
    let _sqlite = Dialect::Sqlite;
}

#[test]
fn test_format_variants() {
    let _text = Format::Text;
    let _json = Format::Json;
    let _yaml = Format::Yaml;
    let _sarif = Format::Sarif;
}

#[test]
fn test_provider_clone() {
    let provider = Provider::OpenAI;
    let cloned = provider.clone();
    assert_eq!(cloned.default_model(), "gpt-4");
}

#[test]
fn test_dialect_clone() {
    let dialect = Dialect::Mysql;
    let _cloned = dialect.clone();
}

#[test]
fn test_format_clone() {
    let format = Format::Json;
    let _cloned = format.clone();
}

#[test]
fn test_provider_debug() {
    let provider = Provider::Ollama;
    let debug = format!("{:?}", provider);
    assert!(debug.contains("Ollama"));
}

#[test]
fn test_dialect_debug() {
    let dialect = Dialect::Postgresql;
    let debug = format!("{:?}", dialect);
    assert!(debug.contains("Postgresql"));
}

#[test]
fn test_format_debug() {
    let format = Format::Sarif;
    let debug = format!("{:?}", format);
    assert!(debug.contains("Sarif"));
}
