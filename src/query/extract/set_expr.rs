use indexmap::IndexSet;

use super::{
    ExtractionContext,
    expr::{contains_subquery, extract_columns_from_expr, extract_window_functions},
    table::extract_from_table_factor
};

pub fn extract_from_set_expr(set_expr: &sqlparser::ast::SetExpr, ctx: &mut ExtractionContext<'_>) {
    use sqlparser::ast::SetExpr;
    match set_expr {
        SetExpr::Select(select) => {
            *ctx.has_distinct = select.distinct.is_some();
            for item in &select.projection {
                if let sqlparser::ast::SelectItem::UnnamedExpr(expr)
                | sqlparser::ast::SelectItem::ExprWithAlias {
                    expr, ..
                } = item
                {
                    extract_window_functions(expr, ctx.window_funcs);
                    if contains_subquery(expr) {
                        *ctx.has_subquery = true;
                    }
                }
            }
            for table in &select.from {
                extract_from_table_factor(&table.relation, ctx.tables);
                for join in &table.joins {
                    extract_from_table_factor(&join.relation, ctx.tables);
                    match &join.join_operator {
                        sqlparser::ast::JoinOperator::Inner(constraint)
                        | sqlparser::ast::JoinOperator::LeftOuter(constraint)
                        | sqlparser::ast::JoinOperator::RightOuter(constraint)
                        | sqlparser::ast::JoinOperator::FullOuter(constraint) => {
                            if let sqlparser::ast::JoinConstraint::On(expr) = constraint {
                                extract_columns_from_expr(expr, ctx.join_cols);
                            }
                        }
                        _ => {}
                    }
                }
            }
            if let Some(selection) = &select.selection {
                extract_columns_from_expr(selection, ctx.where_cols);
                if contains_subquery(selection) {
                    *ctx.has_subquery = true;
                }
            }
            if let sqlparser::ast::GroupByExpr::Expressions(exprs, _) = &select.group_by {
                for expr in exprs {
                    extract_columns_from_expr(expr, ctx.group_cols);
                }
            }
            if let Some(having) = &select.having {
                extract_columns_from_expr(having, ctx.having_cols);
            }
        }
        SetExpr::SetOperation {
            left,
            right,
            ..
        } => {
            *ctx.has_union = true;
            extract_from_set_expr(left, ctx);
            extract_from_set_expr(right, ctx);
        }
        SetExpr::Query(query) => {
            if let Some(order_by) = &query.order_by
                && let sqlparser::ast::OrderByKind::Expressions(exprs) = &order_by.kind
            {
                let mut order_cols = IndexSet::new();
                for expr in exprs {
                    extract_columns_from_expr(&expr.expr, &mut order_cols);
                }
            }
            extract_from_set_expr(&query.body, ctx);
        }
        SetExpr::Values(_)
        | SetExpr::Insert(_)
        | SetExpr::Update(_)
        | SetExpr::Table(_)
        | SetExpr::Delete(_)
        | SetExpr::Merge(_) => {}
    }
}
