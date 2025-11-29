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
//! use sql_query_analyzer::{query::SqlDialect, schema::Schema};
//!
//! let sql = r#"
//!     CREATE TABLE users (
//!         id INT PRIMARY KEY,
//!         email VARCHAR(255) NOT NULL
//!     );
//!     CREATE INDEX idx_email ON users(email);
//! "#;
//!
//! let schema = Schema::parse(sql, SqlDialect::Generic).unwrap();
//!
//! let users = schema.tables.get("users").unwrap();
//! assert_eq!(users.columns.len(), 2);
//! assert_eq!(users.indexes.len(), 1);
//!
//! let summary = schema.to_summary();
//! assert!(summary.contains("users"));
//! ```

use std::collections::BTreeMap;

use sqlparser::parser::Parser;

use crate::{
    error::{AppResult, schema_parse_error},
    query::SqlDialect
};

/// Complete information about a database table.
#[derive(Debug, Clone)]
pub struct TableInfo {
    /// Table name
    pub name:         String,
    /// Ordered list of columns
    pub columns:      Vec<ColumnInfo>,
    /// Indexes defined on this table
    pub indexes:      Vec<IndexInfo>,
    /// Storage engine (ClickHouse: MergeTree, ReplicatedMergeTree, etc.)
    pub engine:       Option<String>,
    /// Physical sort order columns (ClickHouse ORDER BY)
    pub order_by:     Option<Vec<String>>,
    /// Sparse index columns (ClickHouse PRIMARY KEY)
    pub primary_key:  Option<Vec<String>>,
    /// Partitioning expression (ClickHouse PARTITION BY)
    pub partition_by: Option<String>,
    /// Cluster name (ClickHouse ON CLUSTER)
    pub cluster:      Option<String>
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
    pub is_primary:  bool,
    /// Compression codec (ClickHouse: ZSTD, LZ4, Delta, etc.)
    pub codec:       Option<String>
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
    /// Parse SQL schema from string with specified dialect
    ///
    /// # Arguments
    ///
    /// * `sql` - SQL schema definition
    /// * `dialect` - SQL dialect for parsing
    ///
    /// # Returns
    ///
    /// Parsed schema with tables and indexes
    ///
    /// # Errors
    ///
    /// Returns error if SQL parsing fails
    pub fn parse(sql: &str, dialect: SqlDialect) -> AppResult<Self> {
        let parser_dialect = dialect.into_parser_dialect();
        let statements = Parser::parse_sql(parser_dialect.as_ref(), sql)
            .map_err(|e| schema_parse_error(e.to_string()))?;
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
                        is_primary,
                        codec: None
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
                        indexes,
                        engine: None,
                        order_by: None,
                        primary_key: None,
                        partition_by: None,
                        cluster: None
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
            if let Some(engine) = &table.engine {
                summary.push_str(&format!("Engine: {}\n", engine));
            }
            if let Some(cluster) = &table.cluster {
                summary.push_str(&format!("Cluster: {}\n", cluster));
            }
            if let Some(partition_by) = &table.partition_by {
                summary.push_str(&format!("Partition By: {}\n", partition_by));
            }
            if let Some(order_by) = &table.order_by {
                summary.push_str(&format!("Order By: ({})\n", order_by.join(", ")));
            }
            if let Some(primary_key) = &table.primary_key {
                summary.push_str(&format!("Primary Key: ({})\n", primary_key.join(", ")));
            }
            summary.push_str("Columns:\n");
            for col in &table.columns {
                let nullable = if col.is_nullable { "NULL" } else { "NOT NULL" };
                let primary = if col.is_primary { " PRIMARY KEY" } else { "" };
                let codec = col
                    .codec
                    .as_ref()
                    .map(|c| format!(" CODEC({})", c))
                    .unwrap_or_default();
                summary.push_str(&format!(
                    "  - {name} {data_type} {nullable}{primary}{codec}\n",
                    name = col.name,
                    data_type = col.data_type,
                    nullable = nullable,
                    primary = primary,
                    codec = codec
                ));
            }
            if !table.indexes.is_empty() {
                summary.push_str("Indexes:\n");
                for idx in &table.indexes {
                    let unique = if idx.is_unique { "UNIQUE " } else { "" };
                    summary.push_str(&format!(
                        "  - {unique}INDEX {name} ON ({columns})\n",
                        unique = unique,
                        name = idx.name,
                        columns = idx.columns.join(", ")
                    ));
                }
            }
            summary.push('\n');
        }
        summary
    }
}
