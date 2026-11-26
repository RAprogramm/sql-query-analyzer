use std::fmt::Write;

use colored::Colorize;
use serde::Serialize;

pub mod yaml;

use crate::{
    query::Query,
    rules::{AnalysisReport, Severity}
};

/// Output format for results
#[derive(Debug, Clone, Copy, Default)]
pub enum OutputFormat {
    #[default]
    Text,
    Json,
    Yaml,
    Sarif
}

/// Output options
#[derive(Debug, Clone)]
pub struct OutputOptions {
    pub format:  OutputFormat,
    pub colored: bool,
    pub verbose: bool
}

impl Default for OutputOptions {
    fn default() -> Self {
        Self {
            format:  OutputFormat::Text,
            colored: true,
            verbose: false
        }
    }
}

/// Analysis result for serialization
#[derive(Debug, Serialize)]
pub struct AnalysisResult {
    pub queries:  Vec<Query>,
    pub analysis: String
}

/// Format queries summary based on output options
pub fn format_queries_summary(queries: &[Query], opts: &OutputOptions) -> String {
    match opts.format {
        OutputFormat::Json | OutputFormat::Sarif => {
            serde_json::to_string_pretty(queries).unwrap_or_default()
        }
        OutputFormat::Yaml => yaml::to_string(&queries).unwrap_or_default(),
        OutputFormat::Text => format_text_summary(queries, opts)
    }
}

/// Format full analysis result
pub fn format_analysis_result(queries: &[Query], analysis: &str, opts: &OutputOptions) -> String {
    match opts.format {
        OutputFormat::Json | OutputFormat::Sarif => {
            let result = AnalysisResult {
                queries:  queries.to_vec(),
                analysis: analysis.to_string()
            };
            serde_json::to_string_pretty(&result).unwrap_or_default()
        }
        OutputFormat::Yaml => {
            let result = AnalysisResult {
                queries:  queries.to_vec(),
                analysis: analysis.to_string()
            };
            yaml::to_string(&result).unwrap_or_default()
        }
        OutputFormat::Text => {
            let mut output = String::new();
            if opts.colored {
                output.push_str(&"=== SQL Query Analysis ===\n\n".bold().to_string());
            } else {
                output.push_str("=== SQL Query Analysis ===\n\n");
            }
            output.push_str(analysis);
            output
        }
    }
}

fn format_text_summary(queries: &[Query], opts: &OutputOptions) -> String {
    let mut summary = String::from("SQL Queries:\n\n");
    for (i, query) in queries.iter().enumerate() {
        let header = format!("Query #{} ({}):", i + 1, query.kind);
        if opts.colored {
            summary.push_str(&header.cyan().bold().to_string());
        } else {
            summary.push_str(&header);
        }
        summary.push('\n');
        let _ = writeln!(summary, "{}", query.raw);
        if !query.cte_names.is_empty() {
            let ctes: Vec<&str> = query
                .cte_names
                .iter()
                .map(compact_str::CompactString::as_str)
                .collect();
            let _ = writeln!(summary, "CTEs: {}", ctes.join(", "));
        }
        let tables: Vec<&str> = query
            .tables
            .iter()
            .map(compact_str::CompactString::as_str)
            .collect();
        let _ = writeln!(summary, "Tables: {}", tables.join(", "));
        if !query.where_cols.is_empty() {
            let cols: Vec<&str> = query
                .where_cols
                .iter()
                .map(compact_str::CompactString::as_str)
                .collect();
            let _ = writeln!(summary, "WHERE columns: {}", cols.join(", "));
        }
        if !query.join_cols.is_empty() {
            let cols: Vec<&str> = query
                .join_cols
                .iter()
                .map(compact_str::CompactString::as_str)
                .collect();
            let _ = writeln!(summary, "JOIN columns: {}", cols.join(", "));
        }
        if !query.order_cols.is_empty() {
            let cols: Vec<&str> = query
                .order_cols
                .iter()
                .map(compact_str::CompactString::as_str)
                .collect();
            let _ = writeln!(summary, "ORDER BY columns: {}", cols.join(", "));
        }
        if !query.group_cols.is_empty() {
            let cols: Vec<&str> = query
                .group_cols
                .iter()
                .map(compact_str::CompactString::as_str)
                .collect();
            let _ = writeln!(summary, "GROUP BY columns: {}", cols.join(", "));
        }
        if !query.having_cols.is_empty() {
            let cols: Vec<&str> = query
                .having_cols
                .iter()
                .map(compact_str::CompactString::as_str)
                .collect();
            let _ = writeln!(summary, "HAVING columns: {}", cols.join(", "));
        }
        if !query.window_funcs.is_empty() {
            let funcs: Vec<&str> = query.window_funcs.iter().map(|w| w.name.as_str()).collect();
            let _ = writeln!(summary, "Window functions: {}", funcs.join(", "));
        }
        if let Some(limit) = query.limit {
            let _ = writeln!(summary, "LIMIT: {limit}");
        }
        if let Some(offset) = query.offset {
            let _ = writeln!(summary, "OFFSET: {offset}");
        }
        if query.has_distinct {
            summary.push_str("Has DISTINCT: yes\n");
        }
        if query.has_union {
            summary.push_str("Has UNION/INTERSECT/EXCEPT: yes\n");
        }
        if query.has_subquery {
            summary.push_str("Has subquery: yes\n");
        }
        if opts.verbose {
            write_complexity(&mut summary, query, opts);
        }
        summary.push('\n');
    }
    summary
}

/// Append the colored complexity line for a query to `summary`.
fn write_complexity(summary: &mut String, query: &Query, opts: &OutputOptions) {
    let c = query.complexity();
    let complexity_label = if c.score < 5 {
        if opts.colored {
            "Low".green().to_string()
        } else {
            "Low".to_string()
        }
    } else if c.score < 15 {
        if opts.colored {
            "Medium".yellow().to_string()
        } else {
            "Medium".to_string()
        }
    } else if opts.colored {
        "High".red().to_string()
    } else {
        "High".to_string()
    };
    let _ = writeln!(
        summary,
        "Complexity: {} (score: {})",
        complexity_label, c.score
    );
}
#[must_use]
pub fn format_static_analysis(report: &AnalysisReport, opts: &OutputOptions) -> String {
    match opts.format {
        OutputFormat::Json => serde_json::to_string_pretty(report).unwrap_or_default(),
        OutputFormat::Yaml => yaml::to_string(report).unwrap_or_default(),
        OutputFormat::Text => format_text_analysis(report, opts),
        OutputFormat::Sarif => format_sarif(report)
    }
}

fn format_sarif(report: &AnalysisReport) -> String {
    let results: Vec<serde_json::Value> = report
        .violations
        .iter()
        .map(|v| {
            serde_json::json!({
                "ruleId": v.rule_id,
                "level": match v.severity {
                    Severity::Error => "error",
                    Severity::Warning => "warning",
                    Severity::Info => "note",
                },
                "message": {
                    "text": v.message
                },
                "locations": [{
                    "physicalLocation": {
                        "artifactLocation": {
                            "uri": "queries.sql"
                        },
                        "region": {
                            "startLine": v.query_index + 1
                        }
                    }
                }]
            })
        })
        .collect();
    let sarif = serde_json::json!({
        "$schema": "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/master/Schemata/sarif-schema-2.1.0.json",
        "version": "2.1.0",
        "runs": [{
            "tool": {
                "driver": {
                    "name": "sql-query-analyzer",
                    "version": env!("CARGO_PKG_VERSION"),
                    "informationUri": "https://github.com/example/sql-query-analyzer"
                }
            },
            "results": results
        }]
    });
    serde_json::to_string_pretty(&sarif).unwrap_or_default()
}

fn format_text_analysis(report: &AnalysisReport, opts: &OutputOptions) -> String {
    let mut output = String::new();
    let header = "=== Static Analysis ===\n";
    if opts.colored {
        output.push_str(&header.bold().to_string());
    } else {
        output.push_str(header);
    }
    if report.violations.is_empty() {
        let msg = "✓ No issues found\n";
        if opts.colored {
            output.push_str(&msg.green().to_string());
        } else {
            output.push_str(msg);
        }
        return output;
    }
    let summary = format!(
        "Found {errors} error(s), {warnings} warning(s), {infos} info\n\n",
        errors = report.error_count(),
        warnings = report.warning_count(),
        infos = report.info_count()
    );
    output.push_str(&summary);
    let mut current_query = usize::MAX;
    for violation in &report.violations {
        if violation.query_index != current_query {
            current_query = violation.query_index;
            let query_header = format!("Query #{}:\n", current_query + 1);
            if opts.colored {
                output.push_str(&query_header.cyan().to_string());
            } else {
                output.push_str(&query_header);
            }
        }
        let severity_str = match violation.severity {
            Severity::Error => {
                if opts.colored {
                    "ERROR".red().bold().to_string()
                } else {
                    "ERROR".to_string()
                }
            }
            Severity::Warning => {
                if opts.colored {
                    "WARN".yellow().to_string()
                } else {
                    "WARN".to_string()
                }
            }
            Severity::Info => {
                if opts.colored {
                    "INFO".blue().to_string()
                } else {
                    "INFO".to_string()
                }
            }
        };
        let _ = writeln!(
            output,
            "  [{severity:>5}] {rule_id}: {message}",
            severity = severity_str,
            rule_id = violation.rule_id,
            message = violation.message
        );
        if let Some(suggestion) = &violation.suggestion {
            let suggestion_line = format!("         → {suggestion}\n");
            if opts.colored {
                output.push_str(&suggestion_line.dimmed().to_string());
            } else {
                output.push_str(&suggestion_line);
            }
        }
    }
    output.push('\n');
    output
}
