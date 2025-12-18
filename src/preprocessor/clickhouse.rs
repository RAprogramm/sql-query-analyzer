//! ClickHouse-specific SQL preprocessing.
//!
//! Handles ClickHouse DDL constructs not supported by sqlparser:
//! - `CODEC(...)` - Column compression codecs
//! - `TTL ...` - Data expiration rules
//! - `SETTINGS ...` - Table-level settings
//!
//! # Codec Syntax
//!
//! ClickHouse supports various compression codecs:
//! ```sql
//! CREATE TABLE t (
//!     col1 String CODEC(ZSTD),
//!     col2 UInt64 CODEC(Delta, LZ4),
//!     col3 DateTime CODEC(DoubleDelta, ZSTD(3))
//! )
//! ```
//!
//! # TTL Syntax
//!
//! ```sql
//! CREATE TABLE t (...)
//! ENGINE = MergeTree
//! TTL event_date + INTERVAL 90 DAY
//! ```

use std::sync::LazyLock;

use regex::Regex;

use super::{PreprocessorMetadata, PreprocessorResult};

/// Regex for matching CODEC clauses with optional nested parentheses.
/// Matches: `CODEC(ZSTD)`, `CODEC(Delta, LZ4)`, `CODEC(ZSTD(3))`
static CODEC_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)\s+CODEC\s*\(([^()]*(?:\([^()]*\)[^()]*)*)\)").expect("valid regex")
});

/// Regex for extracting column name before CODEC.
/// Captures the identifier immediately before CODEC keyword.
static COLUMN_CODEC_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)(\w+)\s+\w+(?:\([^)]*\))?\s+CODEC\s*\(([^()]*(?:\([^()]*\)[^()]*)*)\)")
        .expect("valid regex")
});

/// Regex for matching TTL clauses.
/// Matches: `TTL expr + INTERVAL n UNIT` up to SETTINGS or end
static TTL_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)\bTTL\s+(.+?)(?:\s+SETTINGS\b|;|$)").expect("valid regex"));

/// Regex for matching SETTINGS clauses.
/// Matches: `SETTINGS key = value, key2 = value2`
static SETTINGS_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)\bSETTINGS\s+([^;]+)").expect("valid regex"));

/// Regex for matching PARTITION BY clauses (ClickHouse DDL).
static PARTITION_BY_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)\bPARTITION\s+BY\s+(\S+(?:\([^)]*\))?)").expect("valid regex")
});

/// Regex for individual setting key-value pairs.
static SETTING_PAIR_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(\w+)\s*=\s*('[^']*'|\d+)").expect("valid regex"));

/// Preprocess ClickHouse SQL.
///
/// Removes unsupported constructs and extracts metadata.
pub fn preprocess(sql: &str) -> PreprocessorResult {
    let mut metadata = PreprocessorMetadata::default();
    let mut result = sql.to_string();

    extract_codecs(&result, &mut metadata);
    extract_ttl(&result, &mut metadata);
    extract_settings(&result, &mut metadata);
    extract_partition_by(&result, &mut metadata);

    result = remove_codecs(&result);
    result = remove_ttl(&result);
    result = remove_settings(&result);
    result = remove_partition_by(&result);

    result = normalize_whitespace(&result);

    PreprocessorResult {
        sql: result,
        metadata
    }
}

/// Extract CODEC metadata from SQL.
fn extract_codecs(sql: &str, metadata: &mut PreprocessorMetadata) {
    for cap in COLUMN_CODEC_REGEX.captures_iter(sql) {
        let column_name = cap.get(1).map(|m| m.as_str().to_string());
        let codec_expr = cap.get(2).map(|m| m.as_str().trim().to_string());
        if let (Some(col), Some(codec)) = (column_name, codec_expr) {
            metadata.codecs.insert(col, codec);
        }
    }
}

/// Extract TTL expressions from SQL.
fn extract_ttl(sql: &str, metadata: &mut PreprocessorMetadata) {
    for cap in TTL_REGEX.captures_iter(sql) {
        if let Some(ttl_expr) = cap.get(1) {
            metadata
                .ttl_expressions
                .push(ttl_expr.as_str().trim().to_string());
        }
    }
}

/// Extract SETTINGS from SQL.
fn extract_settings(sql: &str, metadata: &mut PreprocessorMetadata) {
    for cap in SETTINGS_REGEX.captures_iter(sql) {
        if let Some(settings_str) = cap.get(1) {
            for pair in SETTING_PAIR_REGEX.captures_iter(settings_str.as_str()) {
                let key = pair.get(1).map(|m| m.as_str().to_string());
                let value = pair
                    .get(2)
                    .map(|m| m.as_str().trim_matches('\'').to_string());
                if let (Some(k), Some(v)) = (key, value) {
                    metadata.settings.insert(k, v);
                }
            }
        }
    }
}

/// Extract PARTITION BY expressions from SQL.
fn extract_partition_by(sql: &str, metadata: &mut PreprocessorMetadata) {
    for cap in PARTITION_BY_REGEX.captures_iter(sql) {
        if let Some(expr) = cap.get(1) {
            metadata.partition_by.push(expr.as_str().trim().to_string());
        }
    }
}

/// Remove CODEC clauses from SQL.
fn remove_codecs(sql: &str) -> String {
    CODEC_REGEX.replace_all(sql, "").to_string()
}

/// Remove TTL clauses from SQL.
fn remove_ttl(sql: &str) -> String {
    TTL_REGEX.replace_all(sql, "").to_string()
}

/// Remove SETTINGS clauses from SQL.
fn remove_settings(sql: &str) -> String {
    SETTINGS_REGEX.replace_all(sql, "").to_string()
}

/// Remove PARTITION BY clauses from SQL.
fn remove_partition_by(sql: &str) -> String {
    PARTITION_BY_REGEX.replace_all(sql, "").to_string()
}

/// Normalize excessive whitespace.
fn normalize_whitespace(sql: &str) -> String {
    let re = Regex::new(r"\s+").expect("valid regex");
    re.replace_all(sql.trim(), " ").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_codec_removal() {
        let sql = "CREATE TABLE t (col String CODEC(ZSTD)) ENGINE = MergeTree ORDER BY col";
        let result = preprocess(sql);
        assert!(!result.sql.contains("CODEC"));
        assert!(result.sql.contains("col String"));
    }

    #[test]
    fn test_codec_extraction() {
        let sql = "CREATE TABLE t (data String CODEC(LZ4)) ENGINE = MergeTree ORDER BY data";
        let result = preprocess(sql);
        assert_eq!(result.metadata.codecs.get("data"), Some(&"LZ4".to_string()));
    }

    #[test]
    fn test_multiple_codecs() {
        let sql = r#"
            CREATE TABLE t (
                col1 String CODEC(ZSTD),
                col2 UInt64 CODEC(Delta, LZ4)
            ) ENGINE = MergeTree ORDER BY col1
        "#;
        let result = preprocess(sql);
        assert!(!result.sql.contains("CODEC"));
        assert_eq!(
            result.metadata.codecs.get("col1"),
            Some(&"ZSTD".to_string())
        );
        assert_eq!(
            result.metadata.codecs.get("col2"),
            Some(&"Delta, LZ4".to_string())
        );
    }

    #[test]
    fn test_nested_codec_params() {
        let sql = "CREATE TABLE t (col String CODEC(ZSTD(3))) ENGINE = MergeTree ORDER BY col";
        let result = preprocess(sql);
        assert!(!result.sql.contains("CODEC"));
        assert_eq!(
            result.metadata.codecs.get("col"),
            Some(&"ZSTD(3)".to_string())
        );
    }

    #[test]
    fn test_ttl_extraction() {
        let sql = "CREATE TABLE t (d Date) ENGINE = MergeTree ORDER BY d TTL d + INTERVAL 90 DAY";
        let result = preprocess(sql);
        assert!(!result.sql.contains("TTL"));
        assert_eq!(result.metadata.ttl_expressions.len(), 1);
        assert!(result.metadata.ttl_expressions[0].contains("INTERVAL 90 DAY"));
    }

    #[test]
    fn test_settings_extraction() {
        let sql = "CREATE TABLE t (id UInt64) ENGINE = MergeTree ORDER BY id SETTINGS index_granularity = 8192";
        let result = preprocess(sql);
        assert!(!result.sql.contains("SETTINGS"));
        assert_eq!(
            result.metadata.settings.get("index_granularity"),
            Some(&"8192".to_string())
        );
    }

    #[test]
    fn test_complex_clickhouse_ddl() {
        let sql = r#"
            CREATE TABLE events ON CLUSTER default (
                event_date Date,
                event_time DateTime CODEC(Delta, ZSTD),
                user_id UInt64 CODEC(T64),
                data String CODEC(ZSTD(3))
            ) ENGINE = ReplicatedMergeTree('/clickhouse/tables/{shard}/events', '{replica}')
            PARTITION BY toYYYYMM(event_date)
            ORDER BY (event_date, user_id)
            TTL event_date + INTERVAL 90 DAY
            SETTINGS index_granularity = 8192
        "#;
        let result = preprocess(sql);

        assert!(!result.sql.contains("CODEC"));
        assert!(!result.sql.contains("TTL"));
        assert!(!result.sql.contains("SETTINGS"));
        assert!(result.sql.contains("ENGINE = ReplicatedMergeTree"));
        assert!(result.sql.contains("ORDER BY"));

        assert_eq!(result.metadata.codecs.len(), 3);
        assert_eq!(result.metadata.ttl_expressions.len(), 1);
        assert_eq!(
            result.metadata.settings.get("index_granularity"),
            Some(&"8192".to_string())
        );
    }

    #[test]
    fn test_no_modification_without_special_syntax() {
        let sql = "CREATE TABLE t (id UInt64) ENGINE = MergeTree ORDER BY id";
        let result = preprocess(sql);
        assert_eq!(result.sql, sql);
        assert!(result.metadata.codecs.is_empty());
        assert!(result.metadata.ttl_expressions.is_empty());
        assert!(result.metadata.settings.is_empty());
    }
}
