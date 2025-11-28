//! Error types and constructors for the SQL query analyzer.
//!
//! This module provides error construction functions that create properly
//! formatted [`AppError`] instances with context-specific messages.
//!
//! # Error Categories
//!
//! - **File errors**: IO failures when reading schema/query files
//! - **Parse errors**: SQL parsing failures with position information
//! - **LLM errors**: API communication failures with retry support
//! - **Config errors**: Invalid configuration files or values

pub use masterror::{AppError, AppResult};

/// Create file read error with path context.
///
/// # Arguments
///
/// * `path` - The file path that failed to read
/// * `source` - The underlying IO error
pub fn file_read_error(path: &str, source: std::io::Error) -> AppError {
    AppError::internal(format!("Failed to read file '{}': {}", path, source))
}

/// Create schema parse error with optional position info
pub fn schema_parse_error(message: impl Into<String>) -> AppError {
    let msg = message.into();
    AppError::bad_request(format_sql_error("Schema parse error", &msg))
}

/// Create query parse error with optional position info
pub fn query_parse_error(message: impl Into<String>) -> AppError {
    let msg = message.into();
    AppError::bad_request(format_sql_error("Query parse error", &msg))
}

/// Create LLM API error
pub fn llm_api_error(message: impl Into<String>) -> AppError {
    AppError::service(message.into())
}

/// Create HTTP error
pub fn http_error(err: reqwest::Error) -> AppError {
    let msg = if err.is_timeout() {
        format!("Request timeout: {}", err)
    } else if err.is_connect() {
        format!("Connection failed: {}", err)
    } else if err.is_status() {
        format!("HTTP error {}: {}", err.status().unwrap_or_default(), err)
    } else {
        err.to_string()
    };
    AppError::service(msg)
}

/// Create config error
pub fn config_error(message: impl Into<String>) -> AppError {
    AppError::bad_request(message.into())
}

/// Format SQL error with position highlighting
///
/// # Notes
///
/// - Attempts to extract line and column information from sqlparser errors
/// - Uses "Line: X, Column Y" pattern matching
fn format_sql_error(prefix: &str, message: &str) -> String {
    if let Some(pos) = extract_position(message) {
        format!(
            "{prefix} at line {line}, column {column}:\n  {message}",
            prefix = prefix,
            line = pos.line,
            column = pos.column,
            message = message
        )
    } else {
        format!("{}:\n  {}", prefix, message)
    }
}

struct SqlPosition {
    line:   usize,
    column: usize
}

/// Extract position from sqlparser error message
///
/// # Notes
///
/// - Looks for "Line: X, Column Y" pattern in error messages
fn extract_position(message: &str) -> Option<SqlPosition> {
    let line_marker = "Line: ";
    let col_marker = ", Column ";
    let line_start = message.find(line_marker)?;
    let line_num_start = line_start + line_marker.len();
    let rest = message.get(line_num_start..)?;
    let col_start = rest.find(col_marker)?;
    let line_str = message.get(line_num_start..line_num_start + col_start)?;
    let col_num_start = line_num_start + col_start + col_marker.len();
    let col_rest = message.get(col_num_start..)?;
    let col_end = col_rest
        .find(|c: char| !c.is_ascii_digit())
        .unwrap_or(col_rest.len());
    let col_str = message.get(col_num_start..col_num_start + col_end)?;
    let line = line_str.parse().ok()?;
    let column = col_str.parse().ok()?;
    Some(SqlPosition {
        line,
        column
    })
}
