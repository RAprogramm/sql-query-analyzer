use super::{Rule, RuleCategory, RuleInfo, Severity, Violation};
use crate::{
    query::{Query, QueryType},
    schema::Schema
};

/// Check if WHERE/JOIN columns have indexes
pub struct MissingIndexOnFilterColumn {
    schema: Schema
}

impl MissingIndexOnFilterColumn {
    pub fn new(schema: Schema) -> Self {
        Self {
            schema
        }
    }

    fn get_indexed_columns(&self) -> Vec<String> {
        self.schema
            .tables
            .values()
            .flat_map(|t| t.indexes.iter().flat_map(|idx| idx.columns.clone()))
            .collect()
    }
}

impl Rule for MissingIndexOnFilterColumn {
    fn info(&self) -> RuleInfo {
        RuleInfo {
            id:       "SCHEMA001",
            name:     "Missing index on filter column",
            severity: Severity::Warning,
            category: RuleCategory::Performance
        }
    }

    fn check(&self, query: &Query, query_index: usize) -> Vec<Violation> {
        if query.query_type != QueryType::Select {
            return vec![];
        }
        let indexed_cols = self.get_indexed_columns();
        let mut violations = Vec::new();
        for col in &query.where_cols {
            let col_lower = col.to_lowercase();
            if !indexed_cols.iter().any(|c| c.to_lowercase() == col_lower) {
                let info = self.info();
                violations.push(Violation {
                    rule_id: info.id,
                    rule_name: info.name,
                    message: format!("Column '{}' in WHERE clause has no index", col),
                    severity: info.severity,
                    category: info.category,
                    suggestion: Some(format!("Consider adding index on '{}'", col)),
                    query_index
                });
            }
        }
        for col in &query.join_cols {
            let col_lower = col.to_lowercase();
            if !indexed_cols.iter().any(|c| c.to_lowercase() == col_lower) {
                let info = self.info();
                violations.push(Violation {
                    rule_id: info.id,
                    rule_name: info.name,
                    message: format!("Column '{}' in JOIN clause has no index", col),
                    severity: info.severity,
                    category: info.category,
                    suggestion: Some(format!("Consider adding index on '{}'", col)),
                    query_index
                });
            }
        }
        violations
    }
}

/// Check if columns exist in schema
pub struct ColumnNotInSchema {
    schema: Schema
}

impl ColumnNotInSchema {
    pub fn new(schema: Schema) -> Self {
        Self {
            schema
        }
    }

    fn get_all_columns(&self) -> Vec<String> {
        self.schema
            .tables
            .values()
            .flat_map(|t| t.columns.iter().map(|c| c.name.clone()))
            .collect()
    }
}

impl Rule for ColumnNotInSchema {
    fn info(&self) -> RuleInfo {
        RuleInfo {
            id:       "SCHEMA002",
            name:     "Column not in schema",
            severity: Severity::Warning,
            category: RuleCategory::Style
        }
    }

    fn check(&self, query: &Query, query_index: usize) -> Vec<Violation> {
        let all_cols = self.get_all_columns();
        let mut violations = Vec::new();
        let query_cols: Vec<&str> = query
            .where_cols
            .iter()
            .chain(query.join_cols.iter())
            .chain(query.order_cols.iter())
            .chain(query.group_cols.iter())
            .map(|s| s.as_str())
            .collect();
        for col in query_cols {
            let col_lower = col.to_lowercase();
            if col_lower.chars().all(|c| c.is_numeric() || c == '.') {
                continue;
            }
            if !all_cols.iter().any(|c| c.to_lowercase() == col_lower) {
                let info = self.info();
                violations.push(Violation {
                    rule_id: info.id,
                    rule_name: info.name,
                    message: format!("Column '{}' not found in schema", col),
                    severity: info.severity,
                    category: info.category,
                    suggestion: Some("Check column name spelling or table reference".to_string()),
                    query_index
                });
            }
        }
        violations
    }
}

/// JOIN columns must lead an index of their own table
///
/// SCHEMA001 only asks whether a column name is indexed anywhere in the
/// schema; a join still degrades to a per-row scan when the joined table
/// itself lacks an index that starts with the join column. This rule checks
/// the joined tables precisely: the column must exist in the table and be
/// the leading column of one of that table's indexes.
pub struct JoinOnNonIndexedColumn {
    schema: Schema
}

impl JoinOnNonIndexedColumn {
    pub fn new(schema: Schema) -> Self {
        Self {
            schema
        }
    }
}

impl Rule for JoinOnNonIndexedColumn {
    fn info(&self) -> RuleInfo {
        RuleInfo {
            id:       "SCHEMA004",
            name:     "JOIN on non-indexed column",
            severity: Severity::Warning,
            category: RuleCategory::Performance
        }
    }

    fn check(&self, query: &Query, query_index: usize) -> Vec<Violation> {
        if query.query_type != QueryType::Select || query.join_cols.is_empty() {
            return vec![];
        }
        let mut violations = Vec::new();
        for table_name in &query.tables {
            let Some(table) = self
                .schema
                .tables
                .values()
                .find(|t| t.name.eq_ignore_ascii_case(table_name))
            else {
                continue;
            };
            for col in &query.join_cols {
                let Some(column) = table
                    .columns
                    .iter()
                    .find(|c| c.name.eq_ignore_ascii_case(col))
                else {
                    continue;
                };
                let leads_index = column.is_primary
                    || table.indexes.iter().any(|idx| {
                        idx.columns
                            .first()
                            .is_some_and(|first| first.eq_ignore_ascii_case(col))
                    });
                if !leads_index {
                    let info = self.info();
                    violations.push(Violation {
                        rule_id: info.id,
                        rule_name: info.name,
                        message: format!(
                            "JOIN column '{}' of table '{}' does not lead any index",
                            col, table.name
                        ),
                        severity: info.severity,
                        category: info.category,
                        suggestion: Some(format!(
                            "CREATE INDEX idx_{}_{} ON {}({})",
                            table.name.to_lowercase(),
                            col.to_lowercase(),
                            table.name,
                            col
                        )),
                        query_index
                    });
                }
            }
        }
        violations
    }
}

/// Suggest indexes based on query patterns
pub struct SuggestIndex {
    schema: Schema
}

impl SuggestIndex {
    pub fn new(schema: Schema) -> Self {
        Self {
            schema
        }
    }
}

impl Rule for SuggestIndex {
    fn info(&self) -> RuleInfo {
        RuleInfo {
            id:       "SCHEMA003",
            name:     "Index suggestion",
            severity: Severity::Info,
            category: RuleCategory::Performance
        }
    }

    fn check(&self, query: &Query, query_index: usize) -> Vec<Violation> {
        if query.query_type != QueryType::Select {
            return vec![];
        }
        let indexed_cols: Vec<String> = self
            .schema
            .tables
            .values()
            .flat_map(|t| t.indexes.iter().flat_map(|idx| idx.columns.clone()))
            .collect();
        for col in &query.order_cols {
            let col_lower = col.to_lowercase();
            if !indexed_cols.iter().any(|c| c.to_lowercase() == col_lower) {
                let info = self.info();
                return vec![Violation {
                    rule_id: info.id,
                    rule_name: info.name,
                    message: format!("ORDER BY column '{}' could benefit from index", col),
                    severity: info.severity,
                    category: info.category,
                    suggestion: Some(format!(
                        "CREATE INDEX idx_{col_lower} ON table({col})",
                        col_lower = col.to_lowercase(),
                        col = col
                    )),
                    query_index
                }];
            }
        }
        vec![]
    }
}
