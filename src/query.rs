mod extract;
mod types;

use extract::{ExtractionContext, extract_columns_from_expr, extract_from_set_expr};
use indexmap::IndexSet;
use rayon::prelude::*;
use sqlparser::{
    dialect::{
        ClickHouseDialect, Dialect, GenericDialect, MySqlDialect, PostgreSqlDialect, SQLiteDialect
    },
    parser::Parser
};
pub use types::{Query, QueryType};

use crate::error::{AppResult, query_parse_error};

/// SQL dialect for parsing
#[derive(Debug, Clone, Copy, Default)]
#[non_exhaustive]
pub enum SqlDialect {
    #[default]
    Generic,
    MySQL,
    PostgreSQL,
    SQLite,
    ClickHouse
}

impl SqlDialect {
    /// Convert to sqlparser dialect for parsing
    pub fn into_parser_dialect(self) -> Box<dyn Dialect> {
        match self {
            Self::Generic => Box::new(GenericDialect {}),
            Self::MySQL => Box::new(MySqlDialect {}),
            Self::PostgreSQL => Box::new(PostgreSqlDialect {}),
            Self::SQLite => Box::new(SQLiteDialect {}),
            Self::ClickHouse => Box::new(ClickHouseDialect {})
        }
    }
}

/// Parse multiple SQL queries from string (parallel)
///
/// # Notes
///
/// - Parses statements in parallel for better performance
pub fn parse_queries(sql: &str, dialect: SqlDialect) -> AppResult<Vec<Query>> {
    let parser_dialect = dialect.into_parser_dialect();
    let statements = Parser::parse_sql(parser_dialect.as_ref(), sql)
        .map_err(|e| query_parse_error(e.to_string()))?;
    let queries: Result<Vec<_>, _> = statements.into_par_iter().map(parse_statement).collect();
    queries
}

fn parse_statement(stmt: sqlparser::ast::Statement) -> AppResult<Query> {
    use sqlparser::ast::Statement;
    let raw = stmt.to_string();
    match stmt {
        Statement::Query(query) => parse_select_query(raw, *query),
        Statement::Insert(insert) => {
            let mut q = Query::new(raw, QueryType::Insert);
            q.tables.push(insert.table.to_string().into());
            Ok(q)
        }
        Statement::Update {
            table,
            selection,
            ..
        } => {
            let mut q = Query::new(raw, QueryType::Update);
            q.tables.push(table.relation.to_string().into());
            if let Some(sel) = selection {
                let mut cols = IndexSet::new();
                extract_columns_from_expr(&sel, &mut cols);
                q.where_cols = cols.into_iter().collect();
            }
            Ok(q)
        }
        Statement::Delete(delete) => {
            let mut q = Query::new(raw, QueryType::Delete);
            if let Some(sel) = delete.selection {
                let mut cols = IndexSet::new();
                extract_columns_from_expr(&sel, &mut cols);
                q.where_cols = cols.into_iter().collect();
            }
            if let sqlparser::ast::FromTable::WithFromKeyword(from_items) = delete.from {
                for item in from_items {
                    q.tables.push(item.relation.to_string().into());
                }
            }
            Ok(q)
        }
        Statement::Truncate {
            table_names, ..
        } => {
            let mut q = Query::new(raw, QueryType::Truncate);
            for table in table_names {
                q.tables.push(table.name.to_string().into());
            }
            Ok(q)
        }
        Statement::Drop {
            names,
            object_type,
            ..
        } => {
            let mut q = Query::new(raw, QueryType::Drop);
            for name in names {
                q.tables.push(name.to_string().into());
            }
            q.cte_names
                .push(format!("{:?}", object_type).to_lowercase().into());
            Ok(q)
        }
        _ => Ok(Query::new(raw, QueryType::Other))
    }
}

fn parse_select_query(raw: String, query: sqlparser::ast::Query) -> AppResult<Query> {
    let mut q = Query::new(raw, QueryType::Select);
    for cte in &query
        .with
        .iter()
        .flat_map(|w| &w.cte_tables)
        .collect::<Vec<_>>()
    {
        q.cte_names.push(cte.alias.name.value.as_str().into());
    }
    if let Some(limit_clause) = &query.limit_clause {
        match limit_clause {
            sqlparser::ast::LimitClause::LimitOffset {
                limit,
                offset,
                ..
            } => {
                if let Some(sqlparser::ast::Expr::Value(val)) = limit
                    && let sqlparser::ast::Value::Number(n, _) = &val.value
                {
                    q.limit = n.parse().ok();
                }
                if let Some(offset_expr) = offset
                    && let sqlparser::ast::Expr::Value(val) = &offset_expr.value
                    && let sqlparser::ast::Value::Number(n, _) = &val.value
                {
                    q.offset = n.parse().ok();
                }
            }
            sqlparser::ast::LimitClause::OffsetCommaLimit {
                offset,
                limit,
                ..
            } => {
                if let sqlparser::ast::Expr::Value(val) = limit
                    && let sqlparser::ast::Value::Number(n, _) = &val.value
                {
                    q.limit = n.parse().ok();
                }
                if let sqlparser::ast::Expr::Value(val) = offset
                    && let sqlparser::ast::Value::Number(n, _) = &val.value
                {
                    q.offset = n.parse().ok();
                }
            }
        }
    }
    if let Some(order_by) = &query.order_by
        && let sqlparser::ast::OrderByKind::Expressions(exprs) = &order_by.kind
    {
        let mut cols = IndexSet::new();
        for expr in exprs {
            extract_columns_from_expr(&expr.expr, &mut cols);
        }
        q.order_cols = cols.into_iter().collect();
    }
    let mut tables = IndexSet::new();
    let mut where_cols = IndexSet::new();
    let mut join_cols = IndexSet::new();
    let mut group_cols = IndexSet::new();
    let mut having_cols = IndexSet::new();
    let mut window_funcs = Vec::new();
    let mut ctx = ExtractionContext {
        tables:       &mut tables,
        where_cols:   &mut where_cols,
        join_cols:    &mut join_cols,
        group_cols:   &mut group_cols,
        having_cols:  &mut having_cols,
        window_funcs: &mut window_funcs,
        has_union:    &mut q.has_union,
        has_distinct: &mut q.has_distinct,
        has_subquery: &mut q.has_subquery
    };
    extract_from_set_expr(&query.body, &mut ctx);
    q.tables = tables.into_iter().collect();
    q.where_cols = where_cols.into_iter().collect();
    q.join_cols = join_cols.into_iter().collect();
    q.group_cols = group_cols.into_iter().collect();
    q.having_cols = having_cols.into_iter().collect();
    q.window_funcs = window_funcs;
    Ok(q)
}
