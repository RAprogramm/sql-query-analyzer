//! Integration tests for the sql-query-analyzer binary.

use std::io::Write;

use assert_cmd::{Command, cargo::cargo_bin_cmd};
use predicates::prelude::*;
use tempfile::NamedTempFile;

fn cmd() -> Command {
    cargo_bin_cmd!("sql-query-analyzer")
}

#[test]
fn test_analyze_success() {
    let mut schema = NamedTempFile::new().unwrap();
    writeln!(schema, "CREATE TABLE users (id INT PRIMARY KEY);").unwrap();

    let mut queries = NamedTempFile::new().unwrap();
    writeln!(queries, "SELECT id FROM users;").unwrap();

    cmd()
        .args([
            "analyze",
            "-s",
            schema.path().to_str().unwrap(),
            "-q",
            queries.path().to_str().unwrap(),
            "--provider",
            "open-ai",
            "--no-color"
        ])
        .assert()
        .success();
}

#[test]
fn test_analyze_with_violations() {
    let mut schema = NamedTempFile::new().unwrap();
    writeln!(schema, "CREATE TABLE orders (id INT);").unwrap();

    let mut queries = NamedTempFile::new().unwrap();
    writeln!(queries, "SELECT * FROM orders;").unwrap();

    cmd()
        .args([
            "analyze",
            "-s",
            schema.path().to_str().unwrap(),
            "-q",
            queries.path().to_str().unwrap(),
            "--provider",
            "open-ai",
            "--no-color"
        ])
        .assert()
        .stdout(predicate::str::contains("STYLE001").or(predicate::str::contains("PERF")));
}

#[test]
fn test_analyze_file_not_found() {
    cmd()
        .args([
            "analyze",
            "-s",
            "/nonexistent/schema.sql",
            "-q",
            "/nonexistent/queries.sql",
            "--provider",
            "open-ai"
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Error"));
}

#[test]
fn test_analyze_dry_run() {
    let mut schema = NamedTempFile::new().unwrap();
    writeln!(schema, "CREATE TABLE t (id INT);").unwrap();

    let mut queries = NamedTempFile::new().unwrap();
    writeln!(queries, "SELECT id FROM t;").unwrap();

    cmd()
        .args([
            "analyze",
            "-s",
            schema.path().to_str().unwrap(),
            "-q",
            queries.path().to_str().unwrap(),
            "--provider",
            "open-ai",
            "--dry-run",
            "--no-color"
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("DRY RUN"));
}

#[test]
fn test_analyze_json_format() {
    let mut schema = NamedTempFile::new().unwrap();
    writeln!(schema, "CREATE TABLE items (id INT);").unwrap();

    let mut queries = NamedTempFile::new().unwrap();
    writeln!(queries, "SELECT id FROM items;").unwrap();

    cmd()
        .args([
            "analyze",
            "-s",
            schema.path().to_str().unwrap(),
            "-q",
            queries.path().to_str().unwrap(),
            "--provider",
            "open-ai",
            "-f",
            "json",
            "--no-color"
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("{"));
}

#[test]
fn test_analyze_yaml_format() {
    let mut schema = NamedTempFile::new().unwrap();
    writeln!(schema, "CREATE TABLE events (id INT);").unwrap();

    let mut queries = NamedTempFile::new().unwrap();
    writeln!(queries, "SELECT id FROM events;").unwrap();

    cmd()
        .args([
            "analyze",
            "-s",
            schema.path().to_str().unwrap(),
            "-q",
            queries.path().to_str().unwrap(),
            "--provider",
            "open-ai",
            "-f",
            "yaml",
            "--no-color"
        ])
        .assert()
        .success();
}

#[test]
fn test_analyze_sarif_format() {
    let mut schema = NamedTempFile::new().unwrap();
    writeln!(schema, "CREATE TABLE metrics (id INT);").unwrap();

    let mut queries = NamedTempFile::new().unwrap();
    writeln!(queries, "SELECT id FROM metrics;").unwrap();

    cmd()
        .args([
            "analyze",
            "-s",
            schema.path().to_str().unwrap(),
            "-q",
            queries.path().to_str().unwrap(),
            "--provider",
            "open-ai",
            "-f",
            "sarif",
            "--no-color"
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("$schema"));
}

#[test]
fn test_help() {
    cmd().arg("--help").assert().success();
}

#[test]
fn test_version() {
    cmd().arg("--version").assert().success();
}

#[test]
fn test_analyze_verbose() {
    let mut schema = NamedTempFile::new().unwrap();
    writeln!(schema, "CREATE TABLE logs (id INT);").unwrap();

    let mut queries = NamedTempFile::new().unwrap();
    writeln!(queries, "SELECT id FROM logs;").unwrap();

    cmd()
        .args([
            "analyze",
            "-s",
            schema.path().to_str().unwrap(),
            "-q",
            queries.path().to_str().unwrap(),
            "--provider",
            "open-ai",
            "--verbose",
            "--no-color"
        ])
        .assert()
        .success();
}

#[test]
fn test_analyze_mysql_dialect() {
    let mut schema = NamedTempFile::new().unwrap();
    writeln!(schema, "CREATE TABLE t (id INT PRIMARY KEY);").unwrap();

    let mut queries = NamedTempFile::new().unwrap();
    writeln!(queries, "SELECT id FROM t;").unwrap();

    cmd()
        .args([
            "analyze",
            "-s",
            schema.path().to_str().unwrap(),
            "-q",
            queries.path().to_str().unwrap(),
            "--provider",
            "open-ai",
            "--dialect",
            "mysql",
            "--no-color"
        ])
        .assert()
        .success();
}

#[test]
fn test_analyze_clickhouse_dialect() {
    let mut schema = NamedTempFile::new().unwrap();
    writeln!(
        schema,
        "CREATE TABLE t (id UInt64) ENGINE = MergeTree ORDER BY id;"
    )
    .unwrap();

    let mut queries = NamedTempFile::new().unwrap();
    writeln!(queries, "SELECT id FROM t;").unwrap();

    cmd()
        .args([
            "analyze",
            "-s",
            schema.path().to_str().unwrap(),
            "-q",
            queries.path().to_str().unwrap(),
            "--provider",
            "open-ai",
            "--dialect",
            "clickhouse",
            "--no-color"
        ])
        .assert()
        .success();
}
