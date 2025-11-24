use sql_query_analyzer::config::RulesConfig;
use sql_query_analyzer::query::{parse_queries, SqlDialect};
use sql_query_analyzer::rules::{RuleRunner, Severity};
use sql_query_analyzer::schema::Schema;

fn analyze_query(sql: &str) -> Vec<String> {
    let queries = parse_queries(sql, SqlDialect::Generic).unwrap();
    let runner = RuleRunner::new();
    let report = runner.analyze(&queries);
    report.violations.iter().map(|v| v.rule_id.to_string()).collect()
}

fn analyze_with_schema(sql: &str, schema_sql: &str) -> Vec<String> {
    let queries = parse_queries(sql, SqlDialect::Generic).unwrap();
    let schema = Schema::parse(schema_sql).unwrap();
    let runner = RuleRunner::with_schema_and_config(schema, RulesConfig::default());
    let report = runner.analyze(&queries);
    report.violations.iter().map(|v| v.rule_id.to_string()).collect()
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
fn test_explicit_columns_ok() {
    let violations = analyze_query("SELECT id, name FROM users LIMIT 10");
    assert!(!violations.contains(&"STYLE001".to_string()));
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
        disabled: vec!["PERF001".to_string(), "PERF011".to_string(), "STYLE001".to_string()],
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
    ).unwrap();
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
    ).unwrap();
    let runner = RuleRunner::new();
    let report = runner.analyze(&queries);

    assert_eq!(report.error_count(), 0);
    assert_eq!(report.warning_count(), 0);
}
