// SPDX-FileCopyrightText: 2025 RAprogramm
// SPDX-License-Identifier: MIT

use sql_query_analyzer::schema::Schema;

#[test]
fn test_parse_simple_table() {
    let sql = "CREATE TABLE users (id INT PRIMARY KEY, name VARCHAR(255))";
    let schema = Schema::parse(sql).unwrap();
    assert_eq!(schema.tables.len(), 1);
    assert!(schema.tables.contains_key("users"));
    let users = &schema.tables["users"];
    assert_eq!(users.columns.len(), 2);
    assert_eq!(users.columns[0].name, "id");
    assert!(users.columns[0].is_primary);
}

#[test]
fn test_parse_multiple_tables() {
    let sql = r#"
        CREATE TABLE users (id INT PRIMARY KEY);
        CREATE TABLE orders (id INT PRIMARY KEY, user_id INT);
    "#;
    let schema = Schema::parse(sql).unwrap();
    assert_eq!(schema.tables.len(), 2);
    assert!(schema.tables.contains_key("users"));
    assert!(schema.tables.contains_key("orders"));
}

#[test]
fn test_parse_not_null() {
    let sql = "CREATE TABLE users (id INT NOT NULL, name VARCHAR(255))";
    let schema = Schema::parse(sql).unwrap();
    let users = &schema.tables["users"];
    assert!(!users.columns[0].is_nullable);
    assert!(users.columns[1].is_nullable);
}

#[test]
fn test_parse_index() {
    let sql = r#"
        CREATE TABLE users (id INT PRIMARY KEY, email VARCHAR(255));
        CREATE INDEX idx_email ON users(email);
    "#;
    let schema = Schema::parse(sql).unwrap();
    let users = &schema.tables["users"];
    assert_eq!(users.indexes.len(), 1);
    assert_eq!(users.indexes[0].columns[0], "email");
}

#[test]
fn test_parse_unique_index() {
    let sql = r#"
        CREATE TABLE users (id INT PRIMARY KEY, email VARCHAR(255));
        CREATE UNIQUE INDEX idx_email ON users(email);
    "#;
    let schema = Schema::parse(sql).unwrap();
    let users = &schema.tables["users"];
    assert!(users.indexes[0].is_unique);
}

#[test]
fn test_parse_composite_index() {
    let sql = r#"
        CREATE TABLE orders (id INT, user_id INT, created_at TIMESTAMP);
        CREATE INDEX idx_user_created ON orders(user_id, created_at);
    "#;
    let schema = Schema::parse(sql).unwrap();
    let orders = &schema.tables["orders"];
    assert_eq!(orders.indexes[0].columns.len(), 2);
}

#[test]
fn test_to_summary() {
    let sql = "CREATE TABLE users (id INT PRIMARY KEY, name VARCHAR(255) NOT NULL)";
    let schema = Schema::parse(sql).unwrap();
    let summary = schema.to_summary();
    assert!(summary.contains("users"));
    assert!(summary.contains("id"));
    assert!(summary.contains("name"));
    assert!(summary.contains("PRIMARY KEY"));
    assert!(summary.contains("NOT NULL"));
}

#[test]
fn test_parse_various_types() {
    let sql = r#"
        CREATE TABLE test (
            id BIGINT,
            price DECIMAL(10,2),
            active BOOLEAN,
            data TEXT,
            created_at TIMESTAMP
        )
    "#;
    let schema = Schema::parse(sql).unwrap();
    let test = &schema.tables["test"];
    assert_eq!(test.columns.len(), 5);
}

#[test]
fn test_parse_invalid_schema() {
    let sql = "CREATE TABEL users (id INT)";
    let result = Schema::parse(sql);
    assert!(result.is_err());
}

#[test]
fn test_empty_schema() {
    let sql = "";
    let schema = Schema::parse(sql).unwrap();
    assert!(schema.tables.is_empty());
}
