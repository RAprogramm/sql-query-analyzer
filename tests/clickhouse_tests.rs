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
        &RulesConfig::default()
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
        &RulesConfig::default()
    );
    let ids: Vec<String> = runner
        .analyze(&queries)
        .violations
        .iter()
        .map(|v| v.rule_id.to_string())
        .collect();
    assert!(!ids.iter().any(|id| id.starts_with("CH")));
}

#[test]
fn ch021_flags_where_without_prewhere() {
    let v = ch_ids("SELECT x FROM t WHERE x = 1", MERGE_TREE);
    assert!(v.contains(&"CH021".to_string()));
}

#[test]
fn ch021_ok_when_prewhere_present() {
    let v = ch_ids("SELECT x FROM t PREWHERE x = 1", MERGE_TREE);
    assert!(!v.contains(&"CH021".to_string()));
}

#[test]
fn ch022_flags_alter_update() {
    let v = ch_ids("ALTER TABLE t UPDATE x = 0 WHERE d = today()", MERGE_TREE);
    assert!(v.contains(&"CH022".to_string()));
}

#[test]
fn ch022_ignores_alter_add_column() {
    let v = ch_ids("ALTER TABLE t ADD COLUMN y UInt64", MERGE_TREE);
    assert!(!v.contains(&"CH022".to_string()));
}

#[test]
fn ch023_flags_optimize_final() {
    let v = ch_ids("OPTIMIZE TABLE t FINAL", MERGE_TREE);
    assert!(v.contains(&"CH023".to_string()));
}

#[test]
fn ch024_flags_join_with_dictionary() {
    let schema = "CREATE DICTIONARY dict (id UInt64) PRIMARY KEY id SOURCE(NULL()) LAYOUT(FLAT()); \
                  CREATE TABLE t (id UInt64) ENGINE = MergeTree ORDER BY id; \
                  CREATE TABLE dict (id UInt64) ENGINE = MergeTree ORDER BY id";
    let v = ch_ids("SELECT t.id FROM t JOIN dict ON t.id = dict.id", schema);
    assert!(v.contains(&"CH024".to_string()));
}

#[test]
fn ch025_flags_group_by_plain_string() {
    let schema = "CREATE TABLE t (d Date, s String) ENGINE = MergeTree ORDER BY d";
    let v = ch_ids("SELECT s, count() FROM t GROUP BY s", schema);
    assert!(v.contains(&"CH025".to_string()));
}

#[test]
fn ch025_ok_for_low_cardinality() {
    let schema = "CREATE TABLE t (d Date, s LowCardinality(String)) ENGINE = MergeTree ORDER BY d";
    let v = ch_ids("SELECT s, count() FROM t GROUP BY s", schema);
    assert!(!v.contains(&"CH025".to_string()));
}

fn ch_preprocess(sql: &str) -> sql_query_analyzer::preprocessor::PreprocessorResult {
    sql_query_analyzer::preprocessor::Preprocessor::new(SqlDialect::ClickHouse).process(sql)
}

#[test]
fn test_projection_without_name_is_ignored() {
    let result = ch_preprocess("CREATE TABLE t (a UInt64, PROJECTION (SELECT a))");
    assert!(result.metadata.projections.is_empty());
}

#[test]
fn test_projection_at_end_of_input_is_ignored() {
    let result = ch_preprocess("CREATE TABLE t (a UInt64) PROJECTION p");
    assert!(result.metadata.projections.is_empty());
}

#[test]
fn test_projection_unclosed_paren_is_ignored() {
    let result = ch_preprocess("CREATE TABLE t (a UInt64, PROJECTION p (SELECT a");
    assert!(result.metadata.projections.is_empty());
}

#[test]
fn test_projection_as_unclosed_is_ignored() {
    let result = ch_preprocess("CREATE TABLE t (a UInt64) PROJECTION p AS SELECT a");
    assert!(result.metadata.projections.is_empty());
}

#[test]
fn test_projection_paren_with_string_containing_paren() {
    let result = ch_preprocess("CREATE TABLE t (a UInt64, PROJECTION p (SELECT '(' ))");
    assert_eq!(result.metadata.projections.len(), 1);
    assert_eq!(result.metadata.projections[0].name, "p");
    assert!(result.metadata.projections[0].query.contains('('));
}

#[test]
fn test_projection_as_with_string_containing_comma() {
    let result =
        ch_preprocess("CREATE TABLE t (a UInt64, PROJECTION p AS SELECT 'x,y', b UInt64)");
    assert_eq!(result.metadata.projections.len(), 1);
    assert!(result.metadata.projections[0].query.contains("'x,y'"));
}

#[test]
fn test_projection_removal_drops_leading_comma() {
    let result = ch_preprocess("CREATE TABLE t (a UInt64, PROJECTION p AS SELECT a)");
    assert!(!result.sql.contains("PROJECTION"));
    assert!(!result.sql.contains(", )"));
}

#[test]
fn test_skip_index_granularity_overflow_falls_back_to_one() {
    let result = ch_preprocess(
        "CREATE TABLE t (a UInt64, INDEX i a TYPE minmax GRANULARITY 99999999999999999999999)"
    );
    assert_eq!(result.metadata.skip_indexes.len(), 1);
    assert_eq!(result.metadata.skip_indexes[0].granularity, 1);
}

#[test]
fn test_schema_aware_rules_ignore_unknown_tables() {
    let schema_sql = "CREATE TABLE known (a UInt64) ENGINE = MergeTree ORDER BY a";
    let ids = ch_ids(
        "SELECT * FROM missing JOIN known USING (a) WHERE a = 1 GROUP BY a HAVING count() > 1",
        schema_sql
    );
    assert!(
        !ids.contains(&"CH002".to_string()),
        "unknown table must not produce violations: {ids:?}"
    );
}

#[test]
fn test_primary_key_rules_skip_tables_without_sort_key() {
    let schema_sql = "CREATE TABLE nokey (a UInt64) ENGINE = MergeTree";
    let ids = ch_ids(
        "SELECT a FROM nokey WHERE a = 1 GROUP BY a ORDER BY a",
        schema_sql
    );
    assert!(
        !ids.contains(&"CH002".to_string()) && !ids.contains(&"CH011".to_string()),
        "table without ORDER BY must be skipped: {ids:?}"
    );
}

#[test]
fn test_group_by_string_key_ignores_unknown_columns() {
    let schema_sql = "CREATE TABLE t (a UInt64) ENGINE = MergeTree ORDER BY a";
    let ids = ch_ids("SELECT b FROM t GROUP BY b", schema_sql);
    assert!(
        !ids.contains(&"CH019".to_string()),
        "group column absent from table must be skipped: {ids:?}"
    );
}
