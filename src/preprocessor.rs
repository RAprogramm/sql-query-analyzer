//! SQL dialect-specific preprocessing.
//!
//! This module transforms SQL statements to remove or normalize constructs
//! that are not supported by the `sqlparser` crate, while preserving the
//! semantic meaning for analysis.
//!
//! # Supported Dialects
//!
//! - **`ClickHouse`**: Handles `CODEC`, `TTL`, `SETTINGS` clauses
//!
//! # Architecture
//!
//! The preprocessor operates in two phases:
//! 1. **Extraction**: Captures dialect-specific metadata (codecs, TTL rules)
//! 2. **Transformation**: Removes unsupported syntax for clean parsing
//!
//! # Example
//!
//! ```
//! use sql_query_analyzer::{preprocessor::Preprocessor, query::SqlDialect};
//!
//! let sql = "CREATE TABLE logs (data String CODEC(ZSTD)) ENGINE = MergeTree ORDER BY id";
//! let result = Preprocessor::new(SqlDialect::ClickHouse).process(sql);
//!
//! assert!(!result.sql.contains("CODEC"));
//! assert!(result.metadata.codecs.contains_key("data"));
//! ```

pub mod clickhouse;

use std::collections::HashMap;

use crate::query::SqlDialect;

/// Preprocessor for dialect-specific SQL transformations.
#[derive(Debug)]
pub struct Preprocessor {
    dialect: SqlDialect
}

pub use crate::preprocessor::clickhouse::{MaterializedViewInfo, ProjectionInfo, SkipIndexInfo};

/// Metadata extracted during preprocessing.
#[derive(Debug, Default, Clone)]
pub struct PreprocessorMetadata {
    /// Column codecs: `column_name` -> `codec_expression`
    pub codecs:             HashMap<String, String>,
    /// TTL expressions extracted from table definitions
    pub ttl_expressions:    Vec<String>,
    /// Table settings: `setting_name` -> value
    pub settings:           HashMap<String, String>,
    /// Partition expressions (`ClickHouse` PARTITION BY)
    pub partition_by:       Vec<String>,
    /// Sample by expressions (`ClickHouse` SAMPLE BY)
    pub sample_by:          Vec<String>,
    /// Data skipping indexes (`ClickHouse` INDEX ... TYPE ... GRANULARITY)
    pub skip_indexes:       Vec<SkipIndexInfo>,
    /// Projections (`ClickHouse` PROJECTION ... AS SELECT)
    pub projections:        Vec<ProjectionInfo>,
    /// Dictionary names (`ClickHouse` CREATE DICTIONARY)
    pub dictionaries:       Vec<String>,
    /// Materialized views (`ClickHouse` CREATE MATERIALIZED VIEW)
    pub materialized_views: Vec<MaterializedViewInfo>
}

/// Result of SQL preprocessing.
#[derive(Debug)]
pub struct PreprocessorResult {
    /// Transformed SQL ready for parsing
    pub sql:      String,
    /// Extracted metadata
    pub metadata: PreprocessorMetadata
}

impl Preprocessor {
    /// Create a new preprocessor for the specified dialect.
    #[must_use]
    pub const fn new(dialect: SqlDialect) -> Self {
        Self {
            dialect
        }
    }

    /// Process SQL and return transformed result with metadata.
    #[must_use]
    pub fn process(&self, sql: &str) -> PreprocessorResult {
        match self.dialect {
            SqlDialect::ClickHouse => clickhouse::preprocess(sql),
            _ => PreprocessorResult {
                sql:      sql.to_string(),
                metadata: PreprocessorMetadata::default()
            }
        }
    }
}

/// Split SQL into individual statements at top-level semicolons.
///
/// Semicolons inside single-quoted strings, double-quoted identifiers,
/// backtick identifiers, `--` line comments and block comments do not split.
/// Doubled quotes escape inside their own quote kind for every dialect.
/// `backslash_escapes` selects whether `\` escapes inside single and double
/// quotes, matching the rules of the dialect being split (see
/// [`SqlDialect::uses_backslash_escapes`]). Every split point is an ASCII
/// byte, so each returned fragment is a valid `&str`. Empty and
/// whitespace-only fragments are dropped.
#[must_use]
pub fn split_statements(sql: &str, backslash_escapes: bool) -> Vec<&str> {
    let bytes = sql.as_bytes();
    let mut statements = Vec::new();
    let mut start = 0;
    let mut index = 0;
    while index < bytes.len() {
        match bytes[index] {
            b'\'' | b'"' | b'`' => index = skip_quoted(bytes, index, backslash_escapes),
            b'-' if bytes.get(index + 1) == Some(&b'-') => index = skip_line_comment(bytes, index),
            b'/' if bytes.get(index + 1) == Some(&b'*') => {
                index = skip_block_comment(bytes, index);
            }
            b';' => {
                push_fragment(&mut statements, &sql[start..index]);
                index += 1;
                start = index;
            }
            _ => index += 1
        }
    }
    push_fragment(&mut statements, &sql[start..]);
    statements
}

/// Return the index just past the quoted run starting at `open`.
fn skip_quoted(bytes: &[u8], open: usize, backslash_escapes: bool) -> usize {
    let quote = bytes[open];
    let mut index = open + 1;
    while index < bytes.len() {
        let byte = bytes[index];
        if backslash_escapes && quote != b'`' && byte == b'\\' {
            index += 2;
            continue;
        }
        if byte == quote {
            if bytes.get(index + 1) == Some(&quote) {
                index += 2;
                continue;
            }
            return index + 1;
        }
        index += 1;
    }
    bytes.len()
}

/// Return the index of the newline ending the `--` comment, or end of input.
fn skip_line_comment(bytes: &[u8], start: usize) -> usize {
    bytes[start..]
        .iter()
        .position(|&byte| byte == b'\n')
        .map_or(bytes.len(), |offset| start + offset)
}

/// Return the index just past the `*/` ending the block comment.
fn skip_block_comment(bytes: &[u8], start: usize) -> usize {
    let search_from = (start + 2).min(bytes.len());
    bytes[search_from..]
        .windows(2)
        .position(|window| window == b"*/")
        .map_or(bytes.len(), |offset| search_from + offset + 2)
}

/// Trim a fragment and push it unless it is empty.
fn push_fragment<'a>(statements: &mut Vec<&'a str>, fragment: &'a str) {
    let trimmed = fragment.trim();
    if !trimmed.is_empty() {
        statements.push(trimmed);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_preprocessor_generic_passthrough() {
        let sql = "CREATE TABLE users (id INT PRIMARY KEY)";
        let result = Preprocessor::new(SqlDialect::Generic).process(sql);
        assert_eq!(result.sql, sql);
        assert!(result.metadata.codecs.is_empty());
    }

    #[test]
    fn test_preprocessor_clickhouse_removes_codec() {
        let sql = "CREATE TABLE logs (data String CODEC(ZSTD)) ENGINE = MergeTree ORDER BY id";
        let result = Preprocessor::new(SqlDialect::ClickHouse).process(sql);
        assert!(!result.sql.contains("CODEC"));
    }

    #[test]
    fn test_preprocessor_metadata_extraction() {
        let sql = "CREATE TABLE t (col String CODEC(LZ4)) ENGINE = MergeTree ORDER BY col";
        let result = Preprocessor::new(SqlDialect::ClickHouse).process(sql);
        assert!(result.metadata.codecs.contains_key("col"));
        assert_eq!(result.metadata.codecs.get("col"), Some(&"LZ4".to_string()));
    }

    #[test]
    fn test_split_statements_basic_and_empty_fragments() {
        assert_eq!(
            split_statements("SELECT 1; SELECT 2;", true),
            vec!["SELECT 1", "SELECT 2"]
        );
        assert_eq!(split_statements(";; SELECT 1 ;;", true), vec!["SELECT 1"]);
        assert_eq!(split_statements("", true), Vec::<&str>::new());
        assert_eq!(split_statements("   ;  \n ;", true), Vec::<&str>::new());
    }

    #[test]
    fn test_split_statements_keeps_semicolon_in_quotes() {
        assert_eq!(
            split_statements("INSERT INTO t VALUES ('a;b'); SELECT \"c;d\" FROM t", true),
            vec!["INSERT INTO t VALUES ('a;b')", "SELECT \"c;d\" FROM t"]
        );
        assert_eq!(
            split_statements("SELECT `e;f` FROM t; SELECT 2", true),
            vec!["SELECT `e;f` FROM t", "SELECT 2"]
        );
        assert_eq!(
            split_statements("INSERT INTO t VALUES ('it''s; ok'); SELECT 2", true),
            vec!["INSERT INTO t VALUES ('it''s; ok')", "SELECT 2"]
        );
        assert_eq!(
            split_statements("INSERT INTO t VALUES ('it\\'s; ok'); SELECT 2", true),
            vec!["INSERT INTO t VALUES ('it\\'s; ok')", "SELECT 2"]
        );
    }

    #[test]
    fn test_split_statements_backslash_escapes_follow_dialect() {
        let sql = "INSERT INTO t VALUES ('C:\\'); INSERT INTO u VALUES ('a;b')";
        assert_eq!(
            split_statements(sql, false),
            vec![
                "INSERT INTO t VALUES ('C:\\')",
                "INSERT INTO u VALUES ('a;b')"
            ]
        );
        let escaped = split_statements(sql, true);
        assert_eq!(escaped.len(), 2);
        assert!(escaped[0].contains("INSERT INTO u"), "{escaped:?}");
    }

    #[test]
    fn test_split_statements_keeps_semicolon_in_comments() {
        assert_eq!(
            split_statements("SELECT 1 -- trailing ; note\n; SELECT 2", true),
            vec!["SELECT 1 -- trailing ; note", "SELECT 2"]
        );
        assert_eq!(
            split_statements("SELECT /* ; */ 1; SELECT 2", true),
            vec!["SELECT /* ; */ 1", "SELECT 2"]
        );
    }

    #[test]
    fn test_split_statements_multibyte_is_safe() {
        assert_eq!(
            split_statements("SELECT 'ёжик;'; SELECT \"лош;адь\"", true),
            vec!["SELECT 'ёжик;'", "SELECT \"лош;адь\""]
        );
    }
}
