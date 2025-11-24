use compact_str::CompactString;
use indexmap::IndexSet;

use super::types::WindowFunction;

/// Context for extracting query metadata
pub struct ExtractionContext<'a> {
    pub tables:       &'a mut IndexSet<CompactString>,
    pub where_cols:   &'a mut IndexSet<CompactString>,
    pub join_cols:    &'a mut IndexSet<CompactString>,
    pub group_cols:   &'a mut IndexSet<CompactString>,
    pub having_cols:  &'a mut IndexSet<CompactString>,
    pub window_funcs: &'a mut Vec<WindowFunction>,
    pub has_union:    &'a mut bool,
    pub has_distinct: &'a mut bool,
    pub has_subquery: &'a mut bool
}

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

pub fn extract_columns_from_expr(
    expr: &sqlparser::ast::Expr,
    columns: &mut IndexSet<CompactString>
) {
    use sqlparser::ast::Expr;

    match expr {
        Expr::Identifier(ident) => {
            columns.insert(ident.value.as_str().into());
        }
        Expr::CompoundIdentifier(idents) => {
            if let Some(col) = idents.last() {
                columns.insert(col.value.as_str().into());
            }
        }
        Expr::BinaryOp {
            left,
            right,
            ..
        } => {
            extract_columns_from_expr(left, columns);
            extract_columns_from_expr(right, columns);
        }
        Expr::UnaryOp {
            expr, ..
        } => {
            extract_columns_from_expr(expr, columns);
        }
        Expr::InList {
            expr,
            list,
            ..
        } => {
            extract_columns_from_expr(expr, columns);
            for item in list {
                extract_columns_from_expr(item, columns);
            }
        }
        Expr::InSubquery {
            expr, ..
        } => {
            extract_columns_from_expr(expr, columns);
        }
        Expr::Subquery(_)
        | Expr::Exists {
            ..
        } => {}
        Expr::Between {
            expr,
            low,
            high,
            ..
        } => {
            extract_columns_from_expr(expr, columns);
            extract_columns_from_expr(low, columns);
            extract_columns_from_expr(high, columns);
        }
        Expr::IsNull(e) | Expr::IsNotNull(e) => {
            extract_columns_from_expr(e, columns);
        }
        Expr::Nested(e) => {
            extract_columns_from_expr(e, columns);
        }
        Expr::Function(func) => {
            if let sqlparser::ast::FunctionArguments::List(arg_list) = &func.args {
                for arg in &arg_list.args {
                    if let sqlparser::ast::FunctionArg::Unnamed(
                        sqlparser::ast::FunctionArgExpr::Expr(e)
                    ) = arg
                    {
                        extract_columns_from_expr(e, columns);
                    }
                }
            }
        }
        Expr::Case {
            operand,
            conditions,
            else_result,
            ..
        } => {
            if let Some(op) = operand {
                extract_columns_from_expr(op, columns);
            }
            for case_when in conditions {
                extract_columns_from_expr(&case_when.condition, columns);
                extract_columns_from_expr(&case_when.result, columns);
            }
            if let Some(else_res) = else_result {
                extract_columns_from_expr(else_res, columns);
            }
        }
        Expr::Cast {
            expr, ..
        } => {
            extract_columns_from_expr(expr, columns);
        }
        Expr::Extract {
            expr, ..
        } => {
            extract_columns_from_expr(expr, columns);
        }
        _ => {}
    }
}

fn extract_window_functions(expr: &sqlparser::ast::Expr, windows: &mut Vec<WindowFunction>) {
    use sqlparser::ast::Expr;

    match expr {
        Expr::Function(func) => {
            if let Some(over) = &func.over {
                let mut partition_cols = Vec::new();
                let mut order_cols = Vec::new();

                if let sqlparser::ast::WindowType::WindowSpec(spec) = over {
                    for part_expr in &spec.partition_by {
                        if let Expr::Identifier(ident) = part_expr {
                            partition_cols.push(ident.value.as_str().into());
                        } else if let Expr::CompoundIdentifier(idents) = part_expr
                            && let Some(col) = idents.last()
                        {
                            partition_cols.push(col.value.as_str().into());
                        }
                    }

                    for order_expr in &spec.order_by {
                        if let Expr::Identifier(ident) = &order_expr.expr {
                            order_cols.push(ident.value.as_str().into());
                        } else if let Expr::CompoundIdentifier(idents) = &order_expr.expr
                            && let Some(col) = idents.last()
                        {
                            order_cols.push(col.value.as_str().into());
                        }
                    }
                }

                windows.push(WindowFunction {
                    name: func.name.to_string().into(),
                    partition_cols,
                    order_cols
                });
            }

            if let sqlparser::ast::FunctionArguments::List(arg_list) = &func.args {
                for arg in &arg_list.args {
                    if let sqlparser::ast::FunctionArg::Unnamed(
                        sqlparser::ast::FunctionArgExpr::Expr(e)
                    ) = arg
                    {
                        extract_window_functions(e, windows);
                    }
                }
            }
        }
        Expr::BinaryOp {
            left,
            right,
            ..
        } => {
            extract_window_functions(left, windows);
            extract_window_functions(right, windows);
        }
        Expr::Nested(e) => extract_window_functions(e, windows),
        Expr::Case {
            operand,
            conditions,
            else_result,
            ..
        } => {
            if let Some(op) = operand {
                extract_window_functions(op, windows);
            }
            for cw in conditions {
                extract_window_functions(&cw.condition, windows);
                extract_window_functions(&cw.result, windows);
            }
            if let Some(e) = else_result {
                extract_window_functions(e, windows);
            }
        }
        _ => {}
    }
}

fn contains_subquery(expr: &sqlparser::ast::Expr) -> bool {
    use sqlparser::ast::Expr;

    match expr {
        Expr::Subquery(_)
        | Expr::InSubquery {
            ..
        }
        | Expr::Exists {
            ..
        } => true,
        Expr::BinaryOp {
            left,
            right,
            ..
        } => contains_subquery(left) || contains_subquery(right),
        Expr::Nested(e) => contains_subquery(e),
        Expr::InList {
            expr,
            list,
            ..
        } => contains_subquery(expr) || list.iter().any(contains_subquery),
        Expr::Case {
            operand,
            conditions,
            else_result,
            ..
        } => {
            operand.as_ref().is_some_and(|o| contains_subquery(o))
                || conditions
                    .iter()
                    .any(|cw| contains_subquery(&cw.condition) || contains_subquery(&cw.result))
                || else_result.as_ref().is_some_and(|e| contains_subquery(e))
        }
        _ => false
    }
}
