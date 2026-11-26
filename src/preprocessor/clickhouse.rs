//! ClickHouse-specific SQL preprocessing.
//!
//! Handles `ClickHouse` DDL constructs not supported by sqlparser:
//! - `CODEC(...)` - Column compression codecs
//! - `TTL ...` - Data expiration rules
//! - `SETTINGS ...` - Table-level settings
//!
//! # Codec Syntax
//!
//! `ClickHouse` supports various compression codecs:
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

/// Metadata for a `ClickHouse` data skipping index.
#[derive(Debug, Clone)]
pub struct SkipIndexInfo {
    /// Index name
    pub name:        String,
    /// Indexed expression (column or expression)
    pub expr:        String,
    /// Index type (minmax, set, `bloom_filter`, `ngrambf_v1`, `tokenbf_v1`,
    /// text)
    pub type_name:   String,
    /// Optional type arguments (e.g. "100" for set, "0.01" for `bloom_filter`)
    pub type_args:   Option<String>,
    /// Granularity in granules
    pub granularity: usize
}

/// Metadata for a `ClickHouse` projection.
#[derive(Debug, Clone)]
pub struct ProjectionInfo {
    /// Projection name
    pub name:  String,
    /// Inner `SELECT` definition (without the surrounding parentheses)
    pub query: String
}

/// Metadata for a `ClickHouse` materialized view.
#[derive(Debug, Clone)]
pub struct MaterializedViewInfo {
    /// View name
    pub name:   String,
    /// Destination table (`TO target`), when present
    pub target: Option<String>,
    /// Source `SELECT` feeding the view
    pub query:  String
}

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
    LazyLock::new(|| Regex::new(r"(?i)\bTTL\s+(.+?)(\s+SETTINGS\b|;|$)").expect("valid regex"));

/// Regex for matching SETTINGS clauses.
/// Matches: `SETTINGS key = value, key2 = value2`
static SETTINGS_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)\bSETTINGS\s+([^;]+)").expect("valid regex"));

/// Regex for matching PARTITION BY clauses (`ClickHouse` DDL).
static PARTITION_BY_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)\bPARTITION\s+BY\s+(\S+(?:\([^)]*\))?)").expect("valid regex")
});

/// Regex for matching SAMPLE BY clauses (`ClickHouse` DDL).
static SAMPLE_BY_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?is)\bSAMPLE\s+BY\s+(.+?)(\s+SETTINGS\b|\s+TTL\b|;|$)").expect("valid regex")
});

/// Regex for individual setting key-value pairs.
static SETTING_PAIR_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(\w+)\s*=\s*('[^']*'|\d+)").expect("valid regex"));

/// Regex for matching `ClickHouse` skipping index definitions.
/// Matches: `INDEX name expr TYPE type(args) GRANULARITY n`
static SKIP_INDEX_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?is)\bINDEX\s+(\w+)\s+(.+?)\s+TYPE\s+(\w+)(?:\s*\(([^)]*)\))?\s+GRANULARITY\s+(\d+)"
    )
    .expect("valid regex")
});

/// Regex for removing skipping index definitions (including optional leading
/// comma).
static SKIP_INDEX_REMOVE_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?is),?\s*\bINDEX\s+\w+\s+.+?\s+TYPE\s+\w+(?:\s*\([^)]*\))?\s+GRANULARITY\s+\d+")
        .expect("valid regex")
});

/// Regex for matching a `CREATE DICTIONARY` header.
/// Captures the dictionary name (group 1).
static DICTIONARY_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?i)\bCREATE\s+(?:OR\s+REPLACE\s+)?DICTIONARY\s+(?:IF\s+NOT\s+EXISTS\s+)?([\w.]+)"
    )
    .expect("valid regex")
});

/// Regex for removing a whole `CREATE DICTIONARY` statement (up to `;` or end).
static DICTIONARY_REMOVE_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?is)\bCREATE\s+(?:OR\s+REPLACE\s+)?DICTIONARY\b[^;]*;?").expect("valid regex")
});

/// Regex for matching a `CREATE MATERIALIZED VIEW` header.
/// Captures the view name (group 1) and optional `TO target` (group 2).
static MATERIALIZED_VIEW_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?i)\bCREATE\s+MATERIALIZED\s+VIEW\s+(?:IF\s+NOT\s+EXISTS\s+)?([\w.]+)(?:\s+TO\s+([\w.]+))?"
    )
    .expect("valid regex")
});

/// Regex for stripping the `ClickHouse` engine clause between a materialized
/// view header and its `AS SELECT` body, so the statement becomes parseable.
/// Group 1 keeps the header, group 3 keeps the `AS SELECT` keyword.
static MATERIALIZED_VIEW_ENGINE_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?is)(\bCREATE\s+MATERIALIZED\s+VIEW\s+(?:IF\s+NOT\s+EXISTS\s+)?[\w.]+(?:\s+TO\s+[\w.]+)?)(.*?)(\bAS\s+SELECT\b)")
        .expect("valid regex")
});

/// Regex for a `ClickHouse` `ALTER TABLE ... UPDATE ... WHERE ...` mutation.
static MUTATION_UPDATE_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?is)\bALTER\s+TABLE\s+([\w.]+)\s+UPDATE\s+(.+?)\s+WHERE\s+(.+?)(;|$)")
        .expect("valid regex")
});

/// Regex for a `ClickHouse` `ALTER TABLE ... DELETE WHERE ...` mutation.
static MUTATION_DELETE_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?is)\bALTER\s+TABLE\s+([\w.]+)\s+DELETE\s+WHERE\s+(.+?)(;|$)")
        .expect("valid regex")
});

/// Rewrite `ClickHouse` mutation statements into standard `UPDATE`/`DELETE` so
/// `sqlparser` can parse them.
///
/// `ClickHouse` expresses row mutations as `ALTER TABLE ... UPDATE/DELETE`,
/// which the parser rejects. Rewriting them to the standard form keeps the
/// statement analysable (and lets the mutation rule flag it).
pub fn rewrite_mutations(sql: &str) -> String {
    if !contains_keyword(sql, b"ALTER") {
        return sql.to_owned();
    }
    let step = MUTATION_UPDATE_REGEX.replace_all(sql, "UPDATE ${1} SET ${2} WHERE ${3}${4}");
    MUTATION_DELETE_REGEX
        .replace_all(&step, "DELETE FROM ${1} WHERE ${2}${3}")
        .to_string()
}

/// Preprocess `ClickHouse` SQL.
///
/// Removes unsupported constructs and extracts metadata.
#[must_use]
pub fn preprocess(sql: &str) -> PreprocessorResult {
    // Each clause regex is only compiled and run when its keyword appears:
    // compiling these regexes costs far more than scanning the input.
    let mut metadata = PreprocessorMetadata::default();
    extract_codecs(sql, &mut metadata);
    if contains_keyword(sql, b"TTL") {
        extract_ttl(sql, &mut metadata);
    }
    extract_settings(sql, &mut metadata);
    extract_partition_by(sql, &mut metadata);
    if contains_keyword(sql, b"SAMPLE") {
        extract_sample_by(sql, &mut metadata);
    }
    if contains_keyword(sql, b"INDEX") {
        extract_skip_indexes(sql, &mut metadata);
    }
    extract_projections(sql, &mut metadata);
    if contains_keyword(sql, b"DICTIONARY") {
        extract_dictionaries(sql, &mut metadata);
    }
    if contains_keyword(sql, b"VIEW") {
        extract_materialized_views(sql, &mut metadata);
    }
    let mut result = remove_codecs(sql);
    if contains_keyword(&result, b"TTL") {
        result = remove_ttl(&result);
    }
    result = remove_settings(&result);
    result = remove_partition_by(&result);
    if contains_keyword(&result, b"SAMPLE") {
        result = remove_sample_by(&result);
    }
    if contains_keyword(&result, b"INDEX") {
        result = remove_skip_indexes(&result);
    }
    result = remove_projections(&result);
    if contains_keyword(&result, b"DICTIONARY") {
        result = remove_dictionaries(&result);
    }
    if contains_keyword(&result, b"VIEW") {
        result = remove_materialized_view_engine(&result);
    }
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

/// Extract SAMPLE BY expressions from SQL.
fn extract_sample_by(sql: &str, metadata: &mut PreprocessorMetadata) {
    for cap in SAMPLE_BY_REGEX.captures_iter(sql) {
        if let Some(expr) = cap.get(1) {
            metadata.sample_by.push(expr.as_str().trim().to_string());
        }
    }
}

/// Remove SAMPLE BY clauses from SQL.
fn remove_sample_by(sql: &str) -> String {
    SAMPLE_BY_REGEX.replace_all(sql, "${2}").into_owned()
}

/// Extract data skipping indexes from SQL.
fn extract_skip_indexes(sql: &str, metadata: &mut PreprocessorMetadata) {
    for cap in SKIP_INDEX_REGEX.captures_iter(sql) {
        let name = cap.get(1).map(|m| m.as_str().to_string());
        let expr = cap.get(2).map(|m| m.as_str().trim().to_string());
        let type_name = cap.get(3).map(|m| m.as_str().to_string());
        let type_args = cap.get(4).map(|m| m.as_str().trim().to_string());
        let granularity = cap
            .get(5)
            .and_then(|m| m.as_str().parse::<usize>().ok())
            .unwrap_or(1);
        if let (Some(n), Some(e), Some(t)) = (name, expr, type_name) {
            metadata.skip_indexes.push(SkipIndexInfo {
                name: n,
                expr: e,
                type_name: t,
                type_args: type_args.filter(|s| !s.is_empty()),
                granularity
            });
        }
    }
}

/// Remove data skipping index definitions from SQL.
fn remove_skip_indexes(sql: &str) -> String {
    SKIP_INDEX_REMOVE_REGEX.replace_all(sql, "").into_owned()
}

/// Locate every `PROJECTION` definition and its byte span.
///
/// Supports both `PROJECTION name (SELECT ...)` and `PROJECTION name AS SELECT
/// ...`. Parentheses and commas inside single-quoted strings are ignored when
/// finding the definition boundary.
fn find_projections(sql: &str) -> Vec<(usize, usize, ProjectionInfo)> {
    let bytes = sql.as_bytes();
    let keyword = b"PROJECTION";
    let mut found = Vec::new();
    let mut search = 0;
    while let Some(start) = find_keyword_ignore_case(bytes, keyword, search) {
        let after = start + keyword.len();
        search = after;
        let mut idx = after;
        while idx < bytes.len() && (bytes[idx] as char).is_whitespace() {
            idx += 1;
        }
        let name_start = idx;
        while idx < bytes.len() && (bytes[idx] as char).is_alphanumeric()
            || (idx < bytes.len() && bytes[idx] == b'_')
        {
            idx += 1;
        }
        if name_start == idx {
            continue;
        }
        let name = sql[name_start..idx].to_string();
        while idx < bytes.len() && (bytes[idx] as char).is_whitespace() {
            idx += 1;
        }
        if idx >= bytes.len() {
            continue;
        }
        if bytes[idx] == b'(' {
            if let Some(end) = matching_paren(bytes, idx) {
                let query = sql[idx + 1..end].trim().to_string();
                found.push((
                    start,
                    end + 1,
                    ProjectionInfo {
                        name,
                        query
                    }
                ));
                search = end + 1;
            }
        } else if bytes
            .get(idx..idx + 2)
            .is_some_and(|two| two.eq_ignore_ascii_case(b"AS"))
        {
            let body_start = idx + 2;
            if let Some(end) = projection_as_end(bytes, body_start) {
                let query = sql[body_start..end].trim().to_string();
                found.push((
                    start,
                    end,
                    ProjectionInfo {
                        name,
                        query
                    }
                ));
                search = end;
            }
        }
    }
    found
}

/// Return the byte index of the `)` matching the `(` at `open`.
const fn matching_paren(bytes: &[u8], open: usize) -> Option<usize> {
    let mut depth = 0;
    let mut in_str = false;
    let mut i = open;
    while i < bytes.len() {
        let c = bytes[i];
        if in_str {
            if c == b'\'' {
                in_str = false;
            }
        } else {
            match c {
                b'\'' => in_str = true,
                b'(' => depth += 1,
                b')' => {
                    depth -= 1;
                    if depth == 0 {
                        return Some(i);
                    }
                }
                _ => {}
            }
        }
        i += 1;
    }
    None
}

/// Find the end of a `PROJECTION name AS SELECT ...` body: the first top-level
/// comma or closing parenthesis outside of strings and nested parentheses.
const fn projection_as_end(bytes: &[u8], start: usize) -> Option<usize> {
    let mut depth = 0;
    let mut in_str = false;
    let mut i = start;
    while i < bytes.len() {
        let c = bytes[i];
        if in_str {
            if c == b'\'' {
                in_str = false;
            }
        } else {
            match c {
                b'\'' => in_str = true,
                b'(' => depth += 1,
                b')' | b',' if depth == 0 => return Some(i),
                b')' => depth -= 1,
                _ => {}
            }
        }
        i += 1;
    }
    None
}

/// Extract projection definitions into metadata.
fn extract_projections(sql: &str, metadata: &mut PreprocessorMetadata) {
    for (_, _, projection) in find_projections(sql) {
        metadata.projections.push(projection);
    }
}

/// Remove projection definitions (and a leading comma) from SQL.
fn remove_projections(sql: &str) -> String {
    let mut spans: Vec<(usize, usize)> = find_projections(sql)
        .into_iter()
        .map(|(start, end, _)| (start, end))
        .collect();
    spans.sort_by_key(|(start, _)| *start);
    let mut out = String::with_capacity(sql.len());
    let mut cursor = 0;
    for (start, end) in spans {
        let mut cut_start = start;
        let prefix = sql[cursor..start].trim_end();
        if prefix.ends_with(',') {
            cut_start = sql[..start].trim_end().len() - 1;
        }
        out.push_str(&sql[cursor..cut_start]);
        cursor = end;
    }
    out.push_str(&sql[cursor..]);
    out
}

/// Extract `CREATE DICTIONARY` names into metadata.
fn extract_dictionaries(sql: &str, metadata: &mut PreprocessorMetadata) {
    for cap in DICTIONARY_REGEX.captures_iter(sql) {
        if let Some(name) = cap.get(1) {
            metadata.dictionaries.push(name.as_str().to_string());
        }
    }
}

/// Remove whole `CREATE DICTIONARY` statements from SQL.
fn remove_dictionaries(sql: &str) -> String {
    DICTIONARY_REMOVE_REGEX.replace_all(sql, "").into_owned()
}

/// Extract `CREATE MATERIALIZED VIEW` metadata into metadata.
fn extract_materialized_views(sql: &str, metadata: &mut PreprocessorMetadata) {
    for cap in MATERIALIZED_VIEW_REGEX.captures_iter(sql) {
        let whole = cap.get(0).map_or(0, |m| m.end());
        let name = cap.get(1).map_or_default(|m| m.as_str().to_string());
        let target = cap.get(2).map(|m| m.as_str().to_string());
        let query = sql[whole..]
            .split(';')
            .next()
            .and_then(|seg| {
                find_keyword_ignore_case(seg.as_bytes(), b"SELECT", 0)
                    .map(|i| seg[i..].trim().to_string())
            })
            .unwrap_or_default();
        metadata.materialized_views.push(MaterializedViewInfo {
            name,
            target,
            query
        });
    }
}

/// Strip the engine clause between a materialized view header and its body so
/// the statement parses with `sqlparser`.
fn remove_materialized_view_engine(sql: &str) -> String {
    MATERIALIZED_VIEW_ENGINE_REGEX
        .replace_all(sql, |caps: &regex::Captures| {
            format!("{} {}", &caps[1], &caps[3])
        })
        .to_string()
}

/// Remove CODEC clauses from SQL.
fn remove_codecs(sql: &str) -> String {
    CODEC_REGEX.replace_all(sql, "").into_owned()
}

/// Remove TTL clauses from SQL.
fn remove_ttl(sql: &str) -> String {
    TTL_REGEX.replace_all(sql, "${2}").into_owned()
}

/// Remove SETTINGS clauses from SQL.
fn remove_settings(sql: &str) -> String {
    SETTINGS_REGEX.replace_all(sql, "").into_owned()
}

/// Remove PARTITION BY clauses from SQL.
fn remove_partition_by(sql: &str) -> String {
    PARTITION_BY_REGEX.replace_all(sql, "").into_owned()
}

/// Regex for collapsing whitespace runs.
static WHITESPACE_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\s+").expect("valid regex"));

/// Normalize excessive whitespace.
fn normalize_whitespace(sql: &str) -> String {
    WHITESPACE_REGEX.replace_all(sql.trim(), " ").into_owned()
}

/// Find an ASCII case-insensitive `needle` in `haystack` at or after `from`.
fn find_keyword_ignore_case(haystack: &[u8], needle: &[u8], from: usize) -> Option<usize> {
    if needle.is_empty() || haystack.len() < from + needle.len() {
        return None;
    }
    (from..=haystack.len() - needle.len()).find(|&i| {
        haystack[i..i + needle.len()]
            .iter()
            .zip(needle)
            .all(|(h, n)| h.eq_ignore_ascii_case(n))
    })
}

/// Whether `keyword` occurs in `sql` as a whole ASCII word, ignoring case.
///
/// Used as a cheap pre-check before running a clause regex. A non-ASCII
/// neighbour counts as a word boundary, so this never rejects input that the
/// matching `\bKEYWORD\s` regex would accept.
fn contains_keyword(sql: &str, keyword: &[u8]) -> bool {
    let bytes = sql.as_bytes();
    let is_word = |b: u8| b.is_ascii_alphanumeric() || b == b'_';
    let mut from = 0;
    while let Some(start) = find_keyword_ignore_case(bytes, keyword, from) {
        let end = start + keyword.len();
        let before_ok = start == 0 || !is_word(bytes[start - 1]);
        let after_ok = end == bytes.len() || !is_word(bytes[end]);
        if before_ok && after_ok {
            return true;
        }
        from = start + 1;
    }
    false
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
        let sql = r"
            CREATE TABLE t (
                col1 String CODEC(ZSTD),
                col2 UInt64 CODEC(Delta, LZ4)
            ) ENGINE = MergeTree ORDER BY col1
        ";
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
        let sql = r"
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
        ";
        let result = preprocess(sql);
        assert!(!result.sql.contains("CODEC"));
        assert!(!result.sql.contains("TTL"));
        assert!(!result.sql.contains("SETTINGS"));
        assert!(!result.sql.contains("index_granularity"));
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
    fn test_ttl_followed_by_settings_keeps_no_body() {
        let sql = "CREATE TABLE t (d Date) ENGINE = MergeTree ORDER BY d \
                   TTL d + INTERVAL 7 DAY SETTINGS index_granularity = 4096";
        let result = preprocess(sql);
        assert!(!result.sql.contains("index_granularity"));
        assert!(!result.sql.contains("SETTINGS"));
        assert_eq!(
            result.metadata.settings.get("index_granularity"),
            Some(&"4096".to_string())
        );
    }

    #[test]
    fn test_sample_by_followed_by_settings() {
        let sql = "CREATE TABLE t (id UInt64) ENGINE = MergeTree ORDER BY id \
                   SAMPLE BY id SETTINGS index_granularity = 4096";
        let result = preprocess(sql);
        assert!(!result.sql.contains("SAMPLE"));
        assert!(!result.sql.contains("index_granularity"));
        assert_eq!(result.metadata.sample_by.len(), 1);
    }

    #[test]
    fn test_projection_extraction() {
        let sql = "CREATE TABLE t (a UInt64, b String, PROJECTION p (SELECT a, sum(b) ORDER BY a)) \
                   ENGINE = MergeTree ORDER BY a";
        let result = preprocess(sql);
        assert!(!result.sql.contains("PROJECTION"));
        assert_eq!(result.metadata.projections.len(), 1);
        assert_eq!(result.metadata.projections[0].name, "p");
        assert!(result.metadata.projections[0].query.contains("sum(b)"));
    }

    #[test]
    fn test_projection_as_select_form() {
        let sql = "CREATE TABLE t (a UInt64, PROJECTION p AS SELECT a ORDER BY a) \
                   ENGINE = MergeTree ORDER BY a";
        let result = preprocess(sql);
        assert!(!result.sql.contains("PROJECTION"));
        assert_eq!(result.metadata.projections.len(), 1);
    }

    #[test]
    fn test_dictionary_removal() {
        let sql = "CREATE DICTIONARY d (id UInt64) PRIMARY KEY id SOURCE(NULL()) LAYOUT(FLAT()); \
                   CREATE TABLE x (id UInt64) ENGINE = MergeTree ORDER BY id";
        let result = preprocess(sql);
        assert!(!result.sql.contains("DICTIONARY"));
        assert_eq!(result.metadata.dictionaries, vec!["d".to_string()]);
        assert!(result.sql.contains("CREATE TABLE x"));
    }

    #[test]
    fn test_materialized_view_engine_strip() {
        let sql = "CREATE MATERIALIZED VIEW mv TO dest ENGINE = MergeTree ORDER BY a \
                   AS SELECT a FROM src";
        let result = preprocess(sql);
        assert!(!result.sql.contains("ENGINE"));
        assert!(result.sql.contains("AS SELECT a FROM src"));
        assert_eq!(result.metadata.materialized_views.len(), 1);
        let mv = &result.metadata.materialized_views[0];
        assert_eq!(mv.name, "mv");
        assert_eq!(mv.target.as_deref(), Some("dest"));
        assert!(mv.query.contains("SELECT a FROM src"));
    }

    #[test]
    fn test_rewrite_mutations() {
        let upd = rewrite_mutations("ALTER TABLE t UPDATE x = 0 WHERE d = today()");
        assert_eq!(upd, "UPDATE t SET x = 0 WHERE d = today()");
        let del = rewrite_mutations("ALTER TABLE t DELETE WHERE d = today();");
        assert_eq!(del, "DELETE FROM t WHERE d = today();");
    }

    #[test]
    fn test_contains_keyword_requires_word_boundaries() {
        assert!(contains_keyword("a INDEX b", b"INDEX"));
        assert!(contains_keyword("index", b"INDEX"));
        assert!(contains_keyword("x,Index(", b"INDEX"));
        assert!(!contains_keyword(
            "SETTINGS index_granularity = 8192",
            b"INDEX"
        ));
        assert!(!contains_keyword("reindex", b"INDEX"));
        assert!(!contains_keyword("ttl_only_drop_parts = 1", b"TTL"));
        assert!(!contains_keyword("", b"TTL"));
    }

    #[test]
    fn test_no_modification_without_special_syntax() {
        let sql = "CREATE TABLE t (id UInt64) ENGINE = MergeTree ORDER BY id";
        let result = preprocess(sql);
        assert_eq!(result.sql, sql);
        assert!(result.metadata.codecs.is_empty());
        assert_eq!(
            result.metadata.ttl_expressions,
            [] as [std::string::String; 0]
        );
        assert!(result.metadata.settings.is_empty());
    }
}
