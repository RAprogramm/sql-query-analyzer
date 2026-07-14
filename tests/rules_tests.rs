// SPDX-FileCopyrightText: 2025 RAprogramm
// SPDX-License-Identifier: MIT

use sql_query_analyzer::{
    config::RulesConfig,
    query::{SqlDialect, parse_queries},
    rules::{RuleRunner, Severity},
    schema::Schema
};

fn analyze_query(sql: &str) -> Vec<String> {
    let queries = parse_queries(sql, SqlDialect::Generic).unwrap();
    let runner = RuleRunner::new();
    let report = runner.analyze(&queries);
    report
        .violations
        .iter()
        .map(|v| v.rule_id.to_string())
        .collect()
}

fn analyze_with_schema(sql: &str, schema_sql: &str) -> Vec<String> {
    let queries = parse_queries(sql, SqlDialect::Generic).unwrap();
    let schema = Schema::parse(schema_sql, SqlDialect::Generic).unwrap();
    let runner = RuleRunner::with_schema_and_config(schema, RulesConfig::default());
    let report = runner.analyze(&queries);
    report
        .violations
        .iter()
        .map(|v| v.rule_id.to_string())
        .collect()
}

#[test]
fn test_select_star_without_limit() {
    let violations = analyze_query("SELECT * FROM users");
    assert!(violations.contains(&"PERF001".to_string()));
}

#[test]
fn test_select_star_with_limit() {
    let violations = analyze_query("SELECT * FROM users LIMIT 10");
    assert!(!violations.contains(&"PERF001".to_string()));
}

#[test]
fn test_leading_wildcard() {
    let violations = analyze_query("SELECT * FROM users WHERE name LIKE '%test'");
    assert!(violations.contains(&"PERF002".to_string()));
}

#[test]
fn test_trailing_wildcard_ok() {
    let violations = analyze_query("SELECT * FROM users WHERE name LIKE 'test%' LIMIT 10");
    assert!(!violations.contains(&"PERF002".to_string()));
}

#[test]
fn test_large_offset() {
    let violations = analyze_query("SELECT * FROM users LIMIT 10 OFFSET 5000");
    assert!(violations.contains(&"PERF004".to_string()));
}

#[test]
fn test_small_offset_ok() {
    let violations = analyze_query("SELECT * FROM users LIMIT 10 OFFSET 100");
    assert!(!violations.contains(&"PERF004".to_string()));
}

#[test]
fn test_select_without_where() {
    let violations = analyze_query("SELECT * FROM users");
    assert!(violations.contains(&"PERF011".to_string()));
}

#[test]
fn test_select_with_where() {
    let violations = analyze_query("SELECT * FROM users WHERE id = 1 LIMIT 10");
    assert!(!violations.contains(&"PERF011".to_string()));
}

#[test]
fn test_select_star_style() {
    let violations = analyze_query("SELECT * FROM users LIMIT 10");
    assert!(violations.contains(&"STYLE001".to_string()));
}

#[test]
fn test_ordinal_in_order_by() {
    let violations = analyze_query("SELECT id, name FROM users WHERE id > 0 ORDER BY 1 LIMIT 5");
    assert!(violations.contains(&"STYLE004".to_string()));
}

#[test]
fn test_ordinal_in_group_by() {
    let violations =
        analyze_query("SELECT name, COUNT(*) FROM users WHERE id > 0 GROUP BY 1 LIMIT 5");
    assert!(violations.contains(&"STYLE004".to_string()));
}

#[test]
fn test_ordinal_in_order_by_list() {
    let violations =
        analyze_query("SELECT id, name FROM users WHERE id > 0 ORDER BY name, 2 LIMIT 5");
    assert!(violations.contains(&"STYLE004".to_string()));
}

#[test]
fn test_explicit_order_by_ok() {
    let violations =
        analyze_query("SELECT id, name FROM users WHERE id > 0 ORDER BY name DESC LIMIT 5");
    assert!(!violations.contains(&"STYLE004".to_string()));
}

#[test]
fn test_limit_count_not_ordinal() {
    let violations =
        analyze_query("SELECT id, name FROM users WHERE id > 0 ORDER BY name LIMIT 1");
    assert!(!violations.contains(&"STYLE004".to_string()));
}

#[test]
fn test_function_args_not_ordinal() {
    let violations = analyze_query(
        "SELECT id, name FROM users WHERE id > 0 ORDER BY COALESCE(name, 1) LIMIT 5"
    );
    assert!(!violations.contains(&"STYLE004".to_string()));
}

#[test]
fn test_count_star_without_where() {
    let violations = analyze_query("SELECT COUNT(*) FROM users");
    assert!(violations.contains(&"PERF012".to_string()));
}

#[test]
fn test_count_one_without_where() {
    let violations = analyze_query("SELECT COUNT(1) FROM orders");
    assert!(violations.contains(&"PERF012".to_string()));
}

#[test]
fn test_count_with_where_ok() {
    let violations = analyze_query("SELECT COUNT(*) FROM users WHERE status = 'active'");
    assert!(!violations.contains(&"PERF012".to_string()));
}

#[test]
fn test_select_without_count_not_perf012() {
    let violations = analyze_query("SELECT id FROM users");
    assert!(!violations.contains(&"PERF012".to_string()));
}

#[test]
fn test_order_by_rand_mysql() {
    let violations = analyze_query("SELECT id FROM users ORDER BY RAND() LIMIT 5");
    assert!(violations.contains(&"PERF013".to_string()));
}

#[test]
fn test_order_by_random_postgres() {
    let violations = analyze_query("SELECT id FROM users ORDER BY RANDOM() LIMIT 5");
    assert!(violations.contains(&"PERF013".to_string()));
}

#[test]
fn test_order_by_newid_mssql() {
    let violations = analyze_query("SELECT id FROM users ORDER BY NEWID() LIMIT 5");
    assert!(violations.contains(&"PERF013".to_string()));
}

#[test]
fn test_order_by_column_ok() {
    let violations = analyze_query("SELECT id FROM users WHERE id > 1 ORDER BY id LIMIT 5");
    assert!(!violations.contains(&"PERF013".to_string()));
}

#[test]
fn test_rand_in_where_not_order_by_ok() {
    let violations =
        analyze_query("SELECT id FROM users WHERE id >= FLOOR(RAND() * 100) ORDER BY id LIMIT 5");
    assert!(!violations.contains(&"PERF013".to_string()));
}

#[test]
fn test_explicit_columns_ok() {
    let violations = analyze_query("SELECT id, name FROM users LIMIT 10");
    assert!(!violations.contains(&"STYLE001".to_string()));
}

#[test]
fn test_injection_tautology_quoted() {
    let violations = analyze_query("SELECT id FROM users WHERE name = '' OR '1' = '1' LIMIT 10");
    assert!(violations.contains(&"SEC006".to_string()));
}

#[test]
fn test_injection_tautology_numeric() {
    let violations = analyze_query("SELECT id FROM users WHERE id = 5 OR 1 = 1 LIMIT 10");
    assert!(violations.contains(&"SEC006".to_string()));
}

#[test]
fn test_injection_tautology_empty_strings() {
    let violations = analyze_query("SELECT id FROM users WHERE name = 'a' OR '' = '' LIMIT 10");
    assert!(violations.contains(&"SEC006".to_string()));
}

#[test]
fn test_or_on_columns_not_tautology() {
    let violations = analyze_query("SELECT id FROM users WHERE id = 1 OR id = 2 LIMIT 10");
    assert!(!violations.contains(&"SEC006".to_string()));
}

#[test]
fn test_or_different_literals_not_tautology() {
    let violations =
        analyze_query("SELECT id FROM users WHERE name = 'a' OR status = 'b' LIMIT 10");
    assert!(!violations.contains(&"SEC006".to_string()));
}

#[test]
fn test_hardcoded_credential_insert() {
    let violations =
        analyze_query("INSERT INTO users (email, password) VALUES ('a@b.c', 'admin123')");
    assert!(violations.contains(&"SEC008".to_string()));
}

#[test]
fn test_hardcoded_credential_update_assignment() {
    let violations = analyze_query("UPDATE users SET api_key = 'sk-live-abc123' WHERE id = 1");
    assert!(violations.contains(&"SEC008".to_string()));
}

#[test]
fn test_hardcoded_credential_prefixed_column() {
    let violations = analyze_query("UPDATE users SET user_password = 'hunter2' WHERE id = 1");
    assert!(violations.contains(&"SEC008".to_string()));
}

#[test]
fn test_insert_without_sensitive_columns_ok() {
    let violations = analyze_query("INSERT INTO users (email, name) VALUES ('a@b.c', 'Alice')");
    assert!(!violations.contains(&"SEC008".to_string()));
}

#[test]
fn test_author_column_not_credential() {
    let violations = analyze_query("UPDATE posts SET author = 'Alice' WHERE id = 1");
    assert!(!violations.contains(&"SEC008".to_string()));
}

#[test]
fn test_update_without_where() {
    let violations = analyze_query("UPDATE users SET status = 'inactive'");
    assert!(violations.contains(&"SEC001".to_string()));
}

#[test]
fn test_update_with_where() {
    let violations = analyze_query("UPDATE users SET status = 'inactive' WHERE id = 1");
    assert!(!violations.contains(&"SEC001".to_string()));
}

#[test]
fn test_delete_without_where() {
    let violations = analyze_query("DELETE FROM users");
    assert!(violations.contains(&"SEC002".to_string()));
}

#[test]
fn test_delete_with_where() {
    let violations = analyze_query("DELETE FROM users WHERE id = 1");
    assert!(!violations.contains(&"SEC002".to_string()));
}

#[test]
fn test_union_without_all() {
    let violations = analyze_query("SELECT id FROM users UNION SELECT id FROM admins");
    assert!(violations.contains(&"PERF010".to_string()));
}

#[test]
fn test_union_all_ok() {
    let violations = analyze_query("SELECT id FROM users UNION ALL SELECT id FROM admins");
    assert!(!violations.contains(&"PERF010".to_string()));
}

#[test]
fn test_distinct_with_order_by() {
    let violations = analyze_query("SELECT DISTINCT status FROM orders ORDER BY status");
    assert!(violations.contains(&"PERF006".to_string()));
}

#[test]
fn test_schema_missing_index() {
    let schema = "CREATE TABLE users (id INT PRIMARY KEY, email VARCHAR(255))";
    let violations = analyze_with_schema(
        "SELECT * FROM users WHERE email = 'test@test.com' LIMIT 10",
        schema
    );
    assert!(violations.contains(&"SCHEMA001".to_string()));
}

#[test]
fn test_schema_with_index() {
    let schema = r#"
        CREATE TABLE users (id INT PRIMARY KEY, email VARCHAR(255));
        CREATE INDEX idx_email ON users(email);
    "#;
    let violations = analyze_with_schema(
        "SELECT * FROM users WHERE email = 'test@test.com' LIMIT 10",
        schema
    );
    assert!(!violations.contains(&"SCHEMA001".to_string()));
}

#[test]
fn test_rule_disabled() {
    let queries = parse_queries("SELECT * FROM users", SqlDialect::Generic).unwrap();
    let config = RulesConfig {
        disabled: vec![
            "PERF001".to_string(),
            "PERF011".to_string(),
            "STYLE001".to_string(),
        ],
        ..Default::default()
    };
    let runner = RuleRunner::with_config(config);
    let report = runner.analyze(&queries);
    let rule_ids: Vec<_> = report.violations.iter().map(|v| v.rule_id).collect();
    assert!(!rule_ids.contains(&"PERF001"));
    assert!(!rule_ids.contains(&"PERF011"));
    assert!(!rule_ids.contains(&"STYLE001"));
}

#[test]
fn test_severity_override() {
    let queries = parse_queries("SELECT * FROM users", SqlDialect::Generic).unwrap();
    let mut severity = std::collections::HashMap::new();
    severity.insert("STYLE001".to_string(), "error".to_string());
    let config = RulesConfig {
        disabled: vec![],
        severity
    };
    let runner = RuleRunner::with_config(config);
    let report = runner.analyze(&queries);
    let style_violation = report.violations.iter().find(|v| v.rule_id == "STYLE001");
    assert!(style_violation.is_some());
    assert_eq!(style_violation.unwrap().severity, Severity::Error);
}

#[test]
fn test_error_count() {
    let queries = parse_queries("DELETE FROM users", SqlDialect::Generic).unwrap();
    let runner = RuleRunner::new();
    let report = runner.analyze(&queries);
    assert!(report.error_count() > 0);
}

#[test]
fn test_warning_count() {
    let queries = parse_queries("SELECT * FROM users", SqlDialect::Generic).unwrap();
    let runner = RuleRunner::new();
    let report = runner.analyze(&queries);
    assert!(report.warning_count() > 0);
}

#[test]
fn test_no_violations_for_good_query() {
    let queries = parse_queries(
        "SELECT id, name FROM users WHERE id = 1 LIMIT 10",
        SqlDialect::Generic
    )
    .unwrap();
    let runner = RuleRunner::new();
    let report = runner.analyze(&queries);
    assert_eq!(report.error_count(), 0);
}

#[test]
fn test_multiple_violations() {
    let queries = parse_queries("SELECT * FROM users", SqlDialect::Generic).unwrap();
    let runner = RuleRunner::new();
    let report = runner.analyze(&queries);
    assert!(report.violations.len() >= 2);
}

#[test]
fn test_insert_no_violations() {
    let queries = parse_queries(
        "INSERT INTO users (id, name) VALUES (1, 'test')",
        SqlDialect::Generic
    )
    .unwrap();
    let runner = RuleRunner::new();
    let report = runner.analyze(&queries);
    assert_eq!(report.error_count(), 0);
    assert_eq!(report.warning_count(), 0);
}

#[test]
fn test_scalar_subquery() {
    let violations = analyze_query(
        "SELECT id, (SELECT COUNT(*) FROM orders WHERE orders.user_id = users.id) FROM users LIMIT 10"
    );
    assert!(violations.contains(&"PERF007".to_string()));
}

#[test]
fn test_function_on_column_year() {
    let violations = analyze_query("SELECT * FROM orders WHERE YEAR(created_at) = 2024 LIMIT 10");
    assert!(violations.contains(&"PERF008".to_string()));
}

#[test]
fn test_function_on_column_upper() {
    let violations = analyze_query("SELECT * FROM users WHERE UPPER(name) = 'JOHN' LIMIT 10");
    assert!(violations.contains(&"PERF008".to_string()));
}

#[test]
fn test_function_on_column_lower() {
    let violations =
        analyze_query("SELECT * FROM users WHERE LOWER(email) = 'test@test.com' LIMIT 10");
    assert!(violations.contains(&"PERF008".to_string()));
}

#[test]
fn test_function_on_column_trim() {
    let violations = analyze_query("SELECT * FROM users WHERE TRIM(name) = 'John' LIMIT 10");
    assert!(violations.contains(&"PERF008".to_string()));
}

#[test]
fn test_function_on_column_cast() {
    let violations = analyze_query("SELECT * FROM users WHERE CAST(id AS VARCHAR) = '1' LIMIT 10");
    assert!(violations.contains(&"PERF008".to_string()));
}

#[test]
fn test_function_on_column_coalesce() {
    let violations =
        analyze_query("SELECT * FROM users WHERE COALESCE(status, 'unknown') = 'active' LIMIT 10");
    assert!(violations.contains(&"PERF008".to_string()));
}

#[test]
fn test_not_in_with_subquery() {
    let violations =
        analyze_query("SELECT * FROM users WHERE id NOT IN (SELECT user_id FROM banned) LIMIT 10");
    assert!(violations.contains(&"PERF009".to_string()));
}

#[test]
fn test_or_instead_of_in() {
    let violations = analyze_query(
        "SELECT * FROM users WHERE status = 'a' OR status = 'b' OR status = 'c' OR status = 'd' LIMIT 10"
    );
    assert!(violations.contains(&"PERF003".to_string()));
}

#[test]
fn test_cartesian_product() {
    let violations = analyze_query("SELECT * FROM users, orders LIMIT 10");
    assert!(violations.contains(&"PERF005".to_string()));
}

#[test]
fn test_cartesian_product_with_where() {
    let violations =
        analyze_query("SELECT * FROM users, orders WHERE users.id = orders.user_id LIMIT 10");
    assert!(!violations.contains(&"PERF005".to_string()));
}

#[test]
fn test_leading_wildcard_double_quote() {
    let violations = analyze_query(r#"SELECT * FROM users WHERE name LIKE "%test" LIMIT 10"#);
    assert!(violations.contains(&"PERF002".to_string()));
}

#[test]
fn test_select_star_double_space() {
    let violations = analyze_query("SELECT  * FROM users");
    assert!(violations.contains(&"PERF001".to_string()));
}

#[test]
fn test_join_missing_alias() {
    let violations = analyze_query(
        "SELECT users.id FROM users INNER JOIN orders ON users.id = orders.user_id LIMIT 10"
    );
    assert!(violations.contains(&"STYLE002".to_string()));
}

#[test]
fn test_schema_join_column_missing_index() {
    let schema = r#"
        CREATE TABLE users (id INT PRIMARY KEY);
        CREATE TABLE orders (id INT PRIMARY KEY, user_id INT);
    "#;
    let violations = analyze_with_schema(
        "SELECT * FROM users u INNER JOIN orders o ON u.id = o.user_id LIMIT 10",
        schema
    );
    assert!(violations.contains(&"SCHEMA001".to_string()));
}

#[test]
fn test_schema_order_by_missing_index() {
    let schema = "CREATE TABLE users (id INT PRIMARY KEY, name VARCHAR(255))";
    let violations = analyze_with_schema("SELECT * FROM users ORDER BY name LIMIT 10", schema);
    assert!(violations.contains(&"SCHEMA003".to_string()));
}

#[test]
fn test_schema_column_not_in_schema() {
    let schema = "CREATE TABLE users (id INT PRIMARY KEY, name VARCHAR(255))";
    let violations = analyze_with_schema(
        "SELECT * FROM users WHERE nonexistent_col = 'test' LIMIT 10",
        schema
    );
    assert!(violations.contains(&"SCHEMA002".to_string()));
}

#[test]
fn test_schema_large_table_no_index() {
    let schema = r#"
        CREATE TABLE users (id INT PRIMARY KEY, email VARCHAR(255));
        INSERT INTO users VALUES (1, 'a');
        INSERT INTO users VALUES (2, 'b');
        INSERT INTO users VALUES (3, 'c');
    "#;
    let violations =
        analyze_with_schema("SELECT * FROM users WHERE email = 'test' LIMIT 10", schema);
    assert!(violations.contains(&"SCHEMA001".to_string()));
}

#[test]
fn test_multiple_queries() {
    let violations = analyze_query("SELECT * FROM users; DELETE FROM orders");
    assert!(violations.contains(&"PERF001".to_string()));
    assert!(violations.contains(&"SEC002".to_string()));
}

#[test]
fn test_truncate_detected() {
    let violations = analyze_query("TRUNCATE TABLE users");
    assert!(violations.contains(&"SEC003".to_string()));
}

#[test]
fn test_truncate_without_table_keyword() {
    let violations = analyze_query("TRUNCATE users");
    assert!(violations.contains(&"SEC003".to_string()));
}

#[test]
fn test_truncate_multiple_tables() {
    let violations = analyze_query("TRUNCATE TABLE users, orders");
    assert!(violations.contains(&"SEC003".to_string()));
}

#[test]
fn test_drop_table_detected() {
    let violations = analyze_query("DROP TABLE users");
    assert!(violations.contains(&"SEC004".to_string()));
}

#[test]
fn test_drop_table_if_exists() {
    let violations = analyze_query("DROP TABLE IF EXISTS users");
    assert!(violations.contains(&"SEC004".to_string()));
}

#[test]
fn test_drop_database_detected() {
    let violations = analyze_query("DROP DATABASE production");
    assert!(violations.contains(&"SEC004".to_string()));
}

#[test]
fn test_drop_index_detected() {
    let violations = analyze_query("DROP INDEX idx_users_email");
    assert!(violations.contains(&"SEC004".to_string()));
}
