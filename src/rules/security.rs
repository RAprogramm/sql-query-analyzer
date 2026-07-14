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

/// Detects GRANT/REVOKE privilege changes in query files
///
/// Privilege changes belong in reviewed migrations, not application query
/// sets: an unnoticed GRANT widens the attack surface permanently. Broad
/// grants (ALL PRIVILEGES, ON *.*, TO PUBLIC, SUPERUSER) escalate the
/// violation to Error.
pub struct PrivilegeChange;

/// Grant shapes that hand out broad or public access.
const DANGEROUS_GRANT_MARKERS: [&str; 4] = ["ALL PRIVILEGES", "ON *.*", "TO PUBLIC", "SUPERUSER"];

impl Rule for PrivilegeChange {
    fn info(&self) -> RuleInfo {
        RuleInfo {
            id:       "SEC005",
            name:     "GRANT/REVOKE privilege change",
            severity: Severity::Warning,
            category: RuleCategory::Security
        }
    }

    fn check(&self, query: &Query, query_index: usize) -> Vec<Violation> {
        let upper = query.raw.to_uppercase();
        let trimmed = upper.trim_start();
        let is_grant = trimmed.starts_with("GRANT ");
        if !is_grant && !trimmed.starts_with("REVOKE ") {
            return vec![];
        }
        let dangerous = is_grant
            && DANGEROUS_GRANT_MARKERS
                .iter()
                .any(|marker| upper.contains(marker));
        let info = self.info();
        let (severity, message) = if dangerous {
            (
                Severity::Error,
                "GRANT hands out broad or public privileges".to_string()
            )
        } else {
            (
                info.severity,
                "Privilege change statement found in query set".to_string()
            )
        };
        vec![Violation {
            rule_id: info.id,
            rule_name: info.name,
            message,
            severity,
            category: info.category,
            suggestion: Some(
                "Keep GRANT/REVOKE in reviewed migrations and grant the narrowest privileges needed"
                    .to_string()
            ),
            query_index
        }]
    }
}

/// Detects plaintext credentials embedded in SQL statements
///
/// Secrets committed inside query files leak through source control, slow
/// query logs, and error logs, and violate PCI-DSS/SOC2/HIPAA plaintext
/// storage rules. Flags `IDENTIFIED BY`/`WITH PASSWORD`/`SET PASSWORD`
/// clauses and string literals assigned or inserted into sensitive columns
/// (password, secret, api_key, token, and similar).
pub struct HardcodedCredential;

const SENSITIVE_COLUMNS: [&str; 9] = [
    "PASSWORD",
    "PASSWD",
    "PWD",
    "SECRET",
    "API_KEY",
    "APIKEY",
    "TOKEN",
    "AUTH",
    "CREDENTIAL"
];

/// Returns true when `upper` contains `name` ending at a word boundary,
/// where the tail continues (skipping whitespace) with `next`. Prefixes are
/// deliberately allowed so `user_password = '...'` still matches.
fn sensitive_name_followed_by(upper: &str, name: &str, next: char) -> bool {
    upper.match_indices(name).any(|(pos, _)| {
        let after = &upper[pos + name.len()..];
        let mut chars = after.chars();
        match chars.next() {
            Some(c) if c.is_ascii_alphanumeric() || c == '_' => return false,
            None => return false,
            _ => {}
        }
        after
            .trim_start_matches(|c: char| c.is_whitespace())
            .starts_with(next)
    })
}

/// Returns true when a sensitive column is assigned a string literal
/// (`password = 'plaintext'`) anywhere in the statement.
fn has_sensitive_assignment(upper: &str) -> bool {
    SENSITIVE_COLUMNS.iter().any(|col| {
        upper.match_indices(col).any(|(pos, _)| {
            let after = &upper[pos + col.len()..];
            let mut rest = after.trim_start();
            match after.chars().next() {
                Some(c) if c.is_ascii_alphanumeric() || c == '_' => return false,
                None => return false,
                _ => {}
            }
            if !rest.starts_with('=') {
                return false;
            }
            rest = rest[1..].trim_start();
            rest.starts_with('\'')
        })
    })
}

/// Returns true when an INSERT names a sensitive column before VALUES and
/// supplies at least one string literal.
fn has_sensitive_insert(query: &Query, upper: &str) -> bool {
    if query.query_type != QueryType::Insert {
        return false;
    }
    let Some(values_pos) = upper.find(" VALUES") else {
        return false;
    };
    let (columns_part, values_part) = upper.split_at(values_pos);
    values_part.contains('\'')
        && SENSITIVE_COLUMNS.iter().any(|col| {
            sensitive_name_followed_by(columns_part, col, ',')
                || sensitive_name_followed_by(columns_part, col, ')')
        })
}

impl Rule for HardcodedCredential {
    fn info(&self) -> RuleInfo {
        RuleInfo {
            id:       "SEC008",
            name:     "Hardcoded credential detected",
            severity: Severity::Error,
            category: RuleCategory::Security
        }
    }

    fn check(&self, query: &Query, query_index: usize) -> Vec<Violation> {
        let upper = query.raw.to_uppercase();
        let ddl_credential = upper.contains("IDENTIFIED BY '")
            || upper.contains("WITH PASSWORD '")
            || upper.contains("SET PASSWORD");
        if !ddl_credential
            && !has_sensitive_assignment(&upper)
            && !has_sensitive_insert(query, &upper)
        {
            return vec![];
        }
        let info = self.info();
        vec![Violation {
            rule_id: info.id,
            rule_name: info.name,
            message: "Possible hardcoded credential in SQL statement".to_string(),
            severity: info.severity,
            category: info.category,
            suggestion: Some(
                "Use environment variables, a secret manager, or parameterized values instead of plaintext secrets"
                    .to_string()
            ),
            query_index
        }]
    }
}

/// Detects tautology patterns associated with SQL injection
///
/// A comparison of two identical literals joined by OR (`OR 1 = 1`,
/// `OR '1' = '1'`, `OR '' = ''`) is always true and almost never appears in
/// legitimate queries; it is the classic fingerprint of injected input that
/// widens a WHERE clause to match every row. Comment-marker and
/// statement-stacking heuristics are handled before parsing: the analyzer
/// receives statements re-serialized from the AST, where comments are
/// already stripped and stacked statements are split apart.
pub struct InjectionTautology;

/// Returns true for tokens the tautology check treats as literals:
/// single-quoted strings and bare integers.
fn is_literal_token(tok: &str) -> bool {
    (tok.starts_with('\'') && tok.ends_with('\'') && tok.len() >= 2)
        || (!tok.is_empty() && tok.chars().all(|c| c.is_ascii_digit()))
}

/// Returns true when any `OR a = b` with identical literal operands
/// appears in the uppercased query text.
fn has_or_tautology(upper: &str) -> bool {
    let mut search_from = 0;
    while let Some(pos) = upper[search_from..].find(" OR ") {
        let rest = &upper[search_from + pos + 4..];
        let mut toks = rest.split_whitespace();
        if let (Some(a), Some(op), Some(b)) = (toks.next(), toks.next(), toks.next())
            && op == "="
            && a == b
            && is_literal_token(a)
        {
            return true;
        }
        search_from += pos + 4;
    }
    false
}

impl Rule for InjectionTautology {
    fn info(&self) -> RuleInfo {
        RuleInfo {
            id:       "SEC006",
            name:     "Potential SQL injection pattern",
            severity: Severity::Error,
            category: RuleCategory::Security
        }
    }

    fn check(&self, query: &Query, query_index: usize) -> Vec<Violation> {
        let upper = query.raw.to_uppercase();
        if !has_or_tautology(&upper) {
            return vec![];
        }
        let info = self.info();
        vec![Violation {
            rule_id: info.id,
            rule_name: info.name,
            message: "Query contains an always-true OR tautology, a classic SQL injection pattern"
                .to_string(),
            severity: info.severity,
            category: info.category,
            suggestion: Some(
                "If this query is built in application code, replace string concatenation with parameterized queries"
                    .to_string()
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
