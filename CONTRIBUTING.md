<!--
SPDX-FileCopyrightText: 2025 RAprogramm <andrey.rozanov.vl@gmail.com>
SPDX-License-Identifier: MIT
-->

# Contributing to SQL Query Analyzer

Thank you for your interest in contributing to this project!

## PR Size Limits

This project enforces PR size limits using [rust-prod-diff-checker](https://github.com/RAprogramm/rust-prod-diff-checker).

**Maximum 200 lines of production code per PR.**

- Only production code counts (src/*.rs)
- Tests, benchmarks, examples are excluded
- Documentation changes don't count

Large PRs are harder to review and more likely to contain bugs. If your change exceeds 200 lines, split it into smaller PRs.

## Code Style & Standards

This project follows the [RustManifest](https://github.com/RAprogramm/RustManifest) coding standards. Please read it thoroughly before contributing.

Key points:
- Use `cargo +nightly fmt` for formatting
- No `unwrap()` or `expect()` in production code
- Documentation via Rustdoc only (no inline comments)
- Descriptive naming conventions

## Development Setup

### Prerequisites

- Rust nightly toolchain
- cargo-nextest (for running tests)

### Installation

```bash
git clone https://github.com/RAprogramm/sql-query-analyzer
cd sql-query-analyzer

# Install nightly toolchain
rustup toolchain install nightly
rustup component add rustfmt --toolchain nightly
rustup component add clippy

# Install test runner (optional but recommended)
cargo install cargo-nextest
```

### Pre-commit Checks

Before committing, ensure all checks pass:

```bash
# Format check
cargo +nightly fmt --all -- --check

# Linting
cargo clippy --all-targets --all-features -- -D warnings

# Tests
cargo test --all-features

# Or with nextest
cargo nextest run --all-features
```

## Git Workflow

### Branch Naming

Use issue number as branch name:
```
123
```

### Commit Messages

Format: `#<issue_number> <type>: <description>`

```
#123 feat: add new rule for index suggestions
#123 fix: correct SQL parsing for subqueries
#45 docs: update API examples
#78 test: add tests for schema validation
#90 refactor: simplify rule execution
```

Types:
- `feat` - new feature
- `fix` - bug fix
- `docs` - documentation
- `test` - tests
- `refactor` - code refactoring
- `chore` - maintenance tasks

### Pull Requests

1. Create branch from `main`
2. Make your changes
3. Ensure all CI checks pass
4. Keep PR under 200 lines of production code
5. Create PR with descriptive title
6. Include `Closes #<issue>` in description

## Testing

### Test Organization

```
tests/
├── query_tests.rs    # Query parsing tests
├── schema_tests.rs   # Schema parsing tests
├── rules_tests.rs    # Rule execution tests
└── config_tests.rs   # Configuration tests
```

### Writing Tests

- Cover all public API functions
- Test error paths, not just happy paths
- No `unwrap()` in tests - use `?` with proper error types

```rust
#[test]
fn test_parse_query() -> Result<(), Box<dyn std::error::Error>> {
    let queries = parse_queries("SELECT * FROM users", SqlDialect::Generic)?;

    assert_eq!(queries.len(), 1);
    Ok(())
}
```

### Running Tests

```bash
# All tests
cargo test --all-features

# With coverage
cargo llvm-cov nextest --all-features

# Specific test
cargo test test_parse_query
```

## CI/CD Pipeline

### Automated Checks

Every PR triggers:

| Job | Description |
|-----|-------------|
| PR Size | Max 200 lines production code |
| Format | `cargo +nightly fmt --check` |
| Clippy | `cargo clippy -D warnings` |
| Test | `cargo test --all-features` |
| Doc | `cargo doc --no-deps` |
| Coverage | Upload to Codecov |
| Audit | Security vulnerability scan |
| REUSE | License compliance |

### Coverage Requirements

- Project target: auto (maintain current level)
- Patch target: 80% (new code must be well-tested)

## Architecture

### Module Structure

```
src/
├── lib.rs        # Public API exports
├── main.rs       # CLI entry point
├── cli.rs        # CLI argument parsing
├── config.rs     # Configuration handling
├── error.rs      # Error types
├── query.rs      # SQL query parsing
├── schema.rs     # Schema parsing
├── rules/        # Analysis rules
│   ├── mod.rs
│   ├── performance.rs
│   ├── security.rs
│   ├── style.rs
│   └── schema.rs
├── output.rs     # Output formatting
├── llm.rs        # LLM integration
└── cache.rs      # Query caching
```

### Key Types

- `ParsedQuery` - Parsed SQL query with metadata
- `SchemaInfo` - Database schema information
- `Violation` - A rule violation with severity
- `RuleRunner` - Executes rules in parallel

## Adding Features

### New Rule

1. Determine category (performance, security, style, schema)
2. Add to appropriate file in `src/rules/`
3. Implement `Rule` trait
4. Register in `RuleRunner`
5. Add tests and documentation
6. Update README rule table

### New Output Format

1. Add variant to output format enum
2. Implement formatting in `src/output.rs`
3. Add CLI flag
4. Add tests and documentation

## Release Process

Releases are automated via CI on tag push:

1. Update version in `Cargo.toml`
2. Commit: `chore(release): prepare v0.x.x`
3. Create and push tag:
   ```bash
   git tag v0.x.x
   git push origin v0.x.x
   ```
4. CI builds binaries for all platforms
5. GitHub Release is created automatically
6. Published to crates.io
7. Changelog is updated

### Versioning

Follow [Semantic Versioning](https://semver.org/):
- MAJOR: Breaking API changes
- MINOR: New features, backward compatible
- PATCH: Bug fixes

## Documentation

### Code Documentation

All public items must have Rustdoc:

```rust
/// Parses SQL queries from a string.
///
/// # Errors
///
/// Returns `AppError::ParseError` if the SQL is invalid.
///
/// # Examples
///
/// ```
/// use sql_query_analyzer::query::parse_queries;
///
/// let queries = parse_queries("SELECT * FROM users", SqlDialect::Generic)?;
/// # Ok::<(), sql_query_analyzer::AppError>(())
/// ```
pub fn parse_queries(input: &str, dialect: SqlDialect) -> Result<Vec<ParsedQuery>, AppError> {
    // ...
}
```

### README Updates

Update README.md when:
- Adding new rules
- Adding new CLI options
- Changing configuration format
- Adding new output formats

## Getting Help

- Open an issue for bugs or feature requests
- Check existing issues before creating new ones
- Provide minimal reproduction for bugs

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
