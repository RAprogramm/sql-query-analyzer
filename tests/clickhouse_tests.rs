// SPDX-FileCopyrightText: 2025 RAprogramm
// SPDX-License-Identifier: MIT

use sql_query_analyzer::{
    config::RulesConfig,
    query::{SqlDialect, parse_queries},
    rules::RuleRunner,
    schema::Schema
};

fn ch_ids(sql: &str, schema_sql: &str) -> Vec<String> {
    let schema = Schema::parse(schema_sql, SqlDialect::ClickHouse).unwrap();
    let queries = parse_queries(sql, SqlDialect::ClickHouse).unwrap();
    let runner = RuleRunner::with_schema_and_config_and_dialect(
        SqlDialect::ClickHouse,
        schema,
        RulesConfig::default()
    );
    runner
        .analyze(&queries)
        .violations
        .iter()
        .map(|v| v.rule_id.to_string())
        .collect()
}

const MERGE_TREE: &str = "CREATE TABLE t (d Date, x UInt64) ENGINE = MergeTree ORDER BY d";

#[test]
fn ch001_flags_select_star() {
    let v = ch_ids("SELECT * FROM t", MERGE_TREE);
    assert!(v.contains(&"CH001".to_string()));
}

#[test]
fn ch001_ignores_explicit_columns() {
    let v = ch_ids("SELECT d, x FROM t", MERGE_TREE);
    assert!(!v.contains(&"CH001".to_string()));
}

#[test]
fn ch002_flags_filter_off_sort_key() {
    let v = ch_ids("SELECT d FROM t WHERE x = 1", MERGE_TREE);
    assert!(v.contains(&"CH002".to_string()));
}

#[test]
fn ch002_ok_when_filter_uses_sort_key() {
    let v = ch_ids("SELECT x FROM t WHERE d = today()", MERGE_TREE);
    assert!(!v.contains(&"CH002".to_string()));
}

#[test]
fn ch003_flags_missing_partition_pruning() {
    let schema =
        "CREATE TABLE t (d Date, x UInt64) ENGINE = MergeTree ORDER BY d PARTITION BY toYYYYMM(d)";
    let v = ch_ids("SELECT x FROM t WHERE x = 1", schema);
    assert!(v.contains(&"CH003".to_string()));
}

#[test]
fn ch003_ok_when_partition_pruned() {
    let schema =
        "CREATE TABLE t (d Date, x UInt64) ENGINE = MergeTree ORDER BY d PARTITION BY toYYYYMM(d)";
    let v = ch_ids("SELECT x FROM t WHERE d = today()", schema);
    assert!(!v.contains(&"CH003".to_string()));
}

#[test]
fn ch004_flags_function_on_date_column() {
    let v = ch_ids("SELECT x FROM t WHERE toDate(d) = today()", MERGE_TREE);
    assert!(v.contains(&"CH004".to_string()));
}

#[test]
fn ch004_ok_for_range_predicate() {
    let v = ch_ids(
        "SELECT x FROM t WHERE d >= today() AND d < today() + 1",
        MERGE_TREE
    );
    assert!(!v.contains(&"CH004".to_string()));
}

#[test]
fn ch005_flags_uncovered_filter_column() {
    let v = ch_ids("SELECT d FROM t WHERE x = 1", MERGE_TREE);
    assert!(v.contains(&"CH005".to_string()));
}

#[test]
fn ch005_ok_when_skip_index_present() {
    let schema = "CREATE TABLE t (d Date, x UInt64, INDEX idx_x x TYPE minmax GRANULARITY 4) ENGINE = MergeTree ORDER BY d";
    let v = ch_ids("SELECT d FROM t WHERE x = 1", schema);
    assert!(!v.contains(&"CH005".to_string()));
}

#[test]
fn ch006_flags_lowcardinality_high_cardinality() {
    let schema = "CREATE TABLE t (id LowCardinality(String)) ENGINE = MergeTree ORDER BY id";
    let v = ch_ids("SELECT id FROM t", schema);
    assert!(v.contains(&"CH006".to_string()));
}

#[test]
fn ch007_flags_nullable_in_filter() {
    let schema = "CREATE TABLE t (d Date, s Nullable(String)) ENGINE = MergeTree ORDER BY d";
    let v = ch_ids("SELECT d FROM t WHERE s = 'x'", schema);
    assert!(v.contains(&"CH007".to_string()));
}

#[test]
fn ch009_flags_uniqexact() {
    let v = ch_ids("SELECT uniqExact(x) FROM t", MERGE_TREE);
    assert!(v.contains(&"CH009".to_string()));
}

#[test]
fn ch009_ok_for_approximate_uniq() {
    let v = ch_ids("SELECT uniq(x) FROM t", MERGE_TREE);
    assert!(!v.contains(&"CH009".to_string()));
}

#[test]
fn ch010_flags_multiple_count_subqueries() {
    let sql = "SELECT (SELECT count() FROM t WHERE x = 1), (SELECT count() FROM t WHERE x = 2)";
    let v = ch_ids(sql, MERGE_TREE);
    assert!(v.contains(&"CH010".to_string()));
}

#[test]
fn ch011_flags_order_by_in_subquery_without_limit() {
    let sql = "SELECT * FROM (SELECT d FROM t ORDER BY d)";
    let v = ch_ids(sql, MERGE_TREE);
    assert!(v.contains(&"CH011".to_string()));
}

#[test]
fn ch012_flags_group_by_mismatch() {
    let v = ch_ids("SELECT x, count() FROM t GROUP BY x", MERGE_TREE);
    assert!(v.contains(&"CH012".to_string()));
}

#[test]
fn ch012_ok_when_group_by_matches_key() {
    let v = ch_ids("SELECT d, count() FROM t GROUP BY d", MERGE_TREE);
    assert!(!v.contains(&"CH012".to_string()));
}

#[test]
fn ch016_flags_multi_table_join() {
    let schema = "CREATE TABLE a (id UInt64) ENGINE = MergeTree ORDER BY id; \
                  CREATE TABLE b (id UInt64) ENGINE = MergeTree ORDER BY id";
    let v = ch_ids("SELECT a.id FROM a JOIN b ON a.id = b.id", schema);
    assert!(v.contains(&"CH016".to_string()));
}

#[test]
fn ch019_flags_final() {
    let v = ch_ids("SELECT d FROM t FINAL", MERGE_TREE);
    assert!(v.contains(&"CH019".to_string()));
}

#[test]
fn ch019_ok_without_final() {
    let v = ch_ids("SELECT d FROM t", MERGE_TREE);
    assert!(!v.contains(&"CH019".to_string()));
}

#[test]
fn generic_dialect_does_not_run_clickhouse_rules() {
    let schema = Schema::parse(MERGE_TREE, SqlDialect::Generic).unwrap();
    let queries = parse_queries("SELECT * FROM t", SqlDialect::Generic).unwrap();
    let runner = RuleRunner::with_schema_and_config_and_dialect(
        SqlDialect::Generic,
        schema,
        RulesConfig::default()
    );
    let ids: Vec<String> = runner
        .analyze(&queries)
        .violations
        .iter()
        .map(|v| v.rule_id.to_string())
        .collect();
    assert!(!ids.iter().any(|id| id.starts_with("CH")));
}
