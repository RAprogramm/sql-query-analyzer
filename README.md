# SQL Query Analyzer

[![Crates.io](https://img.shields.io/crates/v/sql-query-analyzer.svg)](https://crates.io/crates/sql-query-analyzer)
[![Docs.rs](https://docs.rs/sql-query-analyzer/badge.svg)](https://docs.rs/sql-query-analyzer)
[![CI](https://github.com/RAprogramm/sql-query-analyzer/actions/workflows/ci.yml/badge.svg)](https://github.com/RAprogramm/sql-query-analyzer/actions/workflows/ci.yml)
[![codecov](https://codecov.io/gh/RAprogramm/sql-query-analyzer/graph/badge.svg?token=hKvq66JThf)](https://codecov.io/gh/RAprogramm/sql-query-analyzer)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Hits-of-Code](https://hitsofcode.com/github/RAprogramm/sql-query-analyzer?branch=main&exclude=Cargo.lock,.gitignore,CHANGELOG.md)](https://hitsofcode.com/github/RAprogramm/sql-query-analyzer/view?branch=main&exclude=Cargo.lock,.gitignore,CHANGELOG.md)
[![REUSE status](https://api.reuse.software/badge/github.com/RAprogramm/sql-query-analyzer)](https://api.reuse.software/info/github.com/RAprogramm/sql-query-analyzer)

**Static analysis and LLM-powered optimization for SQL queries.**

A comprehensive SQL analysis tool that combines fast, deterministic static analysis with optional AI-powered insights. Identifies performance issues, style violations, and security vulnerabilities in your SQL queries.

## Table of Contents

- [Highlights](#highlights)
- [Installation](#installation)
- [Quick Start](#quick-start)
- [Rules](#rules)
- [Configuration](#configuration)
- [CLI Reference](#cli-reference)
- [Example](#example)
- [CI/CD Integration](#cicd-integration)
- [LLM Providers](#llm-providers)
- [Architecture](#architecture)
- [Performance](#performance)
- [Contributing](#contributing)
- [Acknowledgements](#acknowledgements)
- [Coverage](#coverage)
- [License](#license)

## Highlights

- **18 Built-in Rules** — Performance, style, and security checks run instantly without API calls
- **Schema-Aware Analysis** — Validates queries against your database schema, suggests missing indexes
- **Multiple Output Formats** — Text, JSON, YAML, and SARIF for CI/CD integration
- **Parallel Execution** — Rules execute concurrently using [rayon](https://github.com/rayon-rs/rayon)
- **Optional LLM Analysis** — Deep semantic analysis via OpenAI, Anthropic, or local Ollama
- **Configurable** — Disable rules, override severity levels, customize via TOML

<div align="right"><a href="#table-of-contents">↑ Back to top</a></div>

## Installation

### From source

```bash
cargo install --path .
```

### Pre-built binaries

Download from [Releases](https://github.com/RAprogramm/sql-query-analyzer/releases).

<div align="right"><a href="#table-of-contents">↑ Back to top</a></div>

## Quick Start

```bash
# Run static analysis (no API key required)
sql-query-analyzer analyze -s schema.sql -q queries.sql

# Output as SARIF for CI/CD
sql-query-analyzer analyze -s schema.sql -q queries.sql -f sarif > results.sarif

# Pipe queries from stdin
echo "SELECT * FROM users" | sql-query-analyzer analyze -s schema.sql -q -

# Enable LLM analysis
export LLM_API_KEY="sk-..."
sql-query-analyzer analyze -s schema.sql -q queries.sql --provider openai
```

<div align="right"><a href="#table-of-contents">↑ Back to top</a></div>

## Rules

### Performance Rules

| ID | Rule | Severity | Description |
|----|------|----------|-------------|
| `PERF001` | Select star without limit | Warning | `SELECT *` without `LIMIT` can return unbounded rows |
| `PERF002` | Leading wildcard | Warning | `LIKE '%value'` prevents index usage |
| `PERF003` | OR instead of IN | Info | Multiple `OR` conditions can be simplified to `IN` |
| `PERF004` | Large offset | Warning | `OFFSET > 1000` causes performance degradation |
| `PERF005` | Missing join condition | Error | Cartesian product detected |
| `PERF006` | Distinct with order by | Info | Potentially redundant operations |
| `PERF007` | Scalar subquery | Warning | N+1 query pattern detected |
| `PERF008` | Function on column | Warning | Function calls prevent index usage |
| `PERF009` | NOT IN with subquery | Warning | Can cause unexpected NULL behavior |
| `PERF010` | UNION without ALL | Info | Unnecessary deduplication overhead |
| `PERF011` | Select without where | Info | Full table scan on large tables |

### Style Rules

| ID | Rule | Severity | Description |
|----|------|----------|-------------|
| `STYLE001` | Select star | Info | Explicit column list preferred |
| `STYLE002` | Missing table alias | Info | Multi-table queries should use aliases |

### Security Rules

| ID | Rule | Severity | Description |
|----|------|----------|-------------|
| `SEC001` | Missing WHERE in UPDATE | Error | Potentially dangerous bulk update |
| `SEC002` | Missing WHERE in DELETE | Error | Potentially dangerous bulk delete |

### Schema-Aware Rules

| ID | Rule | Severity | Description |
|----|------|----------|-------------|
| `SCHEMA001` | Missing index on filter | Warning | WHERE/JOIN column lacks index |
| `SCHEMA002` | Column not in schema | Warning | Referenced column doesn't exist |
| `SCHEMA003` | Index suggestion | Info | ORDER BY column could benefit from index |

<div align="right"><a href="#table-of-contents">↑ Back to top</a></div>

## Configuration

Configuration is loaded from (in order of precedence):

1. Command-line arguments
2. Environment variables
3. `.sql-analyzer.toml` in current directory
4. `~/.config/sql-analyzer/config.toml`

### Example Configuration

```toml
[rules]
# Disable specific rules by ID
disabled = ["STYLE001", "PERF011"]

# Override default severity levels
[rules.severity]
PERF001 = "error"      # Promote to error
SCHEMA001 = "info"     # Demote to info

[llm]
provider = "ollama"
model = "codellama"
ollama_url = "http://localhost:11434"

[retry]
max_retries = 3
initial_delay_ms = 1000
max_delay_ms = 30000
backoff_factor = 2.0
```

### Environment Variables

| Variable | Description |
|----------|-------------|
| `LLM_API_KEY` | API key for OpenAI/Anthropic |
| `LLM_PROVIDER` | Provider name (openai, anthropic, ollama) |
| `LLM_MODEL` | Model identifier |
| `OLLAMA_URL` | Ollama base URL |

<div align="right"><a href="#table-of-contents">↑ Back to top</a></div>

## CLI Reference

```
sql-query-analyzer analyze [OPTIONS] -s <SCHEMA> -q <QUERIES>
```

### Options

| Flag | Description | Default |
|------|-------------|---------|
| `-s, --schema <FILE>` | Path to SQL schema file | required |
| `-q, --queries <FILE>` | Path to SQL queries file (use `-` for stdin) | required |
| `-p, --provider <PROVIDER>` | LLM provider: `openai`, `anthropic`, `ollama` | `ollama` |
| `-a, --api-key <KEY>` | API key (or use `LLM_API_KEY` env) | - |
| `-m, --model <MODEL>` | Model name | provider default |
| `--ollama-url <URL>` | Ollama base URL | `http://localhost:11434` |
| `--dialect <DIALECT>` | SQL dialect: `generic`, `mysql`, `postgresql`, `sqlite` | `generic` |
| `-f, --output-format <FMT>` | Output: `text`, `json`, `yaml`, `sarif` | `text` |
| `-v, --verbose` | Show complexity scores | false |
| `--dry-run` | Show what would be sent to LLM | false |
| `--no-color` | Disable colored output | false |

### Exit Codes

| Code | Meaning |
|------|---------|
| `0` | Success, no issues or only informational |
| `1` | Warnings found |
| `2` | Errors found |

<div align="right"><a href="#table-of-contents">↑ Back to top</a></div>

## Example

**schema.sql:**
```sql
CREATE TABLE users (
    id INT PRIMARY KEY,
    email VARCHAR(255) NOT NULL,
    created_at TIMESTAMP
);

CREATE TABLE orders (
    id INT PRIMARY KEY,
    user_id INT NOT NULL,
    total DECIMAL(10,2),
    status VARCHAR(20)
);

CREATE INDEX idx_orders_user ON orders(user_id);
```

**queries.sql:**
```sql
SELECT * FROM users WHERE email = 'test@example.com';
SELECT * FROM orders WHERE user_id = 1 ORDER BY created_at DESC;
DELETE FROM users;
```

**Output:**
```
=== Static Analysis ===
Found 1 error(s), 2 warning(s), 1 info

Query #1:
  [ERROR] SEC002: DELETE without WHERE clause is dangerous
         → Add WHERE clause to limit affected rows
  [ WARN] SCHEMA001: Column 'email' in WHERE clause has no index
         → Consider adding index on 'email'

Query #2:
  [ WARN] SCHEMA001: Column 'created_at' in ORDER BY has no index
         → Consider adding index on 'created_at'
  [ INFO] SCHEMA003: ORDER BY column 'created_at' could benefit from index
         → CREATE INDEX idx_created_at ON table(created_at)
```

<div align="right"><a href="#table-of-contents">↑ Back to top</a></div>

## CI/CD Integration

### GitHub Action

The easiest way to integrate SQL Query Analyzer into your CI/CD pipeline.

```yaml
name: SQL Analysis

on:
  pull_request:
    paths:
      - '**/*.sql'

jobs:
  analyze:
    runs-on: ubuntu-latest
    permissions:
      contents: read
      pull-requests: write
      security-events: write
    steps:
      - uses: actions/checkout@v4

      - uses: RAprogramm/sql-query-analyzer@v1
        with:
          schema: db/schema.sql
          queries: db/queries.sql
          upload-sarif: 'true'
          post-comment: 'true'
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
```

#### Action Inputs

| Input | Description | Default |
|-------|-------------|---------|
| `schema` | Path to SQL schema file | required |
| `queries` | Path to SQL queries file | required |
| `dialect` | SQL dialect (generic, mysql, postgresql, sqlite) | `generic` |
| `format` | Output format (text, json, yaml, sarif) | `text` |
| `fail-on-warning` | Fail if warnings are found | `false` |
| `fail-on-error` | Fail if errors are found | `true` |
| `upload-sarif` | Upload SARIF to GitHub Security tab | `false` |
| `post-comment` | Post analysis as PR comment | `false` |

#### Action Outputs

| Output | Description |
|--------|-------------|
| `analysis` | Full analysis result |
| `error-count` | Number of errors found |
| `warning-count` | Number of warnings found |
| `exit-code` | Exit code (0=ok, 1=warnings, 2=errors) |

#### Static Analysis Only (No LLM)

For fast CI checks without external API calls:

```yaml
- uses: RAprogramm/sql-query-analyzer@v1
  with:
    schema: db/schema.sql
    queries: db/queries.sql
    fail-on-error: 'true'
    post-comment: 'true'
  env:
    GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
```

This runs all 18 built-in rules instantly without requiring any API keys.

#### Advanced Usage

```yaml
- uses: RAprogramm/sql-query-analyzer@v1
  id: sql-analysis
  with:
    schema: db/schema.sql
    queries: db/queries.sql
    dialect: postgresql
    format: sarif
    fail-on-warning: 'true'
    upload-sarif: 'true'
    post-comment: 'true'
  env:
    GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}

- name: Check results
  if: steps.sql-analysis.outputs.error-count > 0
  run: echo "Found ${{ steps.sql-analysis.outputs.error-count }} errors"
```

### Manual Installation

For environments where the action is not available:

```yaml
jobs:
  analyze:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install sql-query-analyzer
        run: cargo install sql_query_analyzer

      - name: Analyze SQL
        run: |
          sql_query_analyzer analyze \
            -s db/schema.sql \
            -q db/queries.sql \
            -f sarif > results.sarif

      - name: Upload SARIF
        uses: github/codeql-action/upload-sarif@v3
        with:
          sarif_file: results.sarif
```

### GitLab CI

```yaml
sql-analysis:
  stage: test
  script:
    - cargo install sql-query-analyzer
    - sql-query-analyzer analyze -s schema.sql -q queries.sql -f sarif > gl-sast-report.json
  artifacts:
    reports:
      sast: gl-sast-report.json
```

### Pre-commit Hook

```yaml
# .pre-commit-config.yaml
repos:
  - repo: local
    hooks:
      - id: sql-analyzer
        name: SQL Query Analyzer
        entry: sql-query-analyzer analyze -s schema.sql -q
        language: system
        files: \.sql$
```

<div align="right"><a href="#table-of-contents">↑ Back to top</a></div>

## LLM Providers

| Provider | Model Examples | Notes |
|----------|----------------|-------|
| OpenAI | `gpt-4`, `gpt-3.5-turbo` | Requires API key |
| Anthropic | `claude-sonnet-4-20250514` | Requires API key |
| Ollama | `llama3.2`, `codellama`, `mistral` | Local, no API key |

### Using Ollama (Recommended for Development)

```bash
# Install Ollama
curl -fsSL https://ollama.com/install.sh | sh

# Pull a model
ollama pull llama3.2

# Run analysis
sql-query-analyzer analyze -s schema.sql -q queries.sql
```

<div align="right"><a href="#table-of-contents">↑ Back to top</a></div>

## Architecture

```
┌─────────────────────────────────────────────────────┐
│                    CLI Interface                     │
└─────────────────────┬───────────────────────────────┘
                      │
         ┌────────────┴────────────┐
         ▼                         ▼
┌─────────────────┐       ┌─────────────────┐
│  SQL Parser     │       │  Schema Parser  │
│  (sqlparser)    │       │  (sqlparser)    │
└────────┬────────┘       └────────┬────────┘
         │                         │
         └────────────┬────────────┘
                      ▼
         ┌────────────────────────┐
         │    Static Analysis     │
         │  (18 rules, parallel)  │
         └────────────┬───────────┘
                      │
                      ▼
         ┌────────────────────────┐
         │   LLM Analysis (opt)   │
         │  OpenAI/Anthropic/     │
         │  Ollama                │
         └────────────┬───────────┘
                      │
                      ▼
         ┌────────────────────────┐
         │   Output Formatter     │
         │  Text/JSON/YAML/SARIF  │
         └────────────────────────┘
```

<div align="right"><a href="#table-of-contents">↑ Back to top</a></div>

## Performance

- **Parallel rule execution** via rayon
- **Query caching** to avoid re-parsing identical queries
- **Lazy evaluation** for complexity scoring
- **Memory-efficient** string storage with CompactString

Typical performance: ~1000 queries analyzed in <100ms (static analysis only).

<div align="right"><a href="#table-of-contents">↑ Back to top</a></div>

## Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

### Development

```bash
# Run tests
cargo test

# Run with all checks
cargo clippy --all-targets -- -D warnings

# Generate docs
cargo doc --open

# Format code
cargo fmt
```

<div align="right"><a href="#table-of-contents">↑ Back to top</a></div>

## Acknowledgements

The idea for this tool came from [Yegor Bugayenko](https://www.yegor256.com/):

> It would be great to have a tool that takes two inputs: 1) the entire database schema in SQL, and 2) all SQL queries that my web app issues to the database during unit testing. The tool should use an LLM to analyze the queries and identify which ones are suboptimal, especially with respect to the existing indexes.

<div align="right"><a href="#table-of-contents">↑ Back to top</a></div>

## Coverage

<details>
<summary>Coverage Graphs</summary>

### Sunburst

The inner-most circle is the entire project, moving away from the center are folders then, finally, a single file. The size and color of each slice is representing the number of statements and the coverage, respectively.

![Sunburst](https://codecov.io/gh/RAprogramm/sql-query-analyzer/graphs/sunburst.svg?token=hKvq66JThf)

### Grid

Each block represents a single file in the project. The size and color of each block is represented by the number of statements and the coverage, respectively.

![Grid](https://codecov.io/gh/RAprogramm/sql-query-analyzer/graphs/tree.svg?token=hKvq66JThf)

### Icicle

The top section represents the entire project. Proceeding with folders and finally individual files. The size and color of each slice is representing the number of statements and the coverage, respectively.

![Icicle](https://codecov.io/gh/RAprogramm/sql-query-analyzer/graphs/icicle.svg?token=hKvq66JThf)

</details>

<div align="right"><a href="#table-of-contents">↑ Back to top</a></div>

## License

[MIT](LICENSE) © 2025

<div align="right"><a href="#table-of-contents">↑ Back to top</a></div>
