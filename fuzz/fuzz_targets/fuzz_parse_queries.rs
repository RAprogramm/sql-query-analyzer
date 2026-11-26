#![no_main]

//! Fuzzes SQL query parsing for crashes on attacker-controlled input.

use libfuzzer_sys::fuzz_target;
use sql_query_analyzer::query::parse_queries;
use sql_query_analyzer::query::SqlDialect;

fuzz_target!(|data: &[u8]| {
    if let Ok(sql) = std::str::from_utf8(data) {
        for dialect in [
            SqlDialect::Generic,
            SqlDialect::ClickHouse,
            SqlDialect::MySQL,
            SqlDialect::PostgreSQL,
            SqlDialect::SQLite,
        ] {
            let _ = parse_queries(sql, dialect);
        }
    }
});
