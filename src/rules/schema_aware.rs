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

        // Check WHERE columns
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

        // Check JOIN columns
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

        // Combine all columns from query
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
            // Skip common literals and expressions
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

        // Check for ORDER BY columns without index
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
                        "CREATE INDEX idx_{} ON table({})",
                        col.to_lowercase(),
                        col
                    )),
                    query_index
                }];
            }
        }

        vec![]
    }
}
