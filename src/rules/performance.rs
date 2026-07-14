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

/// ORDER BY RAND() forces a full scan and sort of every candidate row
///
/// The database must generate a random value per row and sort the whole
/// result set before applying LIMIT, so cost stays O(n log n) regardless
/// of how few rows are returned. Detects the MySQL, PostgreSQL, SQL Server,
/// and Oracle spellings.
pub struct OrderByRandom;

impl Rule for OrderByRandom {
    fn info(&self) -> RuleInfo {
        RuleInfo {
            id:       "PERF013",
            name:     "ORDER BY RAND() detected",
            severity: Severity::Warning,
            category: RuleCategory::Performance
        }
    }

    fn check(&self, query: &Query, query_index: usize) -> Vec<Violation> {
        if query.query_type != QueryType::Select {
            return vec![];
        }
        let upper = query.raw.to_uppercase();
        let Some(order_pos) = upper.find("ORDER BY") else {
            return vec![];
        };
        let order_part = &upper[order_pos..];
        let random_funcs = ["RAND()", "RANDOM()", "NEWID()", "DBMS_RANDOM"];
        if random_funcs.iter().any(|f| order_part.contains(f)) {
            let info = self.info();
            return vec![Violation {
                rule_id: info.id,
                rule_name: info.name,
                message: "ORDER BY RAND() scans and sorts every row before applying LIMIT"
                    .to_string(),
                severity: info.severity,
                category: info.category,
                suggestion: Some(
                    "For random selection use a random id range (WHERE id >= FLOOR(RAND() * max_id)) or a pre-generated indexed random column"
                        .to_string()
                ),
                query_index
            }];
        }
        vec![]
    }
}

/// COUNT(*) without WHERE scans the whole table
///
/// Counting every row cannot use an index shortcut on most engines; the
/// query time grows linearly with table size and can block writes on busy
/// tables. Existence checks and cached or estimated counts are cheaper.
pub struct CountWithoutWhere;

impl Rule for CountWithoutWhere {
    fn info(&self) -> RuleInfo {
        RuleInfo {
            id:       "PERF012",
            name:     "COUNT(*) without WHERE",
            severity: Severity::Warning,
            category: RuleCategory::Performance
        }
    }

    fn check(&self, query: &Query, query_index: usize) -> Vec<Violation> {
        if query.query_type != QueryType::Select {
            return vec![];
        }
        if !query.where_cols.is_empty() || query.tables.is_empty() {
            return vec![];
        }
        let upper = query.raw.to_uppercase();
        if !upper.contains("COUNT(") {
            return vec![];
        }
        let info = self.info();
        vec![Violation {
            rule_id: info.id,
            rule_name: info.name,
            message: "COUNT without WHERE clause scans the entire table".to_string(),
            severity: info.severity,
            category: info.category,
            suggestion: Some(
                "Add a WHERE clause, use EXISTS for existence checks, or cache/estimate counts for large tables"
                    .to_string()
            ),
            query_index
        }]
    }
}

/// Large IN value lists degrade planning and execution
///
/// Very long IN lists blow up parse and plan time, defeat plan caching, and
/// on some engines hit hard parameter limits. Severity scales with size:
/// more than 50 items is Info, more than 200 Warning, more than 1000 Error.
pub struct LargeInClause;

/// Counts top-level comma-separated items in an IN list body, returning
/// None when the list is a subquery rather than a value list.
fn in_list_item_count(body: &str) -> Option<usize> {
    if body.trim_start().starts_with("SELECT") {
        return None;
    }
    let mut depth = 0usize;
    let mut items = 1usize;
    for b in body.bytes() {
        match b {
            b'(' => depth += 1,
            b')' => {
                if depth == 0 {
                    break;
                }
                depth -= 1;
            }
            b',' if depth == 0 => items += 1,
            _ => {}
        }
    }
    Some(items)
}

/// Returns the largest IN value-list size found in the statement.
fn max_in_list_size(upper: &str) -> usize {
    let mut max_items = 0;
    let mut search_from = 0;
    while let Some(pos) = upper[search_from..].find(" IN (") {
        let body_start = search_from + pos + " IN (".len();
        if let Some(items) = in_list_item_count(&upper[body_start..]) {
            max_items = max_items.max(items);
        }
        search_from = body_start;
    }
    max_items
}

impl Rule for LargeInClause {
    fn info(&self) -> RuleInfo {
        RuleInfo {
            id:       "PERF019",
            name:     "Large IN clause",
            severity: Severity::Warning,
            category: RuleCategory::Performance
        }
    }

    fn check(&self, query: &Query, query_index: usize) -> Vec<Violation> {
        let upper = query.raw.to_uppercase();
        let items = max_in_list_size(&upper);
        if items <= 50 {
            return vec![];
        }
        let severity = if items > 1000 {
            Severity::Error
        } else if items > 200 {
            Severity::Warning
        } else {
            Severity::Info
        };
        let info = self.info();
        vec![Violation {
            rule_id: info.id,
            rule_name: info.name,
            message: format!("IN clause contains {} values", items),
            severity,
            category: info.category,
            suggestion: Some(
                "Load the values into a temporary table and JOIN against it, or split the query into batches"
                    .to_string()
            ),
            query_index
        }]
    }
}

/// HAVING without an aggregate belongs in WHERE
///
/// HAVING filters after grouping, so a condition on plain columns forces the
/// engine to group rows it could have discarded up front. Moving the
/// condition into WHERE prunes rows before the GROUP BY.
pub struct HavingWithoutAggregate;

/// Aggregate function openers that justify a HAVING clause.
const AGGREGATE_OPENERS: [&str; 9] = [
    "COUNT(",
    "SUM(",
    "AVG(",
    "MIN(",
    "MAX(",
    "GROUP_CONCAT(",
    "STRING_AGG(",
    "STDDEV",
    "VARIANCE"
];

impl Rule for HavingWithoutAggregate {
    fn info(&self) -> RuleInfo {
        RuleInfo {
            id:       "PERF018",
            name:     "HAVING without aggregate function",
            severity: Severity::Warning,
            category: RuleCategory::Performance
        }
    }

    fn check(&self, query: &Query, query_index: usize) -> Vec<Violation> {
        if query.query_type != QueryType::Select || query.having_cols.is_empty() {
            return vec![];
        }
        let upper = query.raw.to_uppercase();
        let Some(having_pos) = upper.find(" HAVING ") else {
            return vec![];
        };
        let having_part = &upper[having_pos + " HAVING ".len()..];
        let clause_end = [" ORDER BY ", " LIMIT ", " OFFSET "]
            .iter()
            .filter_map(|t| having_part.find(t))
            .min()
            .unwrap_or(having_part.len());
        let clause = &having_part[..clause_end];
        if AGGREGATE_OPENERS.iter().any(|agg| clause.contains(agg)) {
            return vec![];
        }
        let info = self.info();
        vec![Violation {
            rule_id: info.id,
            rule_name: info.name,
            message: "HAVING filters plain columns after grouping".to_string(),
            severity: info.severity,
            category: info.category,
            suggestion: Some(
                "Move non-aggregate conditions into WHERE so rows are pruned before GROUP BY"
                    .to_string()
            ),
            query_index
        }]
    }
}

/// DISTINCT that likely papers over a join fan-out
///
/// DISTINCT combined with JOIN usually hides duplicate rows produced by a
/// missing or too-loose join condition; deduplication then costs a sort or
/// hash over the whole result. `SELECT DISTINCT *` escalates to Warning —
/// deduplicating entire rows is almost never intended.
pub struct UnnecessaryDistinct;

impl Rule for UnnecessaryDistinct {
    fn info(&self) -> RuleInfo {
        RuleInfo {
            id:       "PERF014",
            name:     "Potentially unnecessary DISTINCT",
            severity: Severity::Info,
            category: RuleCategory::Performance
        }
    }

    fn check(&self, query: &Query, query_index: usize) -> Vec<Violation> {
        if query.query_type != QueryType::Select || !query.has_distinct {
            return vec![];
        }
        let upper = query.raw.to_uppercase();
        let distinct_star = upper.contains("SELECT DISTINCT *");
        let distinct_with_join = query.tables.len() > 1;
        if !distinct_star && !distinct_with_join {
            return vec![];
        }
        let info = self.info();
        let (severity, message) = if distinct_star {
            (
                Severity::Warning,
                "SELECT DISTINCT * deduplicates entire rows".to_string()
            )
        } else {
            (
                info.severity,
                "DISTINCT combined with JOIN often hides join fan-out".to_string()
            )
        };
        vec![Violation {
            rule_id: info.id,
            rule_name: info.name,
            message,
            severity,
            category: info.category,
            suggestion: Some(
                "Check the join conditions for fan-out before deduplicating; select explicit columns instead of DISTINCT *"
                    .to_string()
            ),
            query_index
        }]
    }
}

/// Deeply nested subqueries defeat optimizers and readers alike
///
/// Each nesting level multiplies planning complexity and usually hides a
/// JOIN or CTE that would express the same logic flatter and faster.
/// Severity scales with total SELECT depth: three levels is Info, four
/// Warning, five or more Error.
pub struct DeeplyNestedSubqueries;

/// Returns the deepest count of parenthesized SELECTs enclosing each other.
fn max_subquery_depth(upper: &str) -> usize {
    let bytes = upper.as_bytes();
    let mut select_stack: Vec<usize> = Vec::new();
    let mut paren_depth = 0usize;
    let mut max_depth = 0usize;
    for (i, b) in bytes.iter().enumerate() {
        match b {
            b'(' => {
                paren_depth += 1;
                if upper[i + 1..].trim_start().starts_with("SELECT") {
                    select_stack.push(paren_depth);
                    max_depth = max_depth.max(select_stack.len());
                }
            }
            b')' => {
                if select_stack.last() == Some(&paren_depth) {
                    select_stack.pop();
                }
                paren_depth = paren_depth.saturating_sub(1);
            }
            _ => {}
        }
    }
    max_depth
}

impl Rule for DeeplyNestedSubqueries {
    fn info(&self) -> RuleInfo {
        RuleInfo {
            id:       "PERF020",
            name:     "Deeply nested subqueries",
            severity: Severity::Warning,
            category: RuleCategory::Performance
        }
    }

    fn check(&self, query: &Query, query_index: usize) -> Vec<Violation> {
        if query.query_type != QueryType::Select {
            return vec![];
        }
        let upper = query.raw.to_uppercase();
        let levels = max_subquery_depth(&upper) + 1;
        if levels < 3 {
            return vec![];
        }
        let severity = if levels >= 5 {
            Severity::Error
        } else if levels >= 4 {
            Severity::Warning
        } else {
            Severity::Info
        };
        let info = self.info();
        vec![Violation {
            rule_id: info.id,
            rule_name: info.name,
            message: format!("Query nests SELECTs {} levels deep", levels),
            severity,
            category: info.category,
            suggestion: Some(
                "Flatten nested subqueries into JOINs or name the steps with CTEs (WITH ...)"
                    .to_string()
            ),
            query_index
        }]
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
