use super::{Rule, RuleCategory, RuleInfo, Severity, Violation};
use crate::query::{Query, QueryType};

/// UPDATE without WHERE affects all rows
pub struct MissingWhereInUpdate;

impl Rule for MissingWhereInUpdate {
    fn info(&self) -> RuleInfo {
        RuleInfo {
            id:       "SEC001",
            name:     "UPDATE without WHERE",
            severity: Severity::Error,
            category: RuleCategory::Security
        }
    }

    fn check(&self, query: &Query, query_index: usize) -> Vec<Violation> {
        if query.query_type != QueryType::Update {
            return vec![];
        }
        if query.where_cols.is_empty() {
            let info = self.info();
            return vec![Violation {
                rule_id: info.id,
                rule_name: info.name,
                message: "UPDATE statement without WHERE clause will affect all rows".to_string(),
                severity: info.severity,
                category: info.category,
                suggestion: Some("Add WHERE clause to limit affected rows".to_string()),
                query_index
            }];
        }
        vec![]
    }
}

/// DELETE without WHERE affects all rows
pub struct MissingWhereInDelete;

impl Rule for MissingWhereInDelete {
    fn info(&self) -> RuleInfo {
        RuleInfo {
            id:       "SEC002",
            name:     "DELETE without WHERE",
            severity: Severity::Error,
            category: RuleCategory::Security
        }
    }

    fn check(&self, query: &Query, query_index: usize) -> Vec<Violation> {
        if query.query_type != QueryType::Delete {
            return vec![];
        }
        if query.where_cols.is_empty() {
            let info = self.info();
            return vec![Violation {
                rule_id: info.id,
                rule_name: info.name,
                message: "DELETE statement without WHERE clause will remove all rows".to_string(),
                severity: info.severity,
                category: info.category,
                suggestion: Some("Add WHERE clause to limit deleted rows".to_string()),
                query_index
            }];
        }
        vec![]
    }
}
