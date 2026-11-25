#![no_main]

//! Fuzzes the dialect-aware SQL preprocessor for crashes and unbounded
//! resource growth on attacker-controlled SQL.

use libfuzzer_sys::fuzz_target;
use sql_query_analyzer::preprocessor::Preprocessor;
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
            let preprocessor = Preprocessor::new(dialect);
            let _ = preprocessor.process(sql);
        }
    }
});
