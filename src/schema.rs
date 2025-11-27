//! Database schema parsing and representation.
//!
//! This module parses SQL DDL statements (CREATE TABLE, CREATE INDEX) into a
//! structured representation that can be used for schema-aware query analysis.
//!
//! # Supported Statements
//!
//! - `CREATE TABLE` with columns, types, constraints
//! - `CREATE INDEX` with column lists and uniqueness
//! - Primary key constraints (inline and table-level)
//! - NOT NULL constraints
//!
//! # Example
//!
//! ```
//! use sql_query_analyzer::schema::Schema;
//!
//! let sql = r#"
//!     CREATE TABLE users (
//!         id INT PRIMARY KEY,
//!         email VARCHAR(255) NOT NULL
//!     );
//!     CREATE INDEX idx_email ON users(email);
//! "#;
//!
//! let schema = Schema::parse(sql).unwrap();
//!
//! let users = schema.tables.get("users").unwrap();
//! assert_eq!(users.columns.len(), 2);
//! assert_eq!(users.indexes.len(), 1);
//!
//! let summary = schema.to_summary();
//! assert!(summary.contains("users"));
//! ```

use std::collections::BTreeMap;

use sqlparser::{dialect::GenericDialect, parser::Parser};

use crate::error::{AppResult, schema_parse_error};

/// Complete information about a database table.
#[derive(Debug, Clone)]
pub struct TableInfo {
    /// Table name
    pub name:    String,
    /// Ordered list of columns
    pub columns: Vec<ColumnInfo>,
    /// Indexes defined on this table
    pub indexes: Vec<IndexInfo>
}

/// Column metadata extracted from CREATE TABLE.
#[derive(Debug, Clone)]
pub struct ColumnInfo {
    /// Column name
    pub name:        String,
    /// SQL data type (e.g., "INT", "VARCHAR(255)")
    pub data_type:   String,
    /// Whether NULL values are allowed
    pub is_nullable: bool,
    /// Whether this is a primary key column
    pub is_primary:  bool
}

/// Index metadata extracted from CREATE INDEX or table constraints.
#[derive(Debug, Clone)]
pub struct IndexInfo {
    /// Index name (may be empty for anonymous indexes)
    pub name:      String,
    /// Ordered list of indexed columns
    pub columns:   Vec<String>,
    /// Whether this is a unique index
    pub is_unique: bool
}

/// Parsed database schema containing all tables and their metadata.
///
/// Tables are stored in a `BTreeMap` for deterministic iteration order.
#[derive(Debug, Default, Clone)]
pub struct Schema {
    /// Map of table name to table information
    pub tables: BTreeMap<String, TableInfo>
}

impl Schema {
    /// Parse SQL schema from string
    ///
    /// # Arguments
    ///
    /// * `sql` - SQL schema definition
    ///
    /// # Returns
    ///
    /// Parsed schema with tables and indexes
    ///
    /// # Errors
    ///
    /// Returns error if SQL parsing fails
    pub fn parse(sql: &str) -> AppResult<Self> {
        let dialect = GenericDialect {};
        let statements =
            Parser::parse_sql(&dialect, sql).map_err(|e| schema_parse_error(e.to_string()))?;

        let mut schema = Self::default();

        for stmt in statements {
            schema.process_statement(stmt)?;
        }

        Ok(schema)
    }

    fn process_statement(&mut self, stmt: sqlparser::ast::Statement) -> AppResult<()> {
        use sqlparser::ast::Statement;

        match stmt {
            Statement::CreateTable(create) => {
                let table_name = create.name.to_string();
                let mut columns = Vec::new();
                let mut indexes = Vec::new();

                for column in create.columns {
                    let is_primary = column.options.iter().any(|opt| {
                        matches!(
                            opt.option,
                            sqlparser::ast::ColumnOption::Unique {
                                is_primary: true,
                                ..
                            }
                        )
                    });

                    columns.push(ColumnInfo {
                        name: column.name.to_string(),
                        data_type: column.data_type.to_string(),
                        is_nullable: !column.options.iter().any(|opt| {
                            matches!(opt.option, sqlparser::ast::ColumnOption::NotNull)
                        }),
                        is_primary
                    });
                }

                for constraint in create.constraints {
                    if let sqlparser::ast::TableConstraint::Index {
                        name,
                        columns: idx_cols,
                        ..
                    } = constraint
                    {
                        indexes.push(IndexInfo {
                            name:      name.map(|n| n.to_string()).unwrap_or_default(),
                            columns:   idx_cols.iter().map(|c| c.to_string()).collect(),
                            is_unique: false
                        });
                    }
                }

                self.tables.insert(
                    table_name.clone(),
                    TableInfo {
                        name: table_name,
                        columns,
                        indexes
                    }
                );
            }
            Statement::CreateIndex(create_index) => {
                let table_name = create_index.table_name.to_string();
                if let Some(table) = self.tables.get_mut(&table_name) {
                    table.indexes.push(IndexInfo {
                        name:      create_index.name.map(|n| n.to_string()).unwrap_or_default(),
                        columns:   create_index.columns.iter().map(|c| c.to_string()).collect(),
                        is_unique: create_index.unique
                    });
                }
            }
            _ => {}
        }

        Ok(())
    }

    /// Get summary of schema for LLM analysis
    pub fn to_summary(&self) -> String {
        let mut summary = String::from("Database Schema:\n\n");

        for table in self.tables.values() {
            summary.push_str(&format!("Table: {}\n", table.name));
            summary.push_str("Columns:\n");

            for col in &table.columns {
                let nullable = if col.is_nullable { "NULL" } else { "NOT NULL" };
                let primary = if col.is_primary { " PRIMARY KEY" } else { "" };
                summary.push_str(&format!(
                    "  - {} {} {}{}\n",
                    col.name, col.data_type, nullable, primary
                ));
            }

            if !table.indexes.is_empty() {
                summary.push_str("Indexes:\n");
                for idx in &table.indexes {
                    let unique = if idx.is_unique { "UNIQUE " } else { "" };
                    summary.push_str(&format!(
                        "  - {}INDEX {} ON ({})\n",
                        unique,
                        idx.name,
                        idx.columns.join(", ")
                    ));
                }
            }

            summary.push('\n');
        }

        summary
    }
}
