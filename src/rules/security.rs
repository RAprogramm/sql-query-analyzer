use super::{Rule, RuleCategory, RuleInfo, Severity, Violation};
use crate::query::{Query, QueryType};

/// Detects TRUNCATE statements which can instantly delete all data
///
/// TRUNCATE is one of the most dangerous SQL operations:
/// - Instant data loss without possibility of rollback in most configurations
/// - No WHERE clause - cannot be limited to specific rows
/// - Bypasses triggers - DELETE triggers don't fire on TRUNCATE
/// - Minimal logging - harder to recover from transaction logs
pub struct TruncateDetected;

impl Rule for TruncateDetected {
    fn info(&self) -> RuleInfo {
        RuleInfo {
            id:       "SEC003",
            name:     "TRUNCATE statement detected",
            severity: Severity::Error,
            category: RuleCategory::Security
        }
    }

    fn check(&self, query: &Query, query_index: usize) -> Vec<Violation> {
        if query.query_type != QueryType::Truncate {
            return vec![];
        }
        let info = self.info();
        let table_names = query.tables.join(", ");
        vec![Violation {
            rule_id: info.id,
            rule_name: info.name,
            message: format!(
                "TRUNCATE removes all rows from table(s) '{}' without logging individual deletions",
                table_names
            ),
            severity: info.severity,
            category: info.category,
            suggestion: Some(
                "Use DELETE with WHERE for safer data removal, or ensure backups exist"
                    .to_string()
            ),
            query_index
        }]
    }
}

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

/// Detects DROP TABLE/DATABASE statements which permanently destroy data
///
/// DROP operations are irreversible and catastrophic:
/// - Permanent data loss with no undo after commit
/// - Entire table structure is removed
/// - Cascading effects on foreign keys, views, and stored procedures
pub struct DropDetected;

impl Rule for DropDetected {
    fn info(&self) -> RuleInfo {
        RuleInfo {
            id:       "SEC004",
            name:     "DROP statement detected",
            severity: Severity::Error,
            category: RuleCategory::Security
        }
    }

    fn check(&self, query: &Query, query_index: usize) -> Vec<Violation> {
        if query.query_type != QueryType::Drop {
            return vec![];
        }
        let info = self.info();
        let object_type = query
            .cte_names
            .first()
            .map(|s| s.as_str())
            .unwrap_or("object");
        let names = query.tables.join(", ");
        vec![Violation {
            rule_id: info.id,
            rule_name: info.name,
            message: format!(
                "DROP {} '{}' permanently destroys data and schema",
                object_type, names
            ),
            severity: info.severity,
            category: info.category,
            suggestion: Some(
                "Ensure this is intentional and backups exist before dropping".to_string()
            ),
            query_index
        }]
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
