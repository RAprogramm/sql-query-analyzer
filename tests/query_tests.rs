// SPDX-FileCopyrightText: 2025 RAprogramm
// SPDX-License-Identifier: MIT

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
    let sql = "SELECT * FROM users u \
               JOIN orders o ON u.id = o.user_id \
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
    let sql = "SELECT u.*, o.total, p.name \
               FROM users u \
               JOIN orders o ON u.id = o.user_id \
               JOIN products p ON o.product_id = p.id \
               WHERE u.active = true AND o.status = 'completed' \
               GROUP BY u.id \
               HAVING COUNT(*) > 5 \
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

#[test]
fn test_derived_subquery_with_alias() {
    let sql = "SELECT t.id FROM (SELECT id FROM users) AS t";
    let queries = parse_queries(sql, SqlDialect::Generic).unwrap();
    assert_eq!(queries.len(), 1);
    assert!(queries[0].tables.iter().any(|t| t.contains("subquery")));
}

#[test]
fn test_nested_join() {
    let sql = "SELECT * FROM (users u INNER JOIN orders o ON u.id = o.user_id)";
    let queries = parse_queries(sql, SqlDialect::Generic).unwrap();
    assert_eq!(queries.len(), 1);
}

#[test]
fn test_window_function() {
    let sql =
        "SELECT id, ROW_NUMBER() OVER (PARTITION BY status ORDER BY created_at) as rn FROM users";
    let queries = parse_queries(sql, SqlDialect::Generic).unwrap();
    assert_eq!(queries.len(), 1);
    assert!(!queries[0].window_funcs.is_empty());
}

#[test]
fn test_window_function_dense_rank() {
    let sql = "SELECT id, DENSE_RANK() OVER (ORDER BY score DESC) as rank FROM players";
    let queries = parse_queries(sql, SqlDialect::Generic).unwrap();
    assert_eq!(queries.len(), 1);
    assert!(!queries[0].window_funcs.is_empty());
}

#[test]
fn test_case_expression() {
    let sql = "SELECT CASE WHEN status = 'active' THEN 1 ELSE 0 END FROM users";
    let queries = parse_queries(sql, SqlDialect::Generic).unwrap();
    assert_eq!(queries.len(), 1);
}

#[test]
fn test_exists_subquery() {
    let sql =
        "SELECT * FROM users WHERE EXISTS (SELECT 1 FROM orders WHERE orders.user_id = users.id)";
    let queries = parse_queries(sql, SqlDialect::Generic).unwrap();
    assert_eq!(queries.len(), 1);
    assert!(queries[0].has_subquery);
}

#[test]
fn test_not_in_subquery() {
    let sql = "SELECT * FROM users WHERE id NOT IN (SELECT user_id FROM banned)";
    let queries = parse_queries(sql, SqlDialect::Generic).unwrap();
    assert_eq!(queries.len(), 1);
    assert!(queries[0].has_subquery);
}

#[test]
fn test_scalar_subquery() {
    let sql = "SELECT id, (SELECT COUNT(*) FROM orders WHERE orders.user_id = users.id) as order_count FROM users";
    let queries = parse_queries(sql, SqlDialect::Generic).unwrap();
    assert_eq!(queries.len(), 1);
    assert!(queries[0].has_subquery);
}

#[test]
fn test_right_join() {
    let sql = "SELECT * FROM users u RIGHT JOIN orders o ON u.id = o.user_id";
    let queries = parse_queries(sql, SqlDialect::Generic).unwrap();
    assert_eq!(queries.len(), 1);
    assert_eq!(queries[0].tables.len(), 2);
}

#[test]
fn test_full_outer_join() {
    let sql = "SELECT * FROM users u FULL OUTER JOIN orders o ON u.id = o.user_id";
    let queries = parse_queries(sql, SqlDialect::Generic).unwrap();
    assert_eq!(queries.len(), 1);
    assert_eq!(queries[0].tables.len(), 2);
}

#[test]
fn test_cross_join() {
    let sql = "SELECT * FROM users CROSS JOIN products";
    let queries = parse_queries(sql, SqlDialect::Generic).unwrap();
    assert_eq!(queries.len(), 1);
    assert_eq!(queries[0].tables.len(), 2);
}

#[test]
fn test_union_all() {
    let sql = "SELECT id FROM users UNION ALL SELECT id FROM admins";
    let queries = parse_queries(sql, SqlDialect::Generic).unwrap();
    assert_eq!(queries.len(), 1);
    assert!(queries[0].has_union);
}

#[test]
fn test_intersect() {
    let sql = "SELECT id FROM users INTERSECT SELECT id FROM premium_users";
    let queries = parse_queries(sql, SqlDialect::Generic).unwrap();
    assert_eq!(queries.len(), 1);
}

#[test]
fn test_except() {
    let sql = "SELECT id FROM users EXCEPT SELECT id FROM banned_users";
    let queries = parse_queries(sql, SqlDialect::Generic).unwrap();
    assert_eq!(queries.len(), 1);
}

#[test]
fn test_coalesce() {
    let sql = "SELECT COALESCE(name, 'Unknown') FROM users";
    let queries = parse_queries(sql, SqlDialect::Generic).unwrap();
    assert_eq!(queries.len(), 1);
}

#[test]
fn test_cast() {
    let sql = "SELECT CAST(id AS VARCHAR) FROM users";
    let queries = parse_queries(sql, SqlDialect::Generic).unwrap();
    assert_eq!(queries.len(), 1);
}

#[test]
fn test_aggregate_functions() {
    let sql = "SELECT COUNT(*), SUM(total), AVG(price), MIN(id), MAX(id) FROM orders";
    let queries = parse_queries(sql, SqlDialect::Generic).unwrap();
    assert_eq!(queries.len(), 1);
}

#[test]
fn test_multiple_ctes() {
    let sql = "WITH cte1 AS (SELECT 1), cte2 AS (SELECT 2) SELECT * FROM cte1, cte2";
    let queries = parse_queries(sql, SqlDialect::Generic).unwrap();
    assert_eq!(queries.len(), 1);
    assert_eq!(queries[0].cte_names.len(), 2);
}

#[test]
fn test_recursive_cte() {
    let sql = "WITH RECURSIVE nums AS (SELECT 1 AS n UNION ALL SELECT n + 1 FROM nums WHERE n < 10) SELECT * FROM nums";
    let queries = parse_queries(sql, SqlDialect::Generic).unwrap();
    assert_eq!(queries.len(), 1);
    assert!(!queries[0].cte_names.is_empty());
}

#[test]
fn test_is_null() {
    let sql = "SELECT * FROM users WHERE email IS NULL";
    let queries = parse_queries(sql, SqlDialect::Generic).unwrap();
    assert_eq!(queries.len(), 1);
}

#[test]
fn test_is_not_null() {
    let sql = "SELECT * FROM users WHERE email IS NOT NULL";
    let queries = parse_queries(sql, SqlDialect::Generic).unwrap();
    assert_eq!(queries.len(), 1);
}

#[test]
fn test_compound_expression() {
    let sql =
        "SELECT * FROM users WHERE (status = 'active' OR status = 'pending') AND verified = true";
    let queries = parse_queries(sql, SqlDialect::Generic).unwrap();
    assert_eq!(queries.len(), 1);
}

#[test]
fn test_negative_number() {
    let sql = "SELECT * FROM accounts WHERE balance < -100";
    let queries = parse_queries(sql, SqlDialect::Generic).unwrap();
    assert_eq!(queries.len(), 1);
}

#[test]
fn test_nested_function_calls() {
    let sql = "SELECT UPPER(TRIM(name)) FROM users";
    let queries = parse_queries(sql, SqlDialect::Generic).unwrap();
    assert_eq!(queries.len(), 1);
}

#[test]
fn test_clickhouse_simple_select() {
    let sql = "SELECT id, name FROM users WHERE id = 1";
    let queries = parse_queries(sql, SqlDialect::ClickHouse).unwrap();
    assert_eq!(queries.len(), 1);
    assert_eq!(queries[0].query_type, QueryType::Select);
}

#[test]
fn test_clickhouse_to_datetime_function() {
    let sql = "SELECT toDateTime(timestamp) FROM events";
    let queries = parse_queries(sql, SqlDialect::ClickHouse).unwrap();
    assert_eq!(queries.len(), 1);
    assert_eq!(queries[0].tables[0].as_str(), "events");
}

#[test]
fn test_clickhouse_to_start_of_interval() {
    let sql = "SELECT toStartOfInterval(Timestamp, INTERVAL 60 second) as time FROM logs";
    let queries = parse_queries(sql, SqlDialect::ClickHouse).unwrap();
    assert_eq!(queries.len(), 1);
    assert_eq!(queries[0].tables[0].as_str(), "logs");
}

#[test]
fn test_clickhouse_now_function() {
    let sql = "SELECT * FROM logs WHERE time >= NOW() - INTERVAL 1 HOUR";
    let queries = parse_queries(sql, SqlDialect::ClickHouse).unwrap();
    assert_eq!(queries.len(), 1);
}

#[test]
fn test_clickhouse_count_function() {
    let sql = "SELECT SeverityText, count() as count FROM logs GROUP BY SeverityText";
    let queries = parse_queries(sql, SqlDialect::ClickHouse).unwrap();
    assert_eq!(queries.len(), 1);
    assert!(
        queries[0]
            .group_cols
            .iter()
            .any(|c| c.as_str() == "SeverityText")
    );
}

#[test]
fn test_clickhouse_array_join() {
    let sql = "SELECT arrayJoin(arr) FROM test";
    let queries = parse_queries(sql, SqlDialect::ClickHouse).unwrap();
    assert_eq!(queries.len(), 1);
}

#[test]
fn test_clickhouse_if_function() {
    let sql = "SELECT if(status = 1, 'active', 'inactive') FROM users";
    let queries = parse_queries(sql, SqlDialect::ClickHouse).unwrap();
    assert_eq!(queries.len(), 1);
}

#[test]
fn test_clickhouse_format_datetime() {
    let sql = "SELECT formatDateTime(now(), '%Y-%m-%d') FROM system.one";
    let queries = parse_queries(sql, SqlDialect::ClickHouse).unwrap();
    assert_eq!(queries.len(), 1);
}
