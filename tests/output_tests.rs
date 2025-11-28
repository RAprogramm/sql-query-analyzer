// SPDX-FileCopyrightText: 2025 RAprogramm
// SPDX-License-Identifier: MIT

use sql_query_analyzer::{
    output::{
        AnalysisResult, OutputFormat, OutputOptions, format_analysis_result,
        format_queries_summary, format_static_analysis
    },
    query::{Query, SqlDialect, parse_queries},
    rules::{AnalysisReport, RuleCategory, Severity, Violation}
};

fn sample_queries() -> Vec<Query> {
    parse_queries(
        "SELECT * FROM users; SELECT id FROM orders WHERE user_id = 1",
        SqlDialect::Generic
    )
    .unwrap()
}

fn make_violation(
    rule_id: &'static str,
    message: &str,
    severity: Severity,
    query_index: usize,
    suggestion: Option<&str>
) -> Violation {
    Violation {
        rule_id,
        rule_name: "Test Rule",
        message: message.to_string(),
        severity,
        category: RuleCategory::Performance,
        query_index,
        suggestion: suggestion.map(|s| s.to_string())
    }
}

#[test]
fn test_output_format_default() {
    let format = OutputFormat::default();
    assert!(matches!(format, OutputFormat::Text));
}

#[test]
fn test_output_options_default() {
    let opts = OutputOptions::default();
    assert!(matches!(opts.format, OutputFormat::Text));
    assert!(opts.colored);
    assert!(!opts.verbose);
}

#[test]
fn test_format_queries_summary_text() {
    let queries = sample_queries();
    let opts = OutputOptions {
        format:  OutputFormat::Text,
        colored: false,
        verbose: false
    };
    let output = format_queries_summary(&queries, &opts);
    assert!(output.contains("SQL Queries"));
    assert!(output.contains("users"));
}

#[test]
fn test_format_queries_summary_json() {
    let queries = sample_queries();
    let opts = OutputOptions {
        format:  OutputFormat::Json,
        colored: false,
        verbose: false
    };
    let output = format_queries_summary(&queries, &opts);
    assert!(output.starts_with('['));
    assert!(output.contains("users"));
}

#[test]
fn test_format_queries_summary_yaml() {
    let queries = sample_queries();
    let opts = OutputOptions {
        format:  OutputFormat::Yaml,
        colored: false,
        verbose: false
    };
    let output = format_queries_summary(&queries, &opts);
    assert!(output.contains("users"));
}

#[test]
fn test_format_queries_summary_sarif() {
    let queries = sample_queries();
    let opts = OutputOptions {
        format:  OutputFormat::Sarif,
        colored: false,
        verbose: false
    };
    let output = format_queries_summary(&queries, &opts);
    assert!(output.starts_with('['));
}

#[test]
fn test_format_queries_summary_with_verbose() {
    let queries = sample_queries();
    let opts = OutputOptions {
        format:  OutputFormat::Text,
        colored: false,
        verbose: true
    };
    let output = format_queries_summary(&queries, &opts);
    assert!(output.contains("Complexity"));
}

#[test]
fn test_format_queries_summary_colored() {
    let queries = sample_queries();
    let opts = OutputOptions {
        format:  OutputFormat::Text,
        colored: true,
        verbose: true
    };
    let output = format_queries_summary(&queries, &opts);
    assert!(output.contains("Complexity"));
}

#[test]
fn test_format_analysis_result_text() {
    let queries = sample_queries();
    let analysis = "Test analysis result";
    let opts = OutputOptions {
        format:  OutputFormat::Text,
        colored: false,
        verbose: false
    };
    let output = format_analysis_result(&queries, analysis, &opts);
    assert!(output.contains("SQL Query Analysis"));
    assert!(output.contains("Test analysis result"));
}

#[test]
fn test_format_analysis_result_text_colored() {
    let queries = sample_queries();
    let analysis = "Test analysis";
    let opts = OutputOptions {
        format:  OutputFormat::Text,
        colored: true,
        verbose: false
    };
    let output = format_analysis_result(&queries, analysis, &opts);
    assert!(output.contains("SQL Query Analysis"));
}

#[test]
fn test_format_analysis_result_json() {
    let queries = sample_queries();
    let analysis = "JSON analysis";
    let opts = OutputOptions {
        format:  OutputFormat::Json,
        colored: false,
        verbose: false
    };
    let output = format_analysis_result(&queries, analysis, &opts);
    assert!(output.contains("queries"));
    assert!(output.contains("analysis"));
}

#[test]
fn test_format_analysis_result_yaml() {
    let queries = sample_queries();
    let analysis = "YAML analysis";
    let opts = OutputOptions {
        format:  OutputFormat::Yaml,
        colored: false,
        verbose: false
    };
    let output = format_analysis_result(&queries, analysis, &opts);
    assert!(output.contains("queries"));
    assert!(output.contains("analysis"));
}

#[test]
fn test_format_static_analysis_no_violations() {
    let report = AnalysisReport::new(1, 1);
    let opts = OutputOptions {
        format:  OutputFormat::Text,
        colored: false,
        verbose: false
    };
    let output = format_static_analysis(&report, &opts);
    assert!(output.contains("No issues found"));
}

#[test]
fn test_format_static_analysis_no_violations_colored() {
    let report = AnalysisReport::new(1, 1);
    let opts = OutputOptions {
        format:  OutputFormat::Text,
        colored: true,
        verbose: false
    };
    let output = format_static_analysis(&report, &opts);
    assert!(output.contains("No issues found"));
}

#[test]
fn test_format_static_analysis_with_error() {
    let mut report = AnalysisReport::new(1, 1);
    report.add_violation(make_violation(
        "SEC001",
        "Missing WHERE clause",
        Severity::Error,
        0,
        Some("Add WHERE clause")
    ));
    let opts = OutputOptions {
        format:  OutputFormat::Text,
        colored: false,
        verbose: false
    };
    let output = format_static_analysis(&report, &opts);
    assert!(output.contains("ERROR"));
    assert!(output.contains("SEC001"));
    assert!(output.contains("Missing WHERE"));
    assert!(output.contains("Add WHERE clause"));
}

#[test]
fn test_format_static_analysis_with_warning() {
    let mut report = AnalysisReport::new(1, 1);
    report.add_violation(make_violation(
        "PERF001",
        "SELECT * detected",
        Severity::Warning,
        0,
        None
    ));
    let opts = OutputOptions {
        format:  OutputFormat::Text,
        colored: false,
        verbose: false
    };
    let output = format_static_analysis(&report, &opts);
    assert!(output.contains("WARN"));
    assert!(output.contains("PERF001"));
}

#[test]
fn test_format_static_analysis_with_info() {
    let mut report = AnalysisReport::new(1, 1);
    report.add_violation(make_violation(
        "STYLE001",
        "Consider using explicit columns",
        Severity::Info,
        0,
        None
    ));
    let opts = OutputOptions {
        format:  OutputFormat::Text,
        colored: false,
        verbose: false
    };
    let output = format_static_analysis(&report, &opts);
    assert!(output.contains("INFO"));
    assert!(output.contains("STYLE001"));
}

#[test]
fn test_format_static_analysis_colored_error() {
    let mut report = AnalysisReport::new(1, 1);
    report.add_violation(make_violation(
        "SEC001",
        "Error message",
        Severity::Error,
        0,
        Some("Fix it")
    ));
    let opts = OutputOptions {
        format:  OutputFormat::Text,
        colored: true,
        verbose: false
    };
    let output = format_static_analysis(&report, &opts);
    assert!(output.contains("SEC001"));
}

#[test]
fn test_format_static_analysis_colored_warning() {
    let mut report = AnalysisReport::new(1, 1);
    report.add_violation(make_violation(
        "PERF001",
        "Warning message",
        Severity::Warning,
        0,
        None
    ));
    let opts = OutputOptions {
        format:  OutputFormat::Text,
        colored: true,
        verbose: false
    };
    let output = format_static_analysis(&report, &opts);
    assert!(output.contains("PERF001"));
}

#[test]
fn test_format_static_analysis_colored_info() {
    let mut report = AnalysisReport::new(1, 1);
    report.add_violation(make_violation(
        "INFO001",
        "Info message",
        Severity::Info,
        0,
        None
    ));
    let opts = OutputOptions {
        format:  OutputFormat::Text,
        colored: true,
        verbose: false
    };
    let output = format_static_analysis(&report, &opts);
    assert!(output.contains("INFO001"));
}

#[test]
fn test_format_static_analysis_json() {
    let mut report = AnalysisReport::new(1, 1);
    report.add_violation(make_violation(
        "TEST001",
        "Test",
        Severity::Warning,
        0,
        None
    ));
    let opts = OutputOptions {
        format:  OutputFormat::Json,
        colored: false,
        verbose: false
    };
    let output = format_static_analysis(&report, &opts);
    assert!(output.contains("violations"));
    assert!(output.contains("TEST001"));
}

#[test]
fn test_format_static_analysis_yaml() {
    let mut report = AnalysisReport::new(1, 1);
    report.add_violation(make_violation(
        "TEST001",
        "Test",
        Severity::Warning,
        0,
        None
    ));
    let opts = OutputOptions {
        format:  OutputFormat::Yaml,
        colored: false,
        verbose: false
    };
    let output = format_static_analysis(&report, &opts);
    assert!(output.contains("violations"));
}

#[test]
fn test_format_static_analysis_sarif() {
    let mut report = AnalysisReport::new(3, 1);
    report.add_violation(make_violation(
        "SEC001",
        "Security issue",
        Severity::Error,
        0,
        None
    ));
    report.add_violation(make_violation(
        "PERF001",
        "Performance issue",
        Severity::Warning,
        1,
        None
    ));
    report.add_violation(make_violation(
        "STYLE001",
        "Style issue",
        Severity::Info,
        2,
        None
    ));
    let opts = OutputOptions {
        format:  OutputFormat::Sarif,
        colored: false,
        verbose: false
    };
    let output = format_static_analysis(&report, &opts);
    assert!(output.contains("$schema"));
    assert!(output.contains("sarif"));
    assert!(output.contains("sql-query-analyzer"));
    assert!(output.contains("SEC001"));
    assert!(output.contains("error"));
    assert!(output.contains("warning"));
    assert!(output.contains("note"));
}

#[test]
fn test_format_static_analysis_multiple_queries() {
    let mut report = AnalysisReport::new(2, 1);
    report.add_violation(make_violation(
        "PERF001",
        "Issue 1",
        Severity::Warning,
        0,
        None
    ));
    report.add_violation(make_violation(
        "PERF002",
        "Issue 2",
        Severity::Warning,
        0,
        None
    ));
    report.add_violation(make_violation(
        "SEC001",
        "Issue 3",
        Severity::Error,
        1,
        None
    ));
    let opts = OutputOptions {
        format:  OutputFormat::Text,
        colored: false,
        verbose: false
    };
    let output = format_static_analysis(&report, &opts);
    assert!(output.contains("Query #1"));
    assert!(output.contains("Query #2"));
}

#[test]
fn test_output_format_debug() {
    let format = OutputFormat::Text;
    let debug = format!("{:?}", format);
    assert!(debug.contains("Text"));
}

#[test]
fn test_output_format_clone() {
    let format = OutputFormat::Json;
    let cloned = format.clone();
    assert!(matches!(cloned, OutputFormat::Json));
}

#[test]
fn test_output_options_debug() {
    let opts = OutputOptions::default();
    let debug = format!("{:?}", opts);
    assert!(debug.contains("OutputOptions"));
}

#[test]
fn test_output_options_clone() {
    let opts = OutputOptions {
        format:  OutputFormat::Yaml,
        colored: false,
        verbose: true
    };
    let cloned = opts.clone();
    assert!(matches!(cloned.format, OutputFormat::Yaml));
    assert!(!cloned.colored);
    assert!(cloned.verbose);
}

#[test]
fn test_analysis_result_debug() {
    let result = AnalysisResult {
        queries:  vec![],
        analysis: "test".to_string()
    };
    let debug = format!("{:?}", result);
    assert!(debug.contains("AnalysisResult"));
}

#[test]
fn test_format_queries_with_ctes() {
    let queries = parse_queries(
        "WITH temp AS (SELECT 1) SELECT * FROM temp",
        SqlDialect::Generic
    )
    .unwrap();
    let opts = OutputOptions {
        format:  OutputFormat::Text,
        colored: false,
        verbose: false
    };
    let output = format_queries_summary(&queries, &opts);
    assert!(output.contains("CTEs"));
    assert!(output.contains("temp"));
}

#[test]
fn test_format_queries_with_joins() {
    let queries = parse_queries(
        "SELECT u.id FROM users u INNER JOIN orders o ON u.id = o.user_id",
        SqlDialect::Generic
    )
    .unwrap();
    let opts = OutputOptions {
        format:  OutputFormat::Text,
        colored: false,
        verbose: false
    };
    let output = format_queries_summary(&queries, &opts);
    assert!(output.contains("JOIN columns"));
}

#[test]
fn test_format_queries_with_order_by() {
    let queries = parse_queries("SELECT * FROM users ORDER BY name", SqlDialect::Generic).unwrap();
    let opts = OutputOptions {
        format:  OutputFormat::Text,
        colored: false,
        verbose: false
    };
    let output = format_queries_summary(&queries, &opts);
    assert!(output.contains("ORDER BY columns"));
}

#[test]
fn test_format_queries_with_group_by() {
    let queries = parse_queries(
        "SELECT count(*) FROM users GROUP BY status",
        SqlDialect::Generic
    )
    .unwrap();
    let opts = OutputOptions {
        format:  OutputFormat::Text,
        colored: false,
        verbose: false
    };
    let output = format_queries_summary(&queries, &opts);
    assert!(output.contains("GROUP BY columns"));
}

#[test]
fn test_format_queries_with_having() {
    let queries = parse_queries(
        "SELECT count(*) as cnt, status FROM users GROUP BY status HAVING status = 'active'",
        SqlDialect::Generic
    )
    .unwrap();
    let opts = OutputOptions {
        format:  OutputFormat::Text,
        colored: false,
        verbose: false
    };
    let output = format_queries_summary(&queries, &opts);
    assert!(output.contains("HAVING columns"));
}

#[test]
fn test_format_queries_with_limit_offset() {
    let queries = parse_queries(
        "SELECT * FROM users LIMIT 10 OFFSET 20",
        SqlDialect::Generic
    )
    .unwrap();
    let opts = OutputOptions {
        format:  OutputFormat::Text,
        colored: false,
        verbose: false
    };
    let output = format_queries_summary(&queries, &opts);
    assert!(output.contains("LIMIT: 10"));
    assert!(output.contains("OFFSET: 20"));
}

#[test]
fn test_format_queries_with_distinct() {
    let queries = parse_queries("SELECT DISTINCT status FROM users", SqlDialect::Generic).unwrap();
    let opts = OutputOptions {
        format:  OutputFormat::Text,
        colored: false,
        verbose: false
    };
    let output = format_queries_summary(&queries, &opts);
    assert!(output.contains("DISTINCT"));
}

#[test]
fn test_format_queries_with_union() {
    let queries = parse_queries(
        "SELECT id FROM users UNION SELECT id FROM admins",
        SqlDialect::Generic
    )
    .unwrap();
    let opts = OutputOptions {
        format:  OutputFormat::Text,
        colored: false,
        verbose: false
    };
    let output = format_queries_summary(&queries, &opts);
    assert!(output.contains("UNION"));
}

#[test]
fn test_format_queries_with_subquery() {
    let queries = parse_queries(
        "SELECT * FROM users WHERE id IN (SELECT user_id FROM orders)",
        SqlDialect::Generic
    )
    .unwrap();
    let opts = OutputOptions {
        format:  OutputFormat::Text,
        colored: false,
        verbose: false
    };
    let output = format_queries_summary(&queries, &opts);
    assert!(output.contains("subquery"));
}

#[test]
fn test_format_queries_verbose_low_complexity() {
    let queries = parse_queries("SELECT id FROM users", SqlDialect::Generic).unwrap();
    let opts = OutputOptions {
        format:  OutputFormat::Text,
        colored: false,
        verbose: true
    };
    let output = format_queries_summary(&queries, &opts);
    assert!(output.contains("Low"));
}

#[test]
fn test_format_queries_verbose_medium_complexity() {
    let queries = parse_queries(
        "SELECT u.id, o.total FROM users u JOIN orders o ON u.id = o.user_id WHERE u.status = \
         'active' GROUP BY u.id ORDER BY o.total DESC",
        SqlDialect::Generic
    )
    .unwrap();
    let opts = OutputOptions {
        format:  OutputFormat::Text,
        colored: false,
        verbose: true
    };
    let output = format_queries_summary(&queries, &opts);
    assert!(output.contains("Complexity"));
}

#[test]
fn test_format_queries_verbose_high_complexity() {
    let queries = parse_queries(
        "SELECT u.id, (SELECT COUNT(*) FROM orders WHERE user_id = u.id) as order_count, (SELECT \
         SUM(total) FROM orders WHERE user_id = u.id) as total_spent FROM users u JOIN profiles p \
         ON u.id = p.user_id JOIN addresses a ON u.id = a.user_id LEFT JOIN preferences pr ON \
         u.id = pr.user_id WHERE u.status = 'active' AND u.created_at > '2020-01-01' GROUP BY \
         u.id HAVING COUNT(*) > 1 ORDER BY u.id DESC LIMIT 100 OFFSET 50",
        SqlDialect::Generic
    )
    .unwrap();
    let opts = OutputOptions {
        format:  OutputFormat::Text,
        colored: false,
        verbose: true
    };
    let output = format_queries_summary(&queries, &opts);
    assert!(output.contains("Complexity"));
}

#[test]
fn test_format_queries_verbose_colored_high() {
    let queries = parse_queries(
        "SELECT u.id, (SELECT COUNT(*) FROM orders WHERE user_id = u.id) as cnt FROM users u JOIN \
         orders o ON u.id = o.user_id JOIN items i ON o.id = i.order_id WHERE u.active = true \
         GROUP BY u.id HAVING COUNT(*) > 5 ORDER BY cnt DESC LIMIT 100 OFFSET 1000",
        SqlDialect::Generic
    )
    .unwrap();
    let opts = OutputOptions {
        format:  OutputFormat::Text,
        colored: true,
        verbose: true
    };
    let output = format_queries_summary(&queries, &opts);
    assert!(output.contains("Complexity"));
}

#[test]
fn test_format_queries_verbose_colored_medium() {
    let queries = parse_queries(
        "SELECT u.id FROM users u JOIN orders o ON u.id = o.user_id WHERE u.status = 'active' \
         ORDER BY u.id",
        SqlDialect::Generic
    )
    .unwrap();
    let opts = OutputOptions {
        format:  OutputFormat::Text,
        colored: true,
        verbose: true
    };
    let output = format_queries_summary(&queries, &opts);
    assert!(output.contains("Complexity"));
}

#[test]
fn test_format_queries_with_window_functions() {
    let queries = parse_queries(
        "SELECT id, ROW_NUMBER() OVER (PARTITION BY status ORDER BY created_at) as rn FROM users",
        SqlDialect::Generic
    )
    .unwrap();
    let opts = OutputOptions {
        format:  OutputFormat::Text,
        colored: false,
        verbose: false
    };
    let output = format_queries_summary(&queries, &opts);
    assert!(output.contains("Window functions"));
}

#[test]
fn test_analysis_report_new() {
    let report = AnalysisReport::new(5, 10);
    assert_eq!(report.queries_count, 5);
    assert_eq!(report.rules_count, 10);
    assert!(report.violations.is_empty());
}

#[test]
fn test_analysis_report_counts() {
    let mut report = AnalysisReport::new(3, 5);
    report.add_violation(make_violation("E1", "err1", Severity::Error, 0, None));
    report.add_violation(make_violation("E2", "err2", Severity::Error, 0, None));
    report.add_violation(make_violation("W1", "warn1", Severity::Warning, 1, None));
    report.add_violation(make_violation("I1", "info1", Severity::Info, 2, None));
    report.add_violation(make_violation("I2", "info2", Severity::Info, 2, None));
    report.add_violation(make_violation("I3", "info3", Severity::Info, 2, None));

    assert_eq!(report.error_count(), 2);
    assert_eq!(report.warning_count(), 1);
    assert_eq!(report.info_count(), 3);
}
