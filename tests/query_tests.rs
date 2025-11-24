use sql_query_analyzer::query::{QueryType, SqlDialect, parse_queries};

#[test]
fn test_parse_simple_select() {
    let sql = "SELECT id, name FROM users WHERE id = 1";
    let queries = parse_queries(sql, SqlDialect::Generic).unwrap();

    assert_eq!(queries.len(), 1);
    assert_eq!(queries[0].query_type, QueryType::Select);
    assert_eq!(queries[0].tables.len(), 1);
    assert_eq!(queries[0].tables[0].as_str(), "users");
    assert!(queries[0].where_cols.iter().any(|c| c.as_str() == "id"));
}

#[test]
fn test_parse_select_star() {
    let sql = "SELECT * FROM orders";
    let queries = parse_queries(sql, SqlDialect::Generic).unwrap();

    assert_eq!(queries.len(), 1);
    assert_eq!(queries[0].query_type, QueryType::Select);
    assert_eq!(queries[0].tables[0].as_str(), "orders");
}

#[test]
fn test_parse_join() {
    let sql = "SELECT u.id, o.total FROM users u JOIN orders o ON u.id = o.user_id";
    let queries = parse_queries(sql, SqlDialect::Generic).unwrap();

    assert_eq!(queries.len(), 1);
    assert_eq!(queries[0].tables.len(), 2);
}

#[test]
fn test_parse_multiple_queries() {
    let sql = "SELECT * FROM users; SELECT * FROM orders;";
    let queries = parse_queries(sql, SqlDialect::Generic).unwrap();

    assert_eq!(queries.len(), 2);
}

#[test]
fn test_parse_insert() {
    let sql = "INSERT INTO users (id, name) VALUES (1, 'test')";
    let queries = parse_queries(sql, SqlDialect::Generic).unwrap();

    assert_eq!(queries.len(), 1);
    assert_eq!(queries[0].query_type, QueryType::Insert);
    assert_eq!(queries[0].tables[0].as_str(), "users");
}

#[test]
fn test_parse_update() {
    let sql = "UPDATE users SET name = 'new' WHERE id = 1";
    let queries = parse_queries(sql, SqlDialect::Generic).unwrap();

    assert_eq!(queries.len(), 1);
    assert_eq!(queries[0].query_type, QueryType::Update);
    assert_eq!(queries[0].tables[0].as_str(), "users");
    assert!(queries[0].where_cols.iter().any(|c| c.as_str() == "id"));
}

#[test]
fn test_parse_update_without_where() {
    let sql = "UPDATE users SET status = 'inactive'";
    let queries = parse_queries(sql, SqlDialect::Generic).unwrap();

    assert_eq!(queries.len(), 1);
    assert_eq!(queries[0].query_type, QueryType::Update);
    assert!(queries[0].where_cols.is_empty());
}

#[test]
fn test_parse_delete() {
    let sql = "DELETE FROM users WHERE id = 1";
    let queries = parse_queries(sql, SqlDialect::Generic).unwrap();

    assert_eq!(queries.len(), 1);
    assert_eq!(queries[0].query_type, QueryType::Delete);
    assert!(queries[0].where_cols.iter().any(|c| c.as_str() == "id"));
}

#[test]
fn test_parse_delete_without_where() {
    let sql = "DELETE FROM users";
    let queries = parse_queries(sql, SqlDialect::Generic).unwrap();

    assert_eq!(queries.len(), 1);
    assert_eq!(queries[0].query_type, QueryType::Delete);
    assert!(queries[0].where_cols.is_empty());
}

#[test]
fn test_parse_limit_offset() {
    let sql = "SELECT * FROM users LIMIT 10 OFFSET 20";
    let queries = parse_queries(sql, SqlDialect::Generic).unwrap();

    assert_eq!(queries.len(), 1);
    assert_eq!(queries[0].limit, Some(10));
    assert_eq!(queries[0].offset, Some(20));
}

#[test]
fn test_parse_order_by() {
    let sql = "SELECT * FROM users ORDER BY created_at DESC, name ASC";
    let queries = parse_queries(sql, SqlDialect::Generic).unwrap();

    assert_eq!(queries.len(), 1);
    assert!(
        queries[0]
            .order_cols
            .iter()
            .any(|c| c.as_str() == "created_at")
    );
    assert!(queries[0].order_cols.iter().any(|c| c.as_str() == "name"));
}

#[test]
fn test_parse_group_by() {
    let sql = "SELECT status, COUNT(*) FROM orders GROUP BY status";
    let queries = parse_queries(sql, SqlDialect::Generic).unwrap();

    assert_eq!(queries.len(), 1);
    assert!(queries[0].group_cols.iter().any(|c| c.as_str() == "status"));
}

#[test]
fn test_parse_having() {
    let sql = "SELECT user_id, SUM(total) FROM orders GROUP BY user_id HAVING SUM(total) > 100";
    let queries = parse_queries(sql, SqlDialect::Generic).unwrap();

    assert_eq!(queries.len(), 1);
    assert!(!queries[0].having_cols.is_empty());
}

#[test]
fn test_parse_distinct() {
    let sql = "SELECT DISTINCT status FROM orders";
    let queries = parse_queries(sql, SqlDialect::Generic).unwrap();

    assert_eq!(queries.len(), 1);
    assert!(queries[0].has_distinct);
}

#[test]
fn test_parse_union() {
    let sql = "SELECT id FROM users UNION SELECT id FROM admins";
    let queries = parse_queries(sql, SqlDialect::Generic).unwrap();

    assert_eq!(queries.len(), 1);
    assert!(queries[0].has_union);
}

#[test]
fn test_parse_subquery() {
    let sql = "SELECT * FROM users WHERE id IN (SELECT user_id FROM orders)";
    let queries = parse_queries(sql, SqlDialect::Generic).unwrap();

    assert_eq!(queries.len(), 1);
    assert!(queries[0].has_subquery);
}

#[test]
fn test_parse_cte() {
    let sql = "WITH active_users AS (SELECT * FROM users WHERE active = true) SELECT * FROM active_users";
    let queries = parse_queries(sql, SqlDialect::Generic).unwrap();

    assert_eq!(queries.len(), 1);
    assert_eq!(queries[0].cte_names.len(), 1);
    assert_eq!(queries[0].cte_names[0].as_str(), "active_users");
}

#[test]
fn test_parse_multiple_joins() {
    let sql = "SELECT * FROM users u
               JOIN orders o ON u.id = o.user_id
               JOIN products p ON o.product_id = p.id";
    let queries = parse_queries(sql, SqlDialect::Generic).unwrap();

    assert_eq!(queries.len(), 1);
    assert_eq!(queries[0].tables.len(), 3);
}

#[test]
fn test_parse_left_join() {
    let sql = "SELECT * FROM users u LEFT JOIN orders o ON u.id = o.user_id";
    let queries = parse_queries(sql, SqlDialect::Generic).unwrap();

    assert_eq!(queries.len(), 1);
    assert_eq!(queries[0].tables.len(), 2);
}

#[test]
fn test_complexity_simple() {
    let sql = "SELECT * FROM users";
    let queries = parse_queries(sql, SqlDialect::Generic).unwrap();

    let complexity = queries[0].complexity();
    assert!(complexity.score < 5);
}

#[test]
fn test_complexity_complex() {
    let sql = "SELECT u.*, o.total, p.name
               FROM users u
               JOIN orders o ON u.id = o.user_id
               JOIN products p ON o.product_id = p.id
               WHERE u.active = true AND o.status = 'completed'
               GROUP BY u.id
               HAVING COUNT(*) > 5
               ORDER BY o.created_at DESC";
    let queries = parse_queries(sql, SqlDialect::Generic).unwrap();

    let complexity = queries[0].complexity();
    assert!(complexity.score >= 5);
    assert_eq!(complexity.table_count, 3);
}

#[test]
fn test_parse_like_wildcard() {
    let sql = "SELECT * FROM users WHERE name LIKE '%test%'";
    let queries = parse_queries(sql, SqlDialect::Generic).unwrap();

    assert_eq!(queries.len(), 1);
    assert_eq!(queries[0].tables[0].as_str(), "users");
}

#[test]
fn test_parse_in_clause() {
    let sql = "SELECT * FROM users WHERE status IN ('active', 'pending')";
    let queries = parse_queries(sql, SqlDialect::Generic).unwrap();

    assert_eq!(queries.len(), 1);
    assert!(queries[0].where_cols.iter().any(|c| c.as_str() == "status"));
}

#[test]
fn test_parse_between() {
    let sql = "SELECT * FROM orders WHERE created_at BETWEEN '2024-01-01' AND '2024-12-31'";
    let queries = parse_queries(sql, SqlDialect::Generic).unwrap();

    assert_eq!(queries.len(), 1);
    assert!(
        queries[0]
            .where_cols
            .iter()
            .any(|c| c.as_str() == "created_at")
    );
}

#[test]
fn test_parse_invalid_sql() {
    let sql = "SELEKT * FORM users";
    let result = parse_queries(sql, SqlDialect::Generic);

    assert!(result.is_err());
}

#[test]
fn test_mysql_dialect() {
    let sql = "SELECT * FROM users LIMIT 10";
    let queries = parse_queries(sql, SqlDialect::MySQL).unwrap();

    assert_eq!(queries.len(), 1);
}

#[test]
fn test_postgresql_dialect() {
    let sql = "SELECT * FROM users LIMIT 10";
    let queries = parse_queries(sql, SqlDialect::PostgreSQL).unwrap();

    assert_eq!(queries.len(), 1);
}

#[test]
fn test_sqlite_dialect() {
    let sql = "SELECT * FROM users LIMIT 10";
    let queries = parse_queries(sql, SqlDialect::SQLite).unwrap();

    assert_eq!(queries.len(), 1);
}
