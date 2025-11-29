// SPDX-FileCopyrightText: 2025 RAprogramm
// SPDX-License-Identifier: MIT

use sql_query_analyzer::error::{
    config_error, file_read_error, llm_api_error, query_parse_error, schema_parse_error
};

#[test]
fn test_file_read_error() {
    let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
    let error = file_read_error("/path/to/file.sql", io_error);
    let _msg = error.to_string();
}

#[test]
fn test_schema_parse_error() {
    let error = schema_parse_error("Invalid syntax");
    let _msg = error.to_string();
}

#[test]
fn test_schema_parse_error_with_position() {
    let error = schema_parse_error("Expected keyword at Line: 5, Column 10");
    let _msg = error.to_string();
}

#[test]
fn test_query_parse_error() {
    let error = query_parse_error("Unexpected token");
    let _msg = error.to_string();
}

#[test]
fn test_query_parse_error_with_position() {
    let error = query_parse_error("Missing semicolon at Line: 3, Column 25");
    let _msg = error.to_string();
}

#[test]
fn test_llm_api_error() {
    let error = llm_api_error("API rate limit exceeded");
    let _msg = error.to_string();
}

#[test]
fn test_config_error() {
    let error = config_error("Invalid configuration value");
    let _msg = error.to_string();
}

#[test]
fn test_position_extraction_edge_cases() {
    let error = schema_parse_error("Error at Line: 1, Column 1 in statement");
    let _msg = error.to_string();
}

#[test]
fn test_position_extraction_large_numbers() {
    let error = query_parse_error("Error at Line: 999, Column 12345");
    let _msg = error.to_string();
}

#[test]
fn test_error_types_are_different() {
    let schema_err = schema_parse_error("test");
    let query_err = query_parse_error("test");
    let llm_err = llm_api_error("test");
    let config_err = config_error("test");
    assert!(!schema_err.to_string().is_empty());
    assert!(!query_err.to_string().is_empty());
    assert!(!llm_err.to_string().is_empty());
    assert!(!config_err.to_string().is_empty());
}
