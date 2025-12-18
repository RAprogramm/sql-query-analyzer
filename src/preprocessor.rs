//! SQL dialect-specific preprocessing.
//!
//! This module transforms SQL statements to remove or normalize constructs
//! that are not supported by the `sqlparser` crate, while preserving the
//! semantic meaning for analysis.
//!
//! # Supported Dialects
//!
//! - **ClickHouse**: Handles `CODEC`, `TTL`, `SETTINGS` clauses
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

/// Metadata extracted during preprocessing.
#[derive(Debug, Default, Clone)]
pub struct PreprocessorMetadata {
    /// Column codecs: column_name -> codec_expression
    pub codecs:          HashMap<String, String>,
    /// TTL expressions extracted from table definitions
    pub ttl_expressions: Vec<String>,
    /// Table settings: setting_name -> value
    pub settings:        HashMap<String, String>,
    /// Partition expressions (ClickHouse PARTITION BY)
    pub partition_by:    Vec<String>
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
    pub fn new(dialect: SqlDialect) -> Self {
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
}
