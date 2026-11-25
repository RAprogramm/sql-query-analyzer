// SPDX-FileCopyrightText: 2025 RAprogramm
// SPDX-License-Identifier: MIT

//! Benchmarks for the static analysis pipeline: schema parsing, query
//! parsing, rule execution and report formatting.

use divan::{Bencher, black_box};
use sql_query_analyzer::{
    config::RulesConfig,
    output::{OutputFormat, OutputOptions, format_static_analysis},
    preprocessor::Preprocessor,
    query::{Query, SqlDialect, parse_queries},
    rules::{AnalysisReport, RuleRunner},
    schema::Schema
};

/// Runs rayon on a single thread so results measure CPU work, not thread
/// scheduling, which makes CPU simulation results unstable between runs.
fn main() {
    rayon::ThreadPoolBuilder::new()
        .num_threads(1)
        .build_global()
        .unwrap();
    divan::main();
}

/// Number of queries in each generated workload.
const SIZES: &[usize] = &[10, 100];

/// Number of tables in the generated schema.
const SCHEMA_TABLES: usize = 20;

const QUERY_TEMPLATES: &[&str] = &[
    "SELECT * FROM users{n} WHERE email LIKE '%@example.com'",
    "SELECT u.id, u.name, o.total FROM users{n} u JOIN orders{n} o ON u.id = o.user_id WHERE o.status = 'paid' ORDER BY o.created_at DESC LIMIT 50",
    "SELECT id, name FROM users{n} WHERE status = 'active' OR status = 'pending' OR status = 'trial'",
    "SELECT * FROM orders{n} ORDER BY created_at LIMIT 20 OFFSET 5000",
    "SELECT DISTINCT user_id FROM orders{n} WHERE total > 100 ORDER BY user_id",
    "SELECT id, (SELECT COUNT(*) FROM orders{n} o WHERE o.user_id = u.id) AS cnt FROM users{n} u",
    "SELECT id FROM users{n} WHERE LOWER(email) = 'a@b.c' AND YEAR(created_at) = 2024",
    "SELECT id FROM users{n} WHERE id NOT IN (SELECT user_id FROM orders{n})",
    "SELECT id FROM users{n} UNION SELECT user_id FROM orders{n}",
    "SELECT COUNT(*) FROM orders{n}",
    "SELECT user_id, SUM(total) FROM orders{n} GROUP BY user_id HAVING SUM(total) > 1000",
    "SELECT * FROM products{n} ORDER BY RANDOM() LIMIT 1",
    "SELECT id FROM products{n} WHERE category_id IN (1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12)",
    "WITH recent AS (SELECT user_id, MAX(created_at) AS last FROM orders{n} GROUP BY user_id) SELECT u.name, r.last FROM users{n} u JOIN recent r ON r.user_id = u.id WHERE u.status = 'active'",
    "INSERT INTO orders{n} (user_id, total, status) VALUES (1, 10.5, 'new')",
    "UPDATE users{n} SET status = 'inactive' WHERE last_login < '2024-01-01'",
    "UPDATE products{n} SET price = price * 1.1",
    "DELETE FROM orders{n} WHERE status = 'cancelled' AND created_at < '2023-01-01'",
    "SELECT id FROM users{n} WHERE name = 'admin' OR 1 = 1",
    "SELECT o.id FROM orders{n} o, users{n} u WHERE o.total > 10"
];

fn queries_sql(count: usize) -> String {
    (0..count)
        .map(|i| {
            let template = QUERY_TEMPLATES[i % QUERY_TEMPLATES.len()];
            let table_suffix = (i / QUERY_TEMPLATES.len()) % SCHEMA_TABLES;
            format!("{};\n", template.replace("{n}", &table_suffix.to_string()))
        })
        .collect()
}

fn schema_sql(tables: usize) -> String {
    let mut sql = String::new();
    for n in 0..tables {
        sql.push_str(&format!(
            "CREATE TABLE users{n} (id INT PRIMARY KEY, name VARCHAR(255) NOT NULL, email VARCHAR(255) NOT NULL, status VARCHAR(32), created_at TIMESTAMP, last_login TIMESTAMP);\n\
             CREATE TABLE orders{n} (id INT PRIMARY KEY, user_id INT NOT NULL, total DECIMAL(10, 2), status VARCHAR(32), created_at TIMESTAMP);\n\
             CREATE TABLE products{n} (id INT PRIMARY KEY, name VARCHAR(255), price DECIMAL(10, 2), category_id INT);\n\
             CREATE UNIQUE INDEX idx_users{n}_email ON users{n}(email);\n\
             CREATE INDEX idx_orders{n}_user ON orders{n}(user_id, created_at);\n"
        ));
    }
    sql
}

fn clickhouse_schema_sql(tables: usize) -> String {
    (0..tables)
        .map(|n| {
            format!(
                "CREATE TABLE events{n} (id UInt64, user_id UInt64, ts DateTime CODEC(Delta, ZSTD), payload String CODEC(ZSTD(3))) \
                 ENGINE = MergeTree PARTITION BY toYYYYMM(ts) ORDER BY (user_id, ts);\n\
                 CREATE TABLE metrics{n} (name LowCardinality(String), ts DateTime CODEC(DoubleDelta), value Float64 CODEC(Gorilla, LZ4)) \
                 ENGINE = MergeTree ORDER BY (name, ts) SETTINGS index_granularity = 8192, ttl_only_drop_parts = 1;\n"
            )
        })
        .collect()
}

fn parsed_schema() -> Schema {
    Schema::parse(&schema_sql(SCHEMA_TABLES), SqlDialect::Generic).unwrap()
}

fn parsed_queries(count: usize) -> Vec<Query> {
    parse_queries(&queries_sql(count), SqlDialect::Generic).unwrap()
}

fn report(count: usize) -> AnalysisReport {
    let runner = RuleRunner::with_schema_and_config(parsed_schema(), RulesConfig::default());
    runner.analyze(&parsed_queries(count))
}

fn output_options(format: OutputFormat) -> OutputOptions {
    OutputOptions {
        format,
        colored: false,
        verbose: true
    }
}

#[divan::bench(args = SIZES)]
fn parse_queries_generic(bencher: Bencher, count: usize) {
    let sql = queries_sql(count);
    bencher.bench(|| parse_queries(black_box(&sql), SqlDialect::Generic).unwrap());
}

#[divan::bench(args = SIZES)]
fn parse_queries_postgres(bencher: Bencher, count: usize) {
    let sql = queries_sql(count);
    bencher.bench(|| parse_queries(black_box(&sql), SqlDialect::PostgreSQL).unwrap());
}

#[divan::bench]
fn parse_schema_generic(bencher: Bencher) {
    let sql = schema_sql(SCHEMA_TABLES);
    bencher.bench(|| Schema::parse(black_box(&sql), SqlDialect::Generic).unwrap());
}

#[divan::bench]
fn parse_schema_clickhouse(bencher: Bencher) {
    let sql = clickhouse_schema_sql(SCHEMA_TABLES);
    bencher.bench(|| Schema::parse(black_box(&sql), SqlDialect::ClickHouse).unwrap());
}

#[divan::bench]
fn preprocess_clickhouse(bencher: Bencher) {
    let sql = clickhouse_schema_sql(SCHEMA_TABLES);
    let preprocessor = Preprocessor::new(SqlDialect::ClickHouse);
    bencher.bench(|| preprocessor.process(black_box(&sql)));
}

#[divan::bench(args = SIZES)]
fn analyze_rules(bencher: Bencher, count: usize) {
    let runner = RuleRunner::new();
    let queries = parsed_queries(count);
    bencher.bench(|| runner.analyze(black_box(&queries)));
}

#[divan::bench(args = SIZES)]
fn analyze_rules_with_schema(bencher: Bencher, count: usize) {
    let runner = RuleRunner::with_schema_and_config(parsed_schema(), RulesConfig::default());
    let queries = parsed_queries(count);
    bencher.bench(|| runner.analyze(black_box(&queries)));
}

#[divan::bench]
fn build_rule_runner_with_schema(bencher: Bencher) {
    let schema = parsed_schema();
    bencher.bench(|| {
        RuleRunner::with_schema_and_config(black_box(schema.clone()), RulesConfig::default())
    });
}

#[divan::bench(args = ["text", "json", "sarif"])]
fn format_report(bencher: Bencher, format: &str) {
    let format = match format {
        "json" => OutputFormat::Json,
        "sarif" => OutputFormat::Sarif,
        _ => OutputFormat::Text
    };
    let report = report(100);
    let opts = output_options(format);
    bencher.bench(|| format_static_analysis(black_box(&report), &opts));
}

/// Static analysis as run by the CLI, without file I/O and LLM calls.
#[divan::bench(args = SIZES)]
fn full_static_pipeline(bencher: Bencher, count: usize) {
    let schema = schema_sql(SCHEMA_TABLES);
    let queries = queries_sql(count);
    let opts = output_options(OutputFormat::Text);
    bencher.bench(|| {
        let schema = Schema::parse(black_box(&schema), SqlDialect::Generic).unwrap();
        let queries = parse_queries(black_box(&queries), SqlDialect::Generic).unwrap();
        let runner = RuleRunner::with_schema_and_config(schema, RulesConfig::default());
        let report = runner.analyze(&queries);
        format_static_analysis(&report, &opts)
    });
}
