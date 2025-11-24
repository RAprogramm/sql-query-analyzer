use compact_str::CompactString;
use indexmap::IndexSet;

use crate::query::types::WindowFunction;

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

pub fn extract_window_functions(expr: &sqlparser::ast::Expr, windows: &mut Vec<WindowFunction>) {
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

pub fn contains_subquery(expr: &sqlparser::ast::Expr) -> bool {
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
