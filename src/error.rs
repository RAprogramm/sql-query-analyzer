pub use masterror::{AppError, AppResult};

/// Create file read error
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
fn format_sql_error(prefix: &str, message: &str) -> String {
    // Try to extract line and column information from sqlparser errors
    // sqlparser format: "... at Line: X, Column Y"
    if let Some(pos) = extract_position(message) {
        format!(
            "{} at line {}, column {}:\n  {}",
            prefix, pos.line, pos.column, message
        )
    } else {
        format!("{}:\n  {}", prefix, message)
    }
}

struct SqlPosition {
    line:   usize,
    column: usize
}

fn extract_position(message: &str) -> Option<SqlPosition> {
    // Look for "Line: X, Column Y" pattern
    let line_marker = "Line: ";
    let col_marker = ", Column ";

    if let Some(line_start) = message.find(line_marker) {
        let line_num_start = line_start + line_marker.len();
        if let Some(col_start) = message[line_num_start..].find(col_marker) {
            let line_str = &message[line_num_start..line_num_start + col_start];
            let col_num_start = line_num_start + col_start + col_marker.len();

            // Find end of column number
            let col_end = message[col_num_start..]
                .find(|c: char| !c.is_ascii_digit())
                .unwrap_or(message.len() - col_num_start);

            let col_str = &message[col_num_start..col_num_start + col_end];

            if let (Ok(line), Ok(column)) = (line_str.parse(), col_str.parse()) {
                return Some(SqlPosition { line, column });
            }
        }
    }

    None
}
