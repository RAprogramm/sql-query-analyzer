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

#[test]
fn test_schema_debug() {
    let sql = "CREATE TABLE users (id INT PRIMARY KEY)";
    let schema = Schema::parse(sql).unwrap();
    let debug = format!("{:?}", schema);
    assert!(debug.contains("Schema"));
}

#[test]
fn test_schema_clone() {
    let sql = "CREATE TABLE users (id INT PRIMARY KEY)";
    let schema = Schema::parse(sql).unwrap();
    let cloned = schema.clone();
    assert_eq!(cloned.tables.len(), schema.tables.len());
}

#[test]
fn test_schema_default() {
    let schema = Schema::default();
    assert!(schema.tables.is_empty());
}

#[test]
fn test_parse_insert_statement() {
    let sql = r#"
        CREATE TABLE users (id INT PRIMARY KEY);
        INSERT INTO users VALUES (1);
    "#;
    let schema = Schema::parse(sql).unwrap();
    assert_eq!(schema.tables.len(), 1);
}

#[test]
fn test_parse_with_default_value() {
    let sql = "CREATE TABLE users (id INT DEFAULT 0, status VARCHAR(50) DEFAULT 'active')";
    let schema = Schema::parse(sql).unwrap();
    let users = &schema.tables["users"];
    assert_eq!(users.columns.len(), 2);
}

#[test]
fn test_schema_to_summary_with_index() {
    let sql = r#"
        CREATE TABLE users (id INT PRIMARY KEY, email VARCHAR(255));
        CREATE INDEX idx_email ON users(email);
    "#;
    let schema = Schema::parse(sql).unwrap();
    let summary = schema.to_summary();
    assert!(summary.contains("idx_email"));
    assert!(summary.contains("email"));
}

#[test]
fn test_schema_to_summary_with_unique_index() {
    let sql = r#"
        CREATE TABLE users (id INT PRIMARY KEY, email VARCHAR(255));
        CREATE UNIQUE INDEX idx_email ON users(email);
    "#;
    let schema = Schema::parse(sql).unwrap();
    let summary = schema.to_summary();
    assert!(summary.contains("UNIQUE"));
}

#[test]
fn test_parse_auto_increment() {
    let sql = "CREATE TABLE users (id INT AUTO_INCREMENT PRIMARY KEY)";
    let schema = Schema::parse(sql).unwrap();
    let users = &schema.tables["users"];
    assert_eq!(users.columns.len(), 1);
}

#[test]
fn test_parse_serial() {
    let sql = "CREATE TABLE users (id SERIAL PRIMARY KEY)";
    let schema = Schema::parse(sql).unwrap();
    let users = &schema.tables["users"];
    assert_eq!(users.columns.len(), 1);
}

#[test]
fn test_table_info_debug() {
    let sql = "CREATE TABLE users (id INT PRIMARY KEY)";
    let schema = Schema::parse(sql).unwrap();
    let users = &schema.tables["users"];
    let debug = format!("{:?}", users);
    assert!(debug.contains("TableInfo"));
}

#[test]
fn test_column_info_debug() {
    use sql_query_analyzer::schema::ColumnInfo;
    let col = ColumnInfo {
        name:        "test".to_string(),
        data_type:   "INT".to_string(),
        is_nullable: true,
        is_primary:  false,
        codec:       None
    };
    let debug = format!("{:?}", col);
    assert!(debug.contains("test"));
}

#[test]
fn test_column_info_codec_default_none() {
    let sql = "CREATE TABLE users (id INT PRIMARY KEY)";
    let schema = Schema::parse(sql).unwrap();
    let users = &schema.tables["users"];
    assert!(users.columns[0].codec.is_none());
}

#[test]
fn test_index_info_debug() {
    use sql_query_analyzer::schema::IndexInfo;
    let idx = IndexInfo {
        name:      "idx_test".to_string(),
        columns:   vec!["col1".to_string()],
        is_unique: false
    };
    let debug = format!("{:?}", idx);
    assert!(debug.contains("idx_test"));
}

#[test]
fn test_parse_nullable_column() {
    let sql = "CREATE TABLE users (id INT, name VARCHAR(255) NULL)";
    let schema = Schema::parse(sql).unwrap();
    let users = &schema.tables["users"];
    assert!(users.columns[1].is_nullable);
}

#[test]
fn test_table_info_clickhouse_fields_default_none() {
    let sql = "CREATE TABLE users (id INT PRIMARY KEY)";
    let schema = Schema::parse(sql).unwrap();
    let users = &schema.tables["users"];
    assert!(users.engine.is_none());
    assert!(users.order_by.is_none());
    assert!(users.primary_key.is_none());
    assert!(users.partition_by.is_none());
    assert!(users.cluster.is_none());
}
