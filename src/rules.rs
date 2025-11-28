//! Static analysis rule engine for SQL queries.
//!
//! This module provides a parallel rule execution engine that analyzes SQL
//! queries for performance issues, style violations, and security
//! vulnerabilities. Rules are implemented as types that implement the [`Rule`]
//! trait.
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────┐     ┌──────────────┐     ┌─────────────┐
//! │  Queries    │────▶│  RuleRunner  │────▶│   Report    │
//! └─────────────┘     └──────────────┘     └─────────────┘
//!                            │
//!                     ┌──────┴──────┐
//!                     │   Rules     │
//!                     │  (parallel) │
//!                     └─────────────┘
//! ```
//!
//! The [`RuleRunner`] executes all enabled rules in parallel using [`rayon`],
//! collecting violations into an [`AnalysisReport`].
//!
//! # Rule Categories
//!
//! - **Performance** (`PERF001`-`PERF011`) - Query optimization issues
//! - **Style** (`STYLE001`-`STYLE002`) - Best practice violations
//! - **Security** (`SEC001`-`SEC003`) - Dangerous operations
//! - **Schema** (`SCHEMA001`-`SCHEMA003`) - Schema validation (requires schema)
//!
//! # Configuration
//!
//! Rules can be disabled or have their severity modified via [`RulesConfig`]:
//!
//! ```toml
//! [rules]
//! disabled = ["STYLE001"]
//!
//! [rules.severity]
//! PERF001 = "error"
//! ```
//!
//! # Implementing Custom Rules
//!
//! ```
//! use sql_query_analyzer::{
//!     query::Query,
//!     rules::{Rule, RuleCategory, RuleInfo, Severity, Violation}
//! };
//!
//! pub struct MyRule;
//!
//! impl Rule for MyRule {
//!     fn info(&self) -> RuleInfo {
//!         RuleInfo {
//!             id:       "CUSTOM001",
//!             name:     "My custom rule",
//!             severity: Severity::Warning,
//!             category: RuleCategory::Performance
//!         }
//!     }
//!
//!     fn check(&self, query: &Query, query_index: usize) -> Vec<Violation> {
//!         vec![]
//!     }
//! }
//! ```

mod performance;
pub mod schema_aware;
mod security;
mod style;
mod types;

use rayon::prelude::*;
pub use types::{AnalysisReport, RuleCategory, RuleInfo, Severity, Violation};

use crate::{config::RulesConfig, query::Query, schema::Schema};

/// Trait for implementing SQL analysis rules.
///
/// Rules are stateless analyzers that examine a single query and return
/// any violations found. They must be `Send + Sync` for parallel execution.
///
/// # Example
///
/// ```
/// use sql_query_analyzer::{
///     query::Query,
///     rules::{Rule, RuleCategory, RuleInfo, Severity, Violation}
/// };
///
/// struct LargeOffsetRule;
///
/// impl Rule for LargeOffsetRule {
///     fn info(&self) -> RuleInfo {
///         RuleInfo {
///             id:       "PERF004",
///             name:     "Large offset",
///             severity: Severity::Warning,
///             category: RuleCategory::Performance
///         }
///     }
///
///     fn check(&self, query: &Query, query_index: usize) -> Vec<Violation> {
///         if query.offset.unwrap_or(0) > 1000 {
///             vec![Violation {
///                 rule_id: "PERF004",
///                 rule_name: "Large offset",
///                 message: "Large OFFSET can cause performance issues".into(),
///                 severity: Severity::Warning,
///                 category: RuleCategory::Performance,
///                 suggestion: Some("Use keyset pagination instead".into()),
///                 query_index
///             }]
///         } else {
///             vec![]
///         }
///     }
/// }
/// ```
pub trait Rule: Send + Sync {
    /// Returns metadata about this rule.
    fn info(&self) -> RuleInfo;

    /// Analyzes a query and returns any violations found.
    ///
    /// # Arguments
    ///
    /// * `query` - The parsed query to analyze
    /// * `query_index` - Zero-based index of this query in the input
    ///
    /// # Returns
    ///
    /// A vector of violations, empty if the query passes this rule.
    fn check(&self, query: &Query, query_index: usize) -> Vec<Violation>;
}

/// Parallel rule execution engine.
///
/// The runner holds a collection of rules and executes them in parallel
/// against each query using [`rayon`]. It supports rule filtering via
/// configuration and severity overrides.
///
/// # Example
///
/// ```
/// use sql_query_analyzer::{
///     config::RulesConfig,
///     query::{SqlDialect, parse_queries},
///     rules::RuleRunner
/// };
///
/// let config = RulesConfig {
///     disabled: vec!["STYLE001".into()],
///     ..Default::default()
/// };
///
/// let runner = RuleRunner::with_config(config);
/// let queries = parse_queries("SELECT id FROM users", SqlDialect::Generic).unwrap();
/// let report = runner.analyze(&queries);
///
/// assert_eq!(report.error_count(), 0);
/// ```
pub struct RuleRunner {
    rules:          Vec<Box<dyn Rule>>,
    severity_cache: std::collections::HashMap<&'static str, Severity>
}

impl Default for RuleRunner {
    fn default() -> Self {
        Self::new()
    }
}

impl RuleRunner {
    /// Create a new runner with all default rules
    pub fn new() -> Self {
        Self::with_config(RulesConfig::default())
    }

    /// Create a new runner with configuration
    ///
    /// # Notes
    ///
    /// - Performance rules (PERF001-PERF011) detect query optimization issues
    /// - Style rules (STYLE001-STYLE002) enforce best practices
    /// - Security rules (SEC001-SEC003) detect dangerous operations
    pub fn with_config(config: RulesConfig) -> Self {
        let all_rules: Vec<Box<dyn Rule>> = vec![
            Box::new(performance::SelectStarWithoutLimit),
            Box::new(performance::LeadingWildcard),
            Box::new(performance::OrInsteadOfIn),
            Box::new(performance::LargeOffset),
            Box::new(performance::MissingJoinCondition),
            Box::new(performance::DistinctWithOrderBy),
            Box::new(performance::ScalarSubquery),
            Box::new(performance::FunctionOnColumn),
            Box::new(performance::NotInWithSubquery),
            Box::new(performance::UnionWithoutAll),
            Box::new(performance::SelectWithoutWhere),
            Box::new(style::SelectStar),
            Box::new(style::MissingTableAlias),
            Box::new(security::MissingWhereInUpdate),
            Box::new(security::MissingWhereInDelete),
            Box::new(security::TruncateDetected),
        ];
        let rules: Vec<Box<dyn Rule>> = all_rules
            .into_iter()
            .filter(|r| {
                !config
                    .disabled
                    .iter()
                    .any(|d| d.eq_ignore_ascii_case(r.info().id))
            })
            .collect();
        let mut severity_cache = std::collections::HashMap::new();
        for rule in &rules {
            let rule_id = rule.info().id;
            if let Some(sev_str) = config.severity.get(rule_id)
                && let Some(sev) = parse_severity(sev_str)
            {
                severity_cache.insert(rule_id, sev);
            }
        }
        Self {
            rules,
            severity_cache
        }
    }

    /// Create runner with schema-aware rules and configuration
    ///
    /// # Notes
    ///
    /// - Adds schema-aware rules (SCHEMA001-SCHEMA003) if not disabled
    /// - Updates severity cache for schema rules
    pub fn with_schema_and_config(schema: Schema, config: RulesConfig) -> Self {
        let mut runner = Self::with_config(config.clone());
        let schema_rules: Vec<Box<dyn Rule>> = vec![
            Box::new(schema_aware::MissingIndexOnFilterColumn::new(
                schema.clone()
            )),
            Box::new(schema_aware::ColumnNotInSchema::new(schema.clone())),
            Box::new(schema_aware::SuggestIndex::new(schema)),
        ];
        for rule in schema_rules {
            if !config
                .disabled
                .iter()
                .any(|d| d.eq_ignore_ascii_case(rule.info().id))
            {
                let rule_id = rule.info().id;
                if let Some(sev_str) = config.severity.get(rule_id)
                    && let Some(sev) = parse_severity(sev_str)
                {
                    runner.severity_cache.insert(rule_id, sev);
                }
                runner.rules.push(rule);
            }
        }
        runner
    }

    /// Run all rules on the provided queries (parallel execution)
    pub fn analyze(&self, queries: &[Query]) -> AnalysisReport {
        let mut report = AnalysisReport::new(queries.len(), self.rules.len());
        let violations: Vec<Violation> = queries
            .par_iter()
            .enumerate()
            .flat_map(|(idx, query)| {
                self.rules
                    .par_iter()
                    .flat_map(|rule| rule.check(query, idx))
                    .collect::<Vec<_>>()
            })
            .collect();
        for mut violation in violations {
            if let Some(&severity) = self.severity_cache.get(violation.rule_id) {
                violation.severity = severity;
            }
            report.add_violation(violation);
        }
        report.violations.sort_by(|a, b| {
            b.severity
                .cmp(&a.severity)
                .then_with(|| a.query_index.cmp(&b.query_index))
        });
        report
    }
}

/// Parse severity string to enum
fn parse_severity(s: &str) -> Option<Severity> {
    match s.to_lowercase().as_str() {
        "error" => Some(Severity::Error),
        "warning" | "warn" => Some(Severity::Warning),
        "info" => Some(Severity::Info),
        _ => None
    }
}
