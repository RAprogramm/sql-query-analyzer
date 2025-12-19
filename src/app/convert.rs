//! Type conversion functions for CLI to internal types.
//!
//! This module provides conversion functions that translate CLI-facing
//! types (from the `cli` module) to internal domain types used by the
//! analysis engine.

use crate::{
    cli::{Dialect, Format},
    output::OutputFormat,
    query::SqlDialect
};

/// Converts a CLI dialect enum to the internal SQL dialect type.
///
/// Maps the user-facing dialect names to the corresponding `sqlparser`
/// dialect implementations used for parsing SQL statements.
///
/// # Arguments
///
/// * `dialect` - The CLI dialect enum value from command-line arguments
///
/// # Returns
///
/// The corresponding internal `SqlDialect` enum variant.
///
/// # Example
///
/// ```
/// use sql_query_analyzer::{app::convert_dialect, cli::Dialect, query::SqlDialect};
///
/// let dialect = convert_dialect(Dialect::Mysql);
/// assert!(matches!(dialect, SqlDialect::MySQL));
/// ```
pub fn convert_dialect(dialect: Dialect) -> SqlDialect {
    match dialect {
        Dialect::Generic => SqlDialect::Generic,
        Dialect::Mysql => SqlDialect::MySQL,
        Dialect::Postgresql => SqlDialect::PostgreSQL,
        Dialect::Sqlite => SqlDialect::SQLite,
        Dialect::Clickhouse => SqlDialect::ClickHouse
    }
}

/// Converts a CLI format enum to the internal output format type.
///
/// Maps the user-facing format names to the corresponding output
/// formatter implementations.
///
/// # Arguments
///
/// * `format` - The CLI format enum value from command-line arguments
///
/// # Returns
///
/// The corresponding internal `OutputFormat` enum variant.
///
/// # Example
///
/// ```
/// use sql_query_analyzer::{app::convert_format, cli::Format, output::OutputFormat};
///
/// let format = convert_format(Format::Json);
/// assert!(matches!(format, OutputFormat::Json));
/// ```
pub fn convert_format(format: Format) -> OutputFormat {
    match format {
        Format::Text => OutputFormat::Text,
        Format::Json => OutputFormat::Json,
        Format::Yaml => OutputFormat::Yaml,
        Format::Sarif => OutputFormat::Sarif
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_dialect_generic() {
        assert!(matches!(
            convert_dialect(Dialect::Generic),
            SqlDialect::Generic
        ));
    }

    #[test]
    fn test_convert_dialect_mysql() {
        assert!(matches!(convert_dialect(Dialect::Mysql), SqlDialect::MySQL));
    }

    #[test]
    fn test_convert_dialect_postgresql() {
        assert!(matches!(
            convert_dialect(Dialect::Postgresql),
            SqlDialect::PostgreSQL
        ));
    }

    #[test]
    fn test_convert_dialect_sqlite() {
        assert!(matches!(
            convert_dialect(Dialect::Sqlite),
            SqlDialect::SQLite
        ));
    }

    #[test]
    fn test_convert_dialect_clickhouse() {
        assert!(matches!(
            convert_dialect(Dialect::Clickhouse),
            SqlDialect::ClickHouse
        ));
    }

    #[test]
    fn test_convert_format_text() {
        assert!(matches!(convert_format(Format::Text), OutputFormat::Text));
    }

    #[test]
    fn test_convert_format_json() {
        assert!(matches!(convert_format(Format::Json), OutputFormat::Json));
    }

    #[test]
    fn test_convert_format_yaml() {
        assert!(matches!(convert_format(Format::Yaml), OutputFormat::Yaml));
    }

    #[test]
    fn test_convert_format_sarif() {
        assert!(matches!(convert_format(Format::Sarif), OutputFormat::Sarif));
    }
}
