use super::{Rule, RuleCategory, RuleInfo, Severity, Violation};
use crate::query::{Query, QueryType};

/// SELECT * is considered bad practice
pub struct SelectStar;

impl Rule for SelectStar {
    fn info(&self) -> RuleInfo {
        RuleInfo {
            id:       "STYLE001",
            name:     "SELECT * usage",
            severity: Severity::Info,
            category: RuleCategory::Style
        }
    }

    fn check(&self, query: &Query, query_index: usize) -> Vec<Violation> {
        if query.query_type != QueryType::Select {
            return vec![];
        }
        let has_star = query.raw.to_uppercase().contains("SELECT *")
            || query.raw.to_uppercase().contains("SELECT  *");
        if has_star {
            let info = self.info();
            return vec![Violation {
                rule_id: info.id,
                rule_name: info.name,
                message: "Query uses SELECT * instead of explicit column list".to_string(),
                severity: info.severity,
                category: info.category,
                suggestion: Some(
                    "Specify explicit columns to improve clarity and performance".to_string()
                ),
                query_index
            }];
        }
        vec![]
    }
}

/// Tables without aliases in JOINs
pub struct MissingTableAlias;

impl Rule for MissingTableAlias {
    fn info(&self) -> RuleInfo {
        RuleInfo {
            id:       "STYLE002",
            name:     "Missing table aliases",
            severity: Severity::Info,
            category: RuleCategory::Style
        }
    }

    fn check(&self, query: &Query, query_index: usize) -> Vec<Violation> {
        if query.query_type != QueryType::Select {
            return vec![];
        }
        if query.tables.len() <= 1 {
            return vec![];
        }
        let upper = query.raw.to_uppercase();
        let has_aliases = upper.contains(" AS ") || query.tables.iter().any(|t| t.contains(' '));
        if !has_aliases && !query.join_cols.is_empty() {
            let info = self.info();
            return vec![Violation {
                rule_id: info.id,
                rule_name: info.name,
                message: "Multi-table query without table aliases".to_string(),
                severity: info.severity,
                category: info.category,
                suggestion: Some(
                    "Add short aliases (e.g., users u, orders o) for readability".to_string()
                ),
                query_index
            }];
        }
        vec![]
    }
}
