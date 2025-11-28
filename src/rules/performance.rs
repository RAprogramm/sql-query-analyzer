use super::{Rule, RuleCategory, RuleInfo, Severity, Violation};
use crate::query::{Query, QueryType};

/// Scalar subquery in SELECT (N+1 pattern)
pub struct ScalarSubquery;

impl Rule for ScalarSubquery {
    fn info(&self) -> RuleInfo {
        RuleInfo {
            id:       "PERF007",
            name:     "Scalar subquery in SELECT",
            severity: Severity::Warning,
            category: RuleCategory::Performance
        }
    }

    fn check(&self, query: &Query, query_index: usize) -> Vec<Violation> {
        if query.query_type != QueryType::Select {
            return vec![];
        }
        let upper = query.raw.to_uppercase();
        if let Some(from_pos) = upper.find(" FROM ") {
            let select_part = &upper[..from_pos];
            if select_part.contains("SELECT")
                && select_part.matches('(').count() > 0
                && query.has_subquery
            {
                let info = self.info();
                return vec![Violation {
                    rule_id: info.id,
                    rule_name: info.name,
                    message: "Scalar subquery in SELECT causes N+1 query pattern".to_string(),
                    severity: info.severity,
                    category: info.category,
                    suggestion: Some("Use JOIN or window function instead".to_string()),
                    query_index
                }];
            }
        }
        vec![]
    }
}

/// Function call on column prevents index usage
pub struct FunctionOnColumn;

impl Rule for FunctionOnColumn {
    fn info(&self) -> RuleInfo {
        RuleInfo {
            id:       "PERF008",
            name:     "Function on indexed column",
            severity: Severity::Warning,
            category: RuleCategory::Performance
        }
    }

    fn check(&self, query: &Query, query_index: usize) -> Vec<Violation> {
        let upper = query.raw.to_uppercase();
        let patterns = [
            "WHERE YEAR(",
            "WHERE MONTH(",
            "WHERE DAY(",
            "WHERE DATE(",
            "WHERE UPPER(",
            "WHERE LOWER(",
            "WHERE TRIM(",
            "WHERE SUBSTRING(",
            "WHERE CAST(",
            "WHERE CONVERT(",
            "WHERE COALESCE("
        ];
        for pattern in patterns {
            if upper.contains(pattern) {
                let info = self.info();
                return vec![Violation {
                    rule_id: info.id,
                    rule_name: info.name,
                    message: "Function call on column in WHERE prevents index usage".to_string(),
                    severity: info.severity,
                    category: info.category,
                    suggestion: Some(
                        "Use computed column, functional index, or rewrite condition".to_string()
                    ),
                    query_index
                }];
            }
        }
        vec![]
    }
}

/// NOT IN with subquery can have NULL issues
pub struct NotInWithSubquery;

impl Rule for NotInWithSubquery {
    fn info(&self) -> RuleInfo {
        RuleInfo {
            id:       "PERF009",
            name:     "NOT IN with subquery",
            severity: Severity::Warning,
            category: RuleCategory::Performance
        }
    }

    fn check(&self, query: &Query, query_index: usize) -> Vec<Violation> {
        let upper = query.raw.to_uppercase();
        if upper.contains("NOT IN") && upper.contains("SELECT") {
            let info = self.info();
            return vec![Violation {
                rule_id: info.id,
                rule_name: info.name,
                message: "NOT IN with subquery can return unexpected results with NULL"
                    .to_string(),
                severity: info.severity,
                category: info.category,
                suggestion: Some("Use NOT EXISTS or LEFT JOIN with IS NULL instead".to_string()),
                query_index
            }];
        }
        vec![]
    }
}

/// UNION instead of UNION ALL when duplicates don't matter
pub struct UnionWithoutAll;

impl Rule for UnionWithoutAll {
    fn info(&self) -> RuleInfo {
        RuleInfo {
            id:       "PERF010",
            name:     "UNION without ALL",
            severity: Severity::Info,
            category: RuleCategory::Performance
        }
    }

    fn check(&self, query: &Query, query_index: usize) -> Vec<Violation> {
        if !query.has_union {
            return vec![];
        }
        let upper = query.raw.to_uppercase();
        if upper.contains(" UNION ") && !upper.contains(" UNION ALL ") {
            let info = self.info();
            return vec![Violation {
                rule_id: info.id,
                rule_name: info.name,
                message: "UNION removes duplicates which requires sorting".to_string(),
                severity: info.severity,
                category: info.category,
                suggestion: Some("Use UNION ALL if duplicates are acceptable".to_string()),
                query_index
            }];
        }
        vec![]
    }
}

/// SELECT without WHERE on large table
pub struct SelectWithoutWhere;

impl Rule for SelectWithoutWhere {
    fn info(&self) -> RuleInfo {
        RuleInfo {
            id:       "PERF011",
            name:     "SELECT without WHERE",
            severity: Severity::Info,
            category: RuleCategory::Performance
        }
    }

    fn check(&self, query: &Query, query_index: usize) -> Vec<Violation> {
        if query.query_type != QueryType::Select {
            return vec![];
        }
        if query.where_cols.is_empty() && query.limit.is_none() && !query.tables.is_empty() {
            let info = self.info();
            return vec![Violation {
                rule_id: info.id,
                rule_name: info.name,
                message: "SELECT without WHERE or LIMIT scans entire table".to_string(),
                severity: info.severity,
                category: info.category,
                suggestion: Some("Add WHERE clause or LIMIT to restrict results".to_string()),
                query_index
            }];
        }
        vec![]
    }
}

/// SELECT * without LIMIT can return unbounded results
pub struct SelectStarWithoutLimit;

impl Rule for SelectStarWithoutLimit {
    fn info(&self) -> RuleInfo {
        RuleInfo {
            id:       "PERF001",
            name:     "SELECT * without LIMIT",
            severity: Severity::Warning,
            category: RuleCategory::Performance
        }
    }

    fn check(&self, query: &Query, query_index: usize) -> Vec<Violation> {
        if query.query_type != QueryType::Select {
            return vec![];
        }
        let has_star = query.raw.to_uppercase().contains("SELECT *")
            || query.raw.to_uppercase().contains("SELECT  *");
        if has_star && query.limit.is_none() {
            let info = self.info();
            return vec![Violation {
                rule_id: info.id,
                rule_name: info.name,
                message: "Query uses SELECT * without LIMIT clause".to_string(),
                severity: info.severity,
                category: info.category,
                suggestion: Some("Add LIMIT clause or specify explicit columns".to_string()),
                query_index
            }];
        }
        vec![]
    }
}

/// LIKE patterns starting with % prevent index usage
pub struct LeadingWildcard;

impl Rule for LeadingWildcard {
    fn info(&self) -> RuleInfo {
        RuleInfo {
            id:       "PERF002",
            name:     "Leading wildcard in LIKE",
            severity: Severity::Warning,
            category: RuleCategory::Performance
        }
    }

    fn check(&self, query: &Query, query_index: usize) -> Vec<Violation> {
        let upper = query.raw.to_uppercase();
        if upper.contains("LIKE '%") || upper.contains("LIKE \"%") {
            let info = self.info();
            return vec![Violation {
                rule_id: info.id,
                rule_name: info.name,
                message: "LIKE pattern starts with wildcard, preventing index usage".to_string(),
                severity: info.severity,
                category: info.category,
                suggestion: Some("Consider full-text search or restructure query".to_string()),
                query_index
            }];
        }
        vec![]
    }
}

/// Multiple OR conditions on same column should use IN
pub struct OrInsteadOfIn;

impl Rule for OrInsteadOfIn {
    fn info(&self) -> RuleInfo {
        RuleInfo {
            id:       "PERF003",
            name:     "OR instead of IN",
            severity: Severity::Info,
            category: RuleCategory::Performance
        }
    }

    fn check(&self, query: &Query, query_index: usize) -> Vec<Violation> {
        let upper = query.raw.to_uppercase();
        let or_count = upper.matches(" OR ").count();
        if or_count >= 3 {
            let info = self.info();
            return vec![Violation {
                rule_id: info.id,
                rule_name: info.name,
                message: format!(
                    "Query has {} OR conditions, consider using IN clause",
                    or_count
                ),
                severity: info.severity,
                category: info.category,
                suggestion: Some(
                    "Replace multiple OR conditions with IN (val1, val2, ...)".to_string()
                ),
                query_index
            }];
        }
        vec![]
    }
}

/// Large OFFSET values cause performance issues
pub struct LargeOffset;

impl Rule for LargeOffset {
    fn info(&self) -> RuleInfo {
        RuleInfo {
            id:       "PERF004",
            name:     "Large OFFSET value",
            severity: Severity::Warning,
            category: RuleCategory::Performance
        }
    }

    fn check(&self, query: &Query, query_index: usize) -> Vec<Violation> {
        if let Some(offset) = query.offset
            && offset > 1000
        {
            let info = self.info();
            return vec![Violation {
                rule_id: info.id,
                rule_name: info.name,
                message: format!(
                    "OFFSET {} is large, causing performance degradation",
                    offset
                ),
                severity: info.severity,
                category: info.category,
                suggestion: Some("Use keyset pagination (WHERE id > last_id) instead".to_string()),
                query_index
            }];
        }
        vec![]
    }
}

/// Missing JOIN condition creates Cartesian product
pub struct MissingJoinCondition;

impl Rule for MissingJoinCondition {
    fn info(&self) -> RuleInfo {
        RuleInfo {
            id:       "PERF005",
            name:     "Potential Cartesian product",
            severity: Severity::Error,
            category: RuleCategory::Performance
        }
    }

    fn check(&self, query: &Query, query_index: usize) -> Vec<Violation> {
        if query.query_type != QueryType::Select {
            return vec![];
        }
        let table_count = query.tables.len();
        let has_conditions = !query.join_cols.is_empty() || !query.where_cols.is_empty();
        if table_count > 1 && !has_conditions {
            let info = self.info();
            return vec![Violation {
                rule_id: info.id,
                rule_name: info.name,
                message: format!(
                    "Query references {} tables without apparent JOIN conditions",
                    table_count
                ),
                severity: info.severity,
                category: info.category,
                suggestion: Some(
                    "Add JOIN conditions or WHERE clause to prevent Cartesian product".to_string()
                ),
                query_index
            }];
        }
        vec![]
    }
}

/// DISTINCT with ORDER BY can be inefficient
pub struct DistinctWithOrderBy;

impl Rule for DistinctWithOrderBy {
    fn info(&self) -> RuleInfo {
        RuleInfo {
            id:       "PERF006",
            name:     "DISTINCT with ORDER BY",
            severity: Severity::Info,
            category: RuleCategory::Performance
        }
    }

    fn check(&self, query: &Query, query_index: usize) -> Vec<Violation> {
        if query.has_distinct && !query.order_cols.is_empty() {
            let info = self.info();
            return vec![Violation {
                rule_id: info.id,
                rule_name: info.name,
                message: "Query uses DISTINCT with ORDER BY".to_string(),
                severity: info.severity,
                category: info.category,
                suggestion: Some(
                    "Consider if both are necessary, or use GROUP BY instead".to_string()
                ),
                query_index
            }];
        }
        vec![]
    }
}
