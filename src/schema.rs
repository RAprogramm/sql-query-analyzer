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

use std::{collections::BTreeMap, fmt::Write};

use sqlparser::parser::Parser;

use crate::{
    error::{AppResult, schema_parse_error},
    preprocessor::{Preprocessor, PreprocessorMetadata},
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
    /// Storage engine (`ClickHouse`: `MergeTree`, `ReplicatedMergeTree`, etc.)
    pub engine:       Option<String>,
    /// Physical sort order columns (`ClickHouse` ORDER BY)
    pub order_by:     Option<Vec<String>>,
    /// Sparse index columns (`ClickHouse` PRIMARY KEY)
    pub primary_key:  Option<Vec<String>>,
    /// Partitioning expression (`ClickHouse` PARTITION BY)
    pub partition_by: Option<String>,
    /// Sample by expression (`ClickHouse` SAMPLE BY)
    pub sample_by:    Option<String>,
    /// Data skipping indexes (`ClickHouse` INDEX ... TYPE ... GRANULARITY)
    pub skip_indexes: Vec<crate::preprocessor::SkipIndexInfo>,
    /// Projections (`ClickHouse` PROJECTION ... AS SELECT)
    pub projections:  Vec<crate::preprocessor::ProjectionInfo>,
    /// TTL expression (`ClickHouse` TTL)
    pub ttl:          Option<String>,
    /// Table-level settings (`ClickHouse` SETTINGS)
    pub settings:     std::collections::HashMap<String, String>,
    /// Cluster name (`ClickHouse` ON CLUSTER)
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
    /// Compression codec (`ClickHouse`: ZSTD, LZ4, Delta, etc.)
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
    pub tables:             BTreeMap<String, TableInfo>,
    /// `ClickHouse` dictionary names defined in the schema
    pub dictionaries:       Vec<String>,
    /// `ClickHouse` materialized views defined in the schema
    pub materialized_views: Vec<crate::preprocessor::MaterializedViewInfo>
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
        let preprocessor = Preprocessor::new(dialect);
        let preprocessed = preprocessor.process(sql);
        let parser_dialect = dialect.into_parser_dialect();
        let statements = Parser::parse_sql(parser_dialect.as_ref(), &preprocessed.sql)
            .map_err(|e| schema_parse_error(e.to_string()))?;
        let mut schema = Self::default();
        for stmt in statements {
            schema.process_statement(stmt, &preprocessed.metadata, dialect);
        }
        schema
            .dictionaries
            .clone_from(&preprocessed.metadata.dictionaries);
        schema
            .materialized_views
            .clone_from(&preprocessed.metadata.materialized_views);
        Ok(schema)
    }

    fn process_statement(
        &mut self,
        stmt: sqlparser::ast::Statement,
        metadata: &PreprocessorMetadata,
        dialect: SqlDialect
    ) {
        use sqlparser::ast::Statement;
        match stmt {
            Statement::CreateTable(create) => {
                let table_name = create.name.to_string();
                let mut columns = Vec::new();
                let mut indexes = Vec::new();
                for column in create.columns {
                    let col_name = column.name.to_string();
                    let is_primary = column.options.iter().any(|opt| {
                        matches!(opt.option, sqlparser::ast::ColumnOption::PrimaryKey(_))
                    });
                    let codec = metadata.codecs.get(&col_name).cloned();
                    let data_type = column.data_type.to_string();
                    let is_nullable = if dialect == SqlDialect::ClickHouse {
                        data_type
                            .trim_start()
                            .to_uppercase()
                            .starts_with("NULLABLE")
                    } else {
                        !column
                            .options
                            .iter()
                            .any(|opt| matches!(opt.option, sqlparser::ast::ColumnOption::NotNull))
                    };
                    columns.push(ColumnInfo {
                        name: col_name,
                        data_type,
                        is_nullable,
                        is_primary,
                        codec
                    });
                }
                for constraint in create.constraints {
                    if let sqlparser::ast::TableConstraint::Index(idx) = constraint {
                        indexes.push(IndexInfo {
                            name:      idx.name.map_or_default(|n| n.to_string()),
                            columns:   idx
                                .columns
                                .iter()
                                .map(std::string::ToString::to_string)
                                .collect(),
                            is_unique: false
                        });
                    }
                }
                let engine = Self::extract_engine(&create.table_options);
                let order_by = create.order_by.as_ref().map(Self::extract_exprs);
                let primary_key = create.primary_key.as_ref().map(|pk| vec![pk.to_string()]);
                let cluster = create.on_cluster.map(|c| c.value);
                let partition_by = metadata.partition_by.first().cloned();
                let ttl = metadata.ttl_expressions.first().cloned();
                let settings = metadata.settings.clone();
                let sample_by = metadata.sample_by.first().cloned();
                let skip_indexes = metadata.skip_indexes.clone();
                let projections = metadata.projections.clone();
                self.tables.insert(
                    table_name.clone(),
                    TableInfo {
                        name: table_name,
                        columns,
                        indexes,
                        engine,
                        order_by,
                        primary_key,
                        partition_by,
                        ttl,
                        settings,
                        sample_by,
                        skip_indexes,
                        projections,
                        cluster
                    }
                );
            }
            Statement::CreateIndex(create_index) => {
                let table_name = create_index.table_name.to_string();
                if let Some(table) = self.tables.get_mut(&table_name) {
                    table.indexes.push(IndexInfo {
                        name:      create_index.name.map_or_default(|n| n.to_string()),
                        columns:   create_index
                            .columns
                            .iter()
                            .map(std::string::ToString::to_string)
                            .collect(),
                        is_unique: create_index.unique
                    });
                }
            }
            _ => {}
        }
    }

    fn extract_engine(options: &sqlparser::ast::CreateTableOptions) -> Option<String> {
        use sqlparser::ast::{CreateTableOptions, SqlOption};
        let opts = match options {
            CreateTableOptions::Plain(opts)
            | CreateTableOptions::With(opts)
            | CreateTableOptions::Options(opts)
            | CreateTableOptions::TableProperties(opts) => opts,
            CreateTableOptions::None => return None
        };
        for opt in opts {
            if let SqlOption::NamedParenthesizedList(list) = opt
                && list.key.value.eq_ignore_ascii_case("ENGINE")
            {
                return list.name.as_ref().map(|n| n.value.clone());
            }
        }
        None
    }

    fn extract_exprs(
        exprs: &sqlparser::ast::OneOrManyWithParens<sqlparser::ast::Expr>
    ) -> Vec<String> {
        use sqlparser::ast::OneOrManyWithParens;
        match exprs {
            OneOrManyWithParens::One(expr) => vec![expr.to_string()],
            OneOrManyWithParens::Many(list) => {
                list.iter().map(std::string::ToString::to_string).collect()
            }
        }
    }

    /// Get summary of schema for LLM analysis
    #[must_use]
    pub fn to_summary(&self) -> String {
        let mut summary = String::from("Database Schema:\n\n");
        for table in self.tables.values() {
            Self::write_table_summary(table, &mut summary);
        }
        if !self.dictionaries.is_empty() {
            let _ = writeln!(summary, "Dictionaries: {}", self.dictionaries.join(", "));
        }
        if !self.materialized_views.is_empty() {
            summary.push_str("Materialized Views:\n");
            for mv in &self.materialized_views {
                let target = mv.target.as_ref().map_or_default(|t| format!(" TO {t}"));
                let _ = writeln!(
                    summary,
                    "  - {name}{target} AS {query}",
                    name = mv.name,
                    query = mv.query
                );
            }
        }
        summary
    }

    /// Append one table's full description (metadata, columns, indexes, skip
    /// indexes, projections) to `summary`.
    fn write_table_summary(table: &TableInfo, summary: &mut String) {
        let _ = writeln!(summary, "Table: {}", table.name);
        if let Some(engine) = &table.engine {
            let _ = writeln!(summary, "Engine: {engine}");
        }
        if let Some(cluster) = &table.cluster {
            let _ = writeln!(summary, "Cluster: {cluster}");
        }
        if let Some(partition_by) = &table.partition_by {
            let _ = writeln!(summary, "Partition By: {partition_by}");
        }
        if let Some(sample_by) = &table.sample_by {
            let _ = writeln!(summary, "Sample By: {sample_by}");
        }
        if let Some(ttl) = &table.ttl {
            let _ = writeln!(summary, "TTL: {ttl}");
        }
        if !table.settings.is_empty() {
            let settings: Vec<String> = table
                .settings
                .iter()
                .map(|(k, v)| format!("{k} = {v}"))
                .collect();
            let _ = writeln!(summary, "Settings: {}", settings.join(", "));
        }
        if let Some(order_by) = &table.order_by {
            let _ = writeln!(summary, "Order By: ({})", order_by.join(", "));
        }
        if let Some(primary_key) = &table.primary_key {
            let _ = writeln!(summary, "Primary Key: ({})", primary_key.join(", "));
        }
        summary.push_str("Columns:\n");
        for col in &table.columns {
            let nullable = if col.is_nullable { "NULL" } else { "NOT NULL" };
            let primary = if col.is_primary { " PRIMARY KEY" } else { "" };
            let codec = col
                .codec
                .as_ref()
                .map_or_default(|c| format!(" CODEC({c})"));
            let _ = writeln!(
                summary,
                "  - {name} {data_type} {nullable}{primary}{codec}",
                name = col.name,
                data_type = col.data_type,
                nullable = nullable,
                primary = primary,
                codec = codec
            );
        }
        if !table.indexes.is_empty() {
            summary.push_str("Indexes:\n");
            for idx in &table.indexes {
                let unique = if idx.is_unique { "UNIQUE " } else { "" };
                let _ = writeln!(
                    summary,
                    "  - {unique}INDEX {name} ON ({columns})",
                    unique = unique,
                    name = idx.name,
                    columns = idx.columns.join(", ")
                );
            }
        }
        if !table.skip_indexes.is_empty() {
            summary.push_str("Skip Indexes:\n");
            for si in &table.skip_indexes {
                let args = si.type_args.as_ref().map_or_default(|a| format!("({a})"));
                let _ = writeln!(
                    summary,
                    "  - INDEX {name} ON {expr} TYPE {ty}{args} GRANULARITY {g}",
                    name = si.name,
                    expr = si.expr,
                    ty = si.type_name,
                    args = args,
                    g = si.granularity
                );
            }
        }
        if !table.projections.is_empty() {
            summary.push_str("Projections:\n");
            for proj in &table.projections {
                let _ = writeln!(summary, "  - {} AS {}", proj.name, proj.query);
            }
        }
        summary.push('\n');
    }
}
