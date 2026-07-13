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

/// Ordinal column references in ORDER BY / GROUP BY
///
/// `ORDER BY 1, 2` sorts by SELECT-list position, so adding or reordering
/// selected columns silently changes the sort with no error. Explicit column
/// names keep the intent stable and readable.
pub struct OrdinalInOrderOrGroupBy;

/// Returns true when any top-level, comma-separated item of the clause
/// segment starts with a bare integer (an ordinal reference). Commas inside
/// parentheses are ignored so function arguments never count as items.
fn clause_has_ordinal(segment: &str) -> bool {
    let mut depth: i32 = 0;
    let mut item_start = 0;
    let mut items = Vec::new();
    for (i, b) in segment.bytes().enumerate() {
        match b {
            b'(' => depth += 1,
            b')' => depth -= 1,
            b',' if depth == 0 => {
                items.push(&segment[item_start..i]);
                item_start = i + 1;
            }
            _ => {}
        }
    }
    items.push(&segment[item_start..]);
    items.iter().any(|item| {
        item.split_whitespace()
            .next()
            .is_some_and(|tok| tok.chars().all(|c| c.is_ascii_digit()))
    })
}

/// Extracts the clause body following `keyword`, stopping at the next
/// clause boundary so LIMIT/OFFSET counts are never mistaken for ordinals.
fn clause_segment<'a>(upper: &'a str, keyword: &str) -> Option<&'a str> {
    let start = upper.find(keyword)? + keyword.len();
    let rest = &upper[start..];
    let terminators = [
        " LIMIT ",
        " OFFSET ",
        " HAVING ",
        " ORDER BY ",
        " GROUP BY "
    ];
    let end = terminators
        .iter()
        .filter_map(|t| rest.find(t))
        .min()
        .unwrap_or(rest.len());
    Some(&rest[..end])
}

impl Rule for OrdinalInOrderOrGroupBy {
    fn info(&self) -> RuleInfo {
        RuleInfo {
            id:       "STYLE004",
            name:     "Ordinal in ORDER BY/GROUP BY",
            severity: Severity::Info,
            category: RuleCategory::Style
        }
    }

    fn check(&self, query: &Query, query_index: usize) -> Vec<Violation> {
        if query.query_type != QueryType::Select {
            return vec![];
        }
        let upper = query.raw.to_uppercase();
        let flagged: Vec<&str> = ["ORDER BY", "GROUP BY"]
            .into_iter()
            .filter(|kw| clause_segment(&upper, kw).is_some_and(clause_has_ordinal))
            .collect();
        if flagged.is_empty() {
            return vec![];
        }
        let info = self.info();
        vec![Violation {
            rule_id: info.id,
            rule_name: info.name,
            message: format!(
                "{} uses a column ordinal instead of a column name",
                flagged.join(" and ")
            ),
            severity: info.severity,
            category: info.category,
            suggestion: Some(
                "Ordinals silently break when the SELECT list changes; use explicit column names"
                    .to_string()
            ),
            query_index
        }]
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
