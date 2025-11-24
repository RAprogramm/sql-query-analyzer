//! Type definitions for the static analysis rule system.
//!
//! This module defines the core types used throughout the rule engine:
//! - [`Severity`] - Violation severity levels (Info, Warning, Error)
//! - [`RuleCategory`] - Rule categories (Performance, Style, Security)
//! - [`Violation`] - Individual rule violations with context
//! - [`AnalysisReport`] - Complete analysis results

use serde::Serialize;

/// Severity level of a rule violation.
///
/// Ordered from lowest to highest severity for sorting purposes.
/// Exit codes are determined by the highest severity violation found.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub enum Severity {
    /// Informational suggestion, does not affect exit code
    Info,
    /// Warning that may indicate a problem (exit code 1)
    Warning,
    /// Critical issue that must be addressed (exit code 2)
    Error
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Info => write!(f, "INFO"),
            Self::Warning => write!(f, "WARN"),
            Self::Error => write!(f, "ERROR")
        }
    }
}

/// Category of a rule for grouping and filtering.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum RuleCategory {
    /// Rules that detect potential performance issues
    Performance,
    /// Rules that enforce coding style and best practices
    Style,
    /// Rules that identify potential security vulnerabilities
    Security
}

impl std::fmt::Display for RuleCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Performance => write!(f, "Performance"),
            Self::Style => write!(f, "Style"),
            Self::Security => write!(f, "Security")
        }
    }
}

/// A single rule violation found in a query.
///
/// Contains all context needed to display and filter the violation,
/// including the originating rule, severity, and optional fix suggestion.
#[derive(Debug, Clone, Serialize)]
pub struct Violation {
    /// Unique rule identifier (e.g., "PERF001", "SEC001")
    pub rule_id:     &'static str,
    /// Human-readable rule name
    pub rule_name:   &'static str,
    /// Detailed description of the violation
    pub message:     String,
    /// Severity level of this violation
    pub severity:    Severity,
    /// Category for grouping violations
    pub category:    RuleCategory,
    /// Optional suggestion for fixing the issue
    pub suggestion:  Option<String>,
    /// Zero-based index of the query in the input
    pub query_index: usize
}

/// Metadata about a rule for identification and configuration.
#[derive(Debug, Clone)]
pub struct RuleInfo {
    /// Unique rule identifier (e.g., "PERF001")
    pub id:       &'static str,
    /// Human-readable rule name
    pub name:     &'static str,
    /// Default severity level
    pub severity: Severity,
    /// Rule category
    pub category: RuleCategory
}

/// Complete analysis report containing all violations.
///
/// Use [`error_count`](Self::error_count),
/// [`warning_count`](Self::warning_count), and [`info_count`](Self::info_count)
/// to get violation counts by severity.
#[derive(Debug, Clone, Serialize)]
pub struct AnalysisReport {
    /// All violations found during analysis
    pub violations:    Vec<Violation>,
    /// Number of queries analyzed
    pub queries_count: usize,
    /// Number of rules executed
    pub rules_count:   usize
}

impl AnalysisReport {
    pub fn new(queries_count: usize, rules_count: usize) -> Self {
        Self {
            violations: Vec::new(),
            queries_count,
            rules_count
        }
    }

    pub fn add_violation(&mut self, violation: Violation) {
        self.violations.push(violation);
    }

    pub fn error_count(&self) -> usize {
        self.violations
            .iter()
            .filter(|v| v.severity == Severity::Error)
            .count()
    }

    pub fn warning_count(&self) -> usize {
        self.violations
            .iter()
            .filter(|v| v.severity == Severity::Warning)
            .count()
    }

    pub fn info_count(&self) -> usize {
        self.violations
            .iter()
            .filter(|v| v.severity == Severity::Info)
            .count()
    }
}
