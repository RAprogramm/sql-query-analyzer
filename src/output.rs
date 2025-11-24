use colored::Colorize;
use serde::Serialize;

use crate::query::Query;

/// Output format for results
#[derive(Debug, Clone, Copy, Default)]
pub enum OutputFormat {
    #[default]
    Text,
    Json,
    Yaml
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
        OutputFormat::Json => serde_json::to_string_pretty(queries).unwrap_or_default(),
        OutputFormat::Yaml => serde_yaml::to_string(queries).unwrap_or_default(),
        OutputFormat::Text => format_text_summary(queries, opts)
    }
}

/// Format full analysis result
pub fn format_analysis_result(queries: &[Query], analysis: &str, opts: &OutputOptions) -> String {
    match opts.format {
        OutputFormat::Json => {
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
            serde_yaml::to_string(&result).unwrap_or_default()
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
        let header = format!("Query #{} ({}):", i + 1, query.query_type);
        if opts.colored {
            summary.push_str(&header.cyan().bold().to_string());
        } else {
            summary.push_str(&header);
        }
        summary.push('\n');
        summary.push_str(&format!("{}\n", query.raw));

        if !query.cte_names.is_empty() {
            let ctes: Vec<&str> = query.cte_names.iter().map(|s| s.as_str()).collect();
            summary.push_str(&format!("CTEs: {}\n", ctes.join(", ")));
        }

        let tables: Vec<&str> = query.tables.iter().map(|s| s.as_str()).collect();
        summary.push_str(&format!("Tables: {}\n", tables.join(", ")));

        if !query.where_cols.is_empty() {
            let cols: Vec<&str> = query.where_cols.iter().map(|s| s.as_str()).collect();
            summary.push_str(&format!("WHERE columns: {}\n", cols.join(", ")));
        }
        if !query.join_cols.is_empty() {
            let cols: Vec<&str> = query.join_cols.iter().map(|s| s.as_str()).collect();
            summary.push_str(&format!("JOIN columns: {}\n", cols.join(", ")));
        }
        if !query.order_cols.is_empty() {
            let cols: Vec<&str> = query.order_cols.iter().map(|s| s.as_str()).collect();
            summary.push_str(&format!("ORDER BY columns: {}\n", cols.join(", ")));
        }
        if !query.group_cols.is_empty() {
            let cols: Vec<&str> = query.group_cols.iter().map(|s| s.as_str()).collect();
            summary.push_str(&format!("GROUP BY columns: {}\n", cols.join(", ")));
        }
        if !query.having_cols.is_empty() {
            let cols: Vec<&str> = query.having_cols.iter().map(|s| s.as_str()).collect();
            summary.push_str(&format!("HAVING columns: {}\n", cols.join(", ")));
        }

        if !query.window_funcs.is_empty() {
            let funcs: Vec<&str> = query.window_funcs.iter().map(|w| w.name.as_str()).collect();
            summary.push_str(&format!("Window functions: {}\n", funcs.join(", ")));
        }

        if let Some(limit) = query.limit {
            summary.push_str(&format!("LIMIT: {}\n", limit));
        }
        if let Some(offset) = query.offset {
            summary.push_str(&format!("OFFSET: {}\n", offset));
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

        // Complexity info
        if opts.verbose {
            let c = query.complexity();
            let complexity_label = if c.score < 5 {
                if opts.colored { "Low".green().to_string() } else { "Low".to_string() }
            } else if c.score < 15 {
                if opts.colored { "Medium".yellow().to_string() } else { "Medium".to_string() }
            } else if opts.colored {
                "High".red().to_string()
            } else {
                "High".to_string()
            };
            summary.push_str(&format!("Complexity: {} (score: {})\n", complexity_label, c.score));
        }

        summary.push('\n');
    }

    summary
}
