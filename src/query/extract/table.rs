use compact_str::CompactString;
use indexmap::IndexSet;

use super::{ExtractionContext, set_expr::extract_from_set_expr};

pub fn extract_from_table_factor(
    table_factor: &sqlparser::ast::TableFactor,
    tables: &mut IndexSet<CompactString>
) {
    use sqlparser::ast::TableFactor;

    match table_factor {
        TableFactor::Table {
            name, ..
        } => {
            tables.insert(name.to_string().into());
        }
        TableFactor::Derived {
            subquery,
            alias,
            ..
        } => {
            if let Some(alias) = alias {
                tables.insert(format!("(subquery) AS {}", alias.name.value).into());
            }
            let mut sub_where = IndexSet::new();
            let mut sub_join = IndexSet::new();
            let mut sub_group = IndexSet::new();
            let mut sub_having = IndexSet::new();
            let mut sub_windows = Vec::new();
            let mut has_union = false;
            let mut has_distinct = false;
            let mut has_subquery = false;
            let mut ctx = ExtractionContext {
                tables,
                where_cols: &mut sub_where,
                join_cols: &mut sub_join,
                group_cols: &mut sub_group,
                having_cols: &mut sub_having,
                window_funcs: &mut sub_windows,
                has_union: &mut has_union,
                has_distinct: &mut has_distinct,
                has_subquery: &mut has_subquery
            };
            extract_from_set_expr(&subquery.body, &mut ctx);
        }
        TableFactor::TableFunction {
            ..
        } => {}
        TableFactor::NestedJoin {
            table_with_joins, ..
        } => {
            extract_from_table_factor(&table_with_joins.relation, tables);
            for join in &table_with_joins.joins {
                extract_from_table_factor(&join.relation, tables);
            }
        }
        _ => {}
    }
}
