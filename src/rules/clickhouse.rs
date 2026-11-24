//! ClickHouse-specific analysis rules.
//!
//! These rules encode ClickHouse best practices: sparse primary index usage,
//! partition pruning, data skipping indexes, columnar-storage anti-patterns,
//! and OLTP-style misuse. They are registered only when the ClickHouse
//! dialect is selected.

use super::{Rule, RuleCategory, RuleInfo, Severity, Violation};
use crate::{
    query::{Query, QueryType},
    schema::{Schema, TableInfo}
};

/// Returns the table in the schema whose name matches a query table reference,
/// ignoring an optional database prefix and case.
fn find_table<'a>(schema: &'a Schema, name: &str) -> Option<&'a TableInfo> {
    let bare = name.rsplit('.').next().unwrap_or(name);
    schema
        .tables
        .values()
        .find(|t| t.name.eq_ignore_ascii_case(bare))
}

/// Extracts bare identifiers from a ClickHouse expression such as
/// `toYYYYMM(event_date)` or `(event_date, user_id)`.
fn identifiers_in(expr: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut current = String::new();
    for ch in expr.chars() {
        if ch.is_ascii_alphanumeric() || ch == '_' {
            current.push(ch);
        } else if !current.is_empty() {
            out.push(current.clone());
            current.clear();
        }
    }
    if !current.is_empty() {
        out.push(current);
    }
    out
}

/// Returns the effective sort key columns for a table (PRIMARY KEY or ORDER
/// BY).
fn sort_key(table: &TableInfo) -> Vec<String> {
    table
        .primary_key
        .clone()
        .or_else(|| table.order_by.clone())
        .unwrap_or_default()
}

/// SELECT * on a columnar store reads every column file.
pub struct SelectStarWideTable;

impl Rule for SelectStarWideTable {
    fn info(&self) -> RuleInfo {
        RuleInfo {
            id:       "CH001",
            name:     "SELECT * on columnar store",
            severity: Severity::Info,
            category: RuleCategory::Performance
        }
    }

    fn check(&self, query: &Query, query_index: usize) -> Vec<Violation> {
        if query.query_type != QueryType::Select {
            return vec![];
        }
        let upper = query.raw.to_uppercase();
        if !(upper.contains("SELECT *") || upper.contains("SELECT  *")) {
            return vec![];
        }
        let info = self.info();
        vec![Violation {
            rule_id: info.id,
            rule_name: info.name,
            message: "SELECT * reads every column file in a columnar store".to_string(),
            severity: info.severity,
            category: info.category,
            suggestion: Some(
                "List only the columns you need; ClickHouse reads and decompresses each selected column"
                    .to_string()
            ),
            query_index
        }]
    }
}

/// WHERE clause does not touch the primary key, so no granules are pruned.
pub struct FilterNotUsingPrimaryKey {
    pub schema: Schema
}

impl Rule for FilterNotUsingPrimaryKey {
    fn info(&self) -> RuleInfo {
        RuleInfo {
            id:       "CH002",
            name:     "Filter does not use primary key",
            severity: Severity::Warning,
            category: RuleCategory::Performance
        }
    }

    fn check(&self, query: &Query, query_index: usize) -> Vec<Violation> {
        if query.query_type != QueryType::Select || query.where_cols.is_empty() {
            return vec![];
        }
        let mut violations = Vec::new();
        for table_name in &query.tables {
            let Some(table) = find_table(&self.schema, table_name) else {
                continue;
            };
            let key = sort_key(table);
            if key.is_empty() {
                continue;
            }
            let uses_key = query
                .where_cols
                .iter()
                .any(|c| key.iter().any(|k| k.eq_ignore_ascii_case(c)));
            if !uses_key {
                let info = self.info();
                violations.push(Violation {
                    rule_id: info.id,
                    rule_name: info.name,
                    message: format!(
                        "WHERE on '{}' does not reference the sort key ({})",
                        table.name,
                        key.join(", ")
                    ),
                    severity: info.severity,
                    category: info.category,
                    suggestion: Some(
                        "Filter on a prefix of the ORDER BY key, or add a data skipping index"
                            .to_string()
                    ),
                    query_index
                });
            }
        }
        violations
    }
}

/// WHERE does not match the partition expression, so partitions are not pruned.
pub struct PartitionPruningMissed {
    pub schema: Schema
}

impl Rule for PartitionPruningMissed {
    fn info(&self) -> RuleInfo {
        RuleInfo {
            id:       "CH003",
            name:     "Partition pruning missed",
            severity: Severity::Warning,
            category: RuleCategory::Performance
        }
    }

    fn check(&self, query: &Query, query_index: usize) -> Vec<Violation> {
        if query.query_type != QueryType::Select || query.where_cols.is_empty() {
            return vec![];
        }
        let mut violations = Vec::new();
        for table_name in &query.tables {
            let Some(table) = find_table(&self.schema, table_name) else {
                continue;
            };
            let Some(partition) = &table.partition_by else {
                continue;
            };
            let part_cols = identifiers_in(partition);
            let prunes = query
                .where_cols
                .iter()
                .any(|c| part_cols.iter().any(|p| p.eq_ignore_ascii_case(c)));
            if !prunes {
                let info = self.info();
                violations.push(Violation {
                    rule_id: info.id,
                    rule_name: info.name,
                    message: format!(
                        "WHERE does not match PARTITION BY ({}) on '{}'",
                        partition, table.name
                    ),
                    severity: info.severity,
                    category: info.category,
                    suggestion: Some(
                        "Filter on the partition expression (e.g. toYYYYMM(date) = 202401) to skip whole partitions"
                            .to_string()
                    ),
                    query_index
                });
            }
        }
        violations
    }
}

/// Function call on a date/time column in WHERE prevents index usage.
pub struct FunctionOnKeyColumn;

const CH_DATE_FUNCTIONS: [&str; 11] = [
    "TODATE(",
    "TODATETIME(",
    "TOYEAR(",
    "TOMONTH(",
    "TOSTARTOFDAY(",
    "TOSTARTOFMONTH(",
    "TOSTARTOFWEEK(",
    "TOSTARTOFHOUR(",
    "TODAYOFWEEK(",
    "TODAYOFYEAR(",
    "TOSTARTOFQUARTER("
];

impl Rule for FunctionOnKeyColumn {
    fn info(&self) -> RuleInfo {
        RuleInfo {
            id:       "CH004",
            name:     "Function on date column in WHERE",
            severity: Severity::Warning,
            category: RuleCategory::Performance
        }
    }

    fn check(&self, query: &Query, query_index: usize) -> Vec<Violation> {
        if query.query_type != QueryType::Select {
            return vec![];
        }
        let upper = query.raw.to_uppercase();
        let Some(pos) = upper.find("WHERE") else {
            return vec![];
        };
        let where_part = &upper[pos..];
        if !CH_DATE_FUNCTIONS.iter().any(|f| where_part.contains(f)) {
            return vec![];
        }
        let info = self.info();
        vec![Violation {
            rule_id: info.id,
            rule_name: info.name,
            message: "Wrapping a date column in a function disables the primary index".to_string(),
            severity: info.severity,
            category: info.category,
            suggestion: Some(
                "Use a range predicate (col >= '...' AND col < '...') so the sparse index can prune granules"
                    .to_string()
            ),
            query_index
        }]
    }
}

/// WHERE column is neither in the sort key nor covered by a skipping index.
pub struct MissingSkipIndex {
    pub schema: Schema
}

impl Rule for MissingSkipIndex {
    fn info(&self) -> RuleInfo {
        RuleInfo {
            id:       "CH005",
            name:     "No skipping index for filter column",
            severity: Severity::Info,
            category: RuleCategory::Performance
        }
    }

    fn check(&self, query: &Query, query_index: usize) -> Vec<Violation> {
        if query.query_type != QueryType::Select || query.where_cols.is_empty() {
            return vec![];
        }
        let mut violations = Vec::new();
        for table_name in &query.tables {
            let Some(table) = find_table(&self.schema, table_name) else {
                continue;
            };
            let key = sort_key(table);
            for col in &query.where_cols {
                let in_key = key.iter().any(|k| k.eq_ignore_ascii_case(col));
                let in_skip = table.skip_indexes.iter().any(|si| {
                    identifiers_in(&si.expr)
                        .iter()
                        .any(|e| e.eq_ignore_ascii_case(col))
                });
                if !in_key && !in_skip {
                    let info = self.info();
                    violations.push(Violation {
                        rule_id: info.id,
                        rule_name: info.name,
                        message: format!(
                            "Column '{}' on '{}' is not in the sort key or a skipping index",
                            col, table.name
                        ),
                        severity: info.severity,
                        category: info.category,
                        suggestion: Some(format!(
                            "ADD INDEX idx_{col} {col} TYPE minmax GRANULARITY 4 (or bloom_filter for high-cardinality)",
                            col = col
                        )),
                        query_index
                    });
                }
            }
        }
        violations
    }
}

/// LowCardinality applied to a likely high-cardinality column.
pub struct LowCardinalityHighCardinality {
    pub schema: Schema
}

const HIGH_CARDINALITY_HINTS: [&str; 6] = ["ID", "UUID", "URL", "HASH", "TOKEN", "TRACE"];

impl Rule for LowCardinalityHighCardinality {
    fn info(&self) -> RuleInfo {
        RuleInfo {
            id:       "CH006",
            name:     "LowCardinality on high-cardinality column",
            severity: Severity::Info,
            category: RuleCategory::Performance
        }
    }

    fn check(&self, query: &Query, query_index: usize) -> Vec<Violation> {
        let mut violations = Vec::new();
        for table_name in &query.tables {
            let Some(table) = find_table(&self.schema, table_name) else {
                continue;
            };
            for col in &table.columns {
                let ty = col.data_type.to_uppercase();
                if !ty.contains("LOWCARDINALITY") {
                    continue;
                }
                let upper_name = col.name.to_uppercase();
                if HIGH_CARDINALITY_HINTS
                    .iter()
                    .any(|h| upper_name.contains(h))
                {
                    let info = self.info();
                    violations.push(Violation {
                        rule_id: info.id,
                        rule_name: info.name,
                        message: format!(
                            "Column '{}' on '{}' uses LowCardinality but looks high-cardinality",
                            col.name, table.name
                        ),
                        severity: info.severity,
                        category: info.category,
                        suggestion: Some(
                            "LowCardinality helps below ~10k distinct values; verify with uniqExact before keeping it"
                                .to_string()
                        ),
                        query_index
                    });
                }
            }
        }
        violations
    }
}

/// Nullable column used in a hot filter path.
pub struct NullableHotPath {
    pub schema: Schema
}

impl Rule for NullableHotPath {
    fn info(&self) -> RuleInfo {
        RuleInfo {
            id:       "CH007",
            name:     "Nullable column in hot path",
            severity: Severity::Info,
            category: RuleCategory::Performance
        }
    }

    fn check(&self, query: &Query, query_index: usize) -> Vec<Violation> {
        if query.where_cols.is_empty() && query.join_cols.is_empty() {
            return vec![];
        }
        let mut violations = Vec::new();
        for table_name in &query.tables {
            let Some(table) = find_table(&self.schema, table_name) else {
                continue;
            };
            for col in &table.columns {
                if !col.is_nullable {
                    continue;
                }
                let used = query
                    .where_cols
                    .iter()
                    .chain(query.join_cols.iter())
                    .any(|c| c.eq_ignore_ascii_case(&col.name));
                if used {
                    let info = self.info();
                    violations.push(Violation {
                        rule_id: info.id,
                        rule_name: info.name,
                        message: format!(
                            "Nullable column '{}' on '{}' is used in a filter",
                            col.name, table.name
                        ),
                        severity: info.severity,
                        category: info.category,
                        suggestion: Some(
                            "Nullable adds a null-mask read and blocks optimizations; prefer a sentinel default"
                                .to_string()
                        ),
                        query_index
                    });
                }
            }
        }
        violations
    }
}

/// uniqExact used where approximate uniq is usually acceptable.
pub struct UniqExactUsage;

impl Rule for UniqExactUsage {
    fn info(&self) -> RuleInfo {
        RuleInfo {
            id:       "CH009",
            name:     "uniqExact usage",
            severity: Severity::Info,
            category: RuleCategory::Performance
        }
    }

    fn check(&self, query: &Query, query_index: usize) -> Vec<Violation> {
        if !query.raw.to_uppercase().contains("UNIQEXACT(") {
            return vec![];
        }
        let info = self.info();
        vec![Violation {
            rule_id: info.id,
            rule_name: info.name,
            message: "uniqExact tracks every distinct value and is memory-heavy".to_string(),
            severity: info.severity,
            category: info.category,
            suggestion: Some(
                "Use uniq() or uniqCombined() for approximate distinct counts in analytics"
                    .to_string()
            ),
            query_index
        }]
    }
}

/// Several count subqueries that a single countIf scan can replace.
pub struct CountIfOpportunity;

impl Rule for CountIfOpportunity {
    fn info(&self) -> RuleInfo {
        RuleInfo {
            id:       "CH010",
            name:     "Conditional counts via subqueries",
            severity: Severity::Info,
            category: RuleCategory::Performance
        }
    }

    fn check(&self, query: &Query, query_index: usize) -> Vec<Violation> {
        let upper = query.raw.to_uppercase();
        let count_subqueries = upper.matches("(SELECT COUNT(").count();
        if count_subqueries < 2 {
            return vec![];
        }
        let info = self.info();
        vec![Violation {
            rule_id: info.id,
            rule_name: info.name,
            message: format!(
                "{} count subqueries scan the table separately",
                count_subqueries
            ),
            severity: info.severity,
            category: info.category,
            suggestion: Some(
                "Use countIf(condition) columns in a single scan instead of separate subqueries"
                    .to_string()
            ),
            query_index
        }]
    }
}

/// ORDER BY inside a subquery without LIMIT wastes CPU.
pub struct OrderByInSubquery;

impl Rule for OrderByInSubquery {
    fn info(&self) -> RuleInfo {
        RuleInfo {
            id:       "CH011",
            name:     "ORDER BY in subquery without LIMIT",
            severity: Severity::Warning,
            category: RuleCategory::Performance
        }
    }

    fn check(&self, query: &Query, query_index: usize) -> Vec<Violation> {
        if query.query_type != QueryType::Select || !query.has_subquery {
            return vec![];
        }
        let upper = query.raw.to_uppercase();
        if upper.contains("ORDER BY") && query.limit.is_none() {
            let info = self.info();
            return vec![Violation {
                rule_id: info.id,
                rule_name: info.name,
                message: "Sorting a subquery result is wasted work when the outer query re-sorts"
                    .to_string(),
                severity: info.severity,
                category: info.category,
                suggestion: Some(
                    "Remove ORDER BY from the subquery unless it is paired with LIMIT".to_string()
                ),
                query_index
            }];
        }
        vec![]
    }
}

/// GROUP BY order does not match the sort key, blocking stream aggregation.
pub struct GroupByOrderMismatch {
    pub schema: Schema
}

impl Rule for GroupByOrderMismatch {
    fn info(&self) -> RuleInfo {
        RuleInfo {
            id:       "CH012",
            name:     "GROUP BY does not match sort key",
            severity: Severity::Info,
            category: RuleCategory::Performance
        }
    }

    fn check(&self, query: &Query, query_index: usize) -> Vec<Violation> {
        if query.query_type != QueryType::Select || query.group_cols.is_empty() {
            return vec![];
        }
        let mut violations = Vec::new();
        for table_name in &query.tables {
            let Some(table) = find_table(&self.schema, table_name) else {
                continue;
            };
            let key = sort_key(table);
            if key.is_empty() {
                continue;
            }
            let first_group = query.group_cols.first().map(|c| c.to_lowercase());
            let matches = first_group
                .as_ref()
                .is_some_and(|g| key.first().is_some_and(|k| k.eq_ignore_ascii_case(g)));
            if !matches {
                let info = self.info();
                violations.push(Violation {
                    rule_id: info.id,
                    rule_name: info.name,
                    message: format!(
                        "GROUP BY prefix differs from the sort key of '{}'",
                        table.name
                    ),
                    severity: info.severity,
                    category: info.category,
                    suggestion: Some(
                        "Align GROUP BY with the ORDER BY prefix to enable streaming aggregation without a hash table"
                            .to_string()
                    ),
                    query_index
                });
            }
        }
        violations
    }
}

/// Joining multiple tables where a dictionary or denormalization is cheaper.
pub struct JoinBigTables;

impl Rule for JoinBigTables {
    fn info(&self) -> RuleInfo {
        RuleInfo {
            id:       "CH016",
            name:     "Multi-table JOIN",
            severity: Severity::Info,
            category: RuleCategory::Performance
        }
    }

    fn check(&self, query: &Query, query_index: usize) -> Vec<Violation> {
        if query.query_type != QueryType::Select || query.join_cols.is_empty() {
            return vec![];
        }
        if query.tables.len() < 2 {
            return vec![];
        }
        let info = self.info();
        vec![Violation {
            rule_id: info.id,
            rule_name: info.name,
            message: format!(
                "Query joins {} tables; ClickHouse builds an in-memory hash table per join",
                query.tables.len()
            ),
            severity: info.severity,
            category: info.category,
            suggestion: Some(
                "Keep the large table on the left, or replace small dimension joins with a dictionary or denormalized columns"
                    .to_string()
            ),
            query_index
        }]
    }
}

/// FINAL forces read-time deduplication.
pub struct FinalUsage;

impl Rule for FinalUsage {
    fn info(&self) -> RuleInfo {
        RuleInfo {
            id:       "CH019",
            name:     "FINAL modifier usage",
            severity: Severity::Warning,
            category: RuleCategory::Performance
        }
    }

    fn check(&self, query: &Query, query_index: usize) -> Vec<Violation> {
        if !query.has_final {
            return vec![];
        }
        let info = self.info();
        vec![Violation {
            rule_id: info.id,
            rule_name: info.name,
            message: "FINAL merges parts at read time, which is expensive on large tables"
                .to_string(),
            severity: info.severity,
            category: info.category,
            suggestion: Some(
                "Prefer argMax/argMin or ReplacingMergeTree with eventual merges over FINAL"
                    .to_string()
            ),
            query_index
        }]
    }
}
