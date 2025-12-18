# SQL Query Analyzer

[![Crates.io](https://img.shields.io/crates/v/sql-query-analyzer.svg)](https://crates.io/crates/sql-query-analyzer)
[![Docs.rs](https://docs.rs/sql_query_analyzer/badge.svg)](https://docs.rs/sql_query_analyzer)
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
- [ClickHouse Support](#clickhouse-support)
- [CI Pipeline](#ci-pipeline)
- [Performance](#performance)
- [Contributing](#contributing)
- [Acknowledgements](#acknowledgements)
- [Coverage](#coverage)
- [License](#license)

## Highlights

- **20 Built-in Rules** — Performance, style, and security checks run instantly without API calls
- **Schema-Aware Analysis** — Validates queries against your database schema, suggests missing indexes
- **Multi-Dialect Support** — Generic, MySQL, PostgreSQL, SQLite, and ClickHouse with preprocessor for dialect-specific syntax
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
| `SEC003` | TRUNCATE detected | Error | Instant data deletion without logging |
| `SEC004` | DROP detected | Error | Permanent data/schema destruction |

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
| `--dialect <DIALECT>` | SQL dialect: `generic`, `mysql`, `postgresql`, `sqlite`, `clickhouse` | `generic` |
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
| `dialect` | SQL dialect (generic, mysql, postgresql, sqlite, clickhouse) | `generic` |
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

This runs all 20 built-in rules instantly without requiring any API keys.

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
         │  (20 rules, parallel)  │
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

## ClickHouse Support

The analyzer includes a preprocessor that handles ClickHouse-specific DDL constructs not supported by the underlying SQL parser:

### Supported Constructs

| Construct | Example | Description |
|-----------|---------|-------------|
| `CODEC` | `col String CODEC(ZSTD)` | Column compression codecs |
| `TTL` | `TTL event_date + INTERVAL 90 DAY` | Data expiration rules |
| `SETTINGS` | `SETTINGS index_granularity = 8192` | Table-level settings |
| `PARTITION BY` | `PARTITION BY toYYYYMM(date)` | Partitioning expressions |

### Example

```sql
CREATE TABLE events ON CLUSTER default (
    event_date Date,
    event_time DateTime CODEC(Delta, ZSTD),
    user_id UInt64 CODEC(T64),
    data String CODEC(ZSTD(3))
) ENGINE = ReplicatedMergeTree('/clickhouse/tables/{shard}/events', '{replica}')
PARTITION BY toYYYYMM(event_date)
ORDER BY (event_date, user_id)
TTL event_date + INTERVAL 90 DAY
SETTINGS index_granularity = 8192
```

```bash
sql-query-analyzer analyze --dialect clickhouse -s schema.sql -q queries.sql
```

The preprocessor extracts metadata (codecs, TTL, settings) and removes unsupported syntax before parsing, ensuring compatibility while preserving information for analysis output.

<div align="right"><a href="#table-of-contents">↑ Back to top</a></div>

## CI Pipeline

This project uses a comprehensive CI pipeline with 16 jobs organized into quality gates.

### Pipeline Overview

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              CI PIPELINE                                     │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  ┌─────────┐                                                                │
│  │ changes │ ─── Detects modified files and triggers relevant jobs          │
│  └────┬────┘                                                                │
│       │                                                                     │
│       ├──────────────────────────────────────────────────────────────────┐  │
│       │                                                                  │  │
│       ▼                                                                  │  │
│  ┌─────────┐                                                             │  │
│  │   fmt   │ ─── cargo +nightly fmt --check                              │  │
│  └────┬────┘                                                             │  │
│       │                                                                  │  │
│       ├─────────────────────┬─────────────────────┐                      │  │
│       │                     │                     │                      │  │
│       ▼                     ▼                     ▼                      │  │
│  ┌─────────┐           ┌─────────┐           ┌─────────┐                 │  │
│  │ clippy  │           │  msrv   │           │ machete │                 │  │
│  │         │           │ (1.90)  │           │         │                 │  │
│  └────┬────┘           └────┬────┘           └────┬────┘                 │  │
│       │                     │                     │                      │  │
│       ├──────────┬──────────┼─────────────────────┤                      │  │
│       │          │          │                     │                      │  │
│       ▼          ▼          │                     │                      │  │
│  ┌─────────┐ ┌─────────┐    │                     │                      │  │
│  │  test   │ │   doc   │    │                     │                      │  │
│  │+coverage│ │         │    │                     │      ┌─────────┐     │  │
│  └────┬────┘ └────┬────┘    │                     │      │  audit  │◄────┘  │
│       │          │          │                     │      └────┬────┘        │
│       ▼          │          │                     │           │             │
│  ┌─────────┐     │          │                     │      ┌────▼────┐        │
│  │ doctest │     │          │                     │      │  deny   │        │
│  └────┬────┘     │          │                     │      └────┬────┘        │
│       │          │          │                     │           │             │
│       ▼          │          │                     │      ┌────▼────┐        │
│  ┌─────────┐     │          │                     │      │  reuse  │        │
│  │ semver  │     │          │                     │      └────┬────┘        │
│  │(PR only)│     │          │                     │           │             │
│  └────┬────┘     │          │                     │           │             │
│       │          │          │                     │           │             │
│       └──────────┴──────────┴─────────────────────┴───────────┘             │
│                                    │                                        │
│                                    ▼                                        │
│                             ┌───────────┐                                   │
│                             │   build   │                                   │
│                             └─────┬─────┘                                   │
│                                   │                                         │
│                                   ▼                                         │
│                            ┌───────────┐                                    │
│                            │ changelog │ (main branch only)                 │
│                            └───────────┘                                    │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Job Dependency Graph

```
                    ┌─────────┐
                    │ changes │
                    └────┬────┘
                         │
         ┌───────────────┼───────────────┬───────────────┐
         │               │               │               │
         ▼               ▼               ▼               ▼
    ┌─────────┐     ┌─────────┐     ┌─────────┐     ┌─────────┐
    │   fmt   │     │  audit  │     │  deny   │     │  reuse  │
    └────┬────┘     └────┬────┘     └────┬────┘     └────┬────┘
         │               │               │               │
    ┌────┴────┐          │               │               │
    │         │          │               │               │
    ▼         ▼          │               │               │
┌───────┐ ┌───────┐      │               │               │
│clippy │ │ msrv  │      │               │               │
└───┬───┘ └───┬───┘      │               │               │
    │         │          │               │               │
    │    ┌────┘          │               │               │
    │    │               │               │               │
    ▼    │               │               │               │
┌───────┐│               │               │               │
│machete│◄───────────────┤               │               │
└───┬───┘                │               │               │
    │                    │               │               │
    ├────────────────────┤               │               │
    │                    │               │               │
    ▼                    │               │               │
┌────────┐               │               │               │
│  test  │               │               │               │
│  doc   │               │               │               │
│doctest │               │               │               │
│ semver │               │               │               │
└───┬────┘               │               │               │
    │                    │               │               │
    └────────────────────┴───────────────┴───────────────┘
                         │
                         ▼
                    ┌─────────┐
                    │  build  │
                    └────┬────┘
                         │
                         ▼
                   ┌───────────┐
                   │ changelog │
                   └───────────┘
```

### Quality Gates

| Job | Trigger | Tool | Description |
|-----|---------|------|-------------|
| **fmt** | `src/**`, `tests/**`, `Cargo.*` | `cargo +nightly fmt` | Code formatting verification |
| **clippy** | `src/**`, `tests/**`, `Cargo.*` | `cargo clippy` | Static analysis with `-D warnings` |
| **test** | `src/**`, `tests/**`, `Cargo.*` | `cargo-nextest` + `cargo-llvm-cov` | Tests with coverage upload to Codecov |
| **doc** | `src/**`, `tests/**`, `Cargo.*` | `cargo doc` | Documentation with `-D warnings` |
| **doctest** | `src/**`, `tests/**`, `Cargo.*` | `cargo test --doc` | Documentation examples verification |
| **audit** | `Cargo.toml`, `Cargo.lock` | `cargo-audit` | Security vulnerability scanning (RustSec) |
| **deny** | `Cargo.toml`, `Cargo.lock` | `cargo-deny` | License and dependency policy |
| **msrv** | `src/**`, `tests/**`, `Cargo.*` | `rustc 1.90` | MSRV compatibility check |
| **machete** | `Cargo.toml`, `Cargo.lock` | `cargo-machete` | Unused dependency detection |
| **semver** | Pull requests only | `cargo-semver-checks` | Public API compatibility |
| **reuse** | `LICENSES/**`, `**/*.rs`, `**/*.toml` | `reuse lint` | SPDX license compliance |

### Change Detection

The pipeline uses smart change detection to skip unnecessary jobs:

```
┌──────────────────────────────────────────────────────────────┐
│                    Change Detection Matrix                    │
├────────────────────┬─────────────────────────────────────────┤
│ Filter             │ Paths                                   │
├────────────────────┼─────────────────────────────────────────┤
│ rust               │ src/**, tests/**, Cargo.toml,           │
│                    │ Cargo.lock, .rustfmt.toml               │
├────────────────────┼─────────────────────────────────────────┤
│ deps               │ Cargo.toml, Cargo.lock                  │
├────────────────────┼─────────────────────────────────────────┤
│ reuse              │ LICENSES/**, .reuse/**, **/*.rs,        │
│                    │ **/*.toml, **/*.yml, **/*.md            │
└────────────────────┴─────────────────────────────────────────┘
```

### Dependency Policy

The `deny.toml` configuration enforces:

| Policy | Configuration |
|--------|---------------|
| **Allowed Licenses** | MIT, Apache-2.0, BSD-2-Clause, BSD-3-Clause, ISC, Zlib, CC0-1.0, Unicode-3.0, Unicode-DFS-2016, BSL-1.0, MPL-2.0 |
| **Banned Crates** | `openssl`, `openssl-sys` (use `rustls` instead) |
| **Registry** | crates.io only (no unknown registries or git sources) |
| **Duplicates** | Warn on multiple versions of the same crate |
| **Wildcards** | Denied in version requirements |

### Release Pipeline

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           RELEASE PIPELINE                                   │
│                         (triggered by v* tags)                               │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │                         Quality Gates                                │    │
│  │    test + doc + audit + deny + reuse + msrv + machete + doctest     │    │
│  └──────────────────────────────┬──────────────────────────────────────┘    │
│                                 │                                           │
│                                 ▼                                           │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │                       release-build (matrix)                         │    │
│  │  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐ ┌─────────────┐    │    │
│  │  │ linux-gnu   │ │ linux-musl  │ │ linux-arm64 │ │ macos-x64   │    │    │
│  │  │   x86_64    │ │   x86_64    │ │   aarch64   │ │   x86_64    │    │    │
│  │  └─────────────┘ └─────────────┘ └─────────────┘ └─────────────┘    │    │
│  │  ┌─────────────┐ ┌─────────────┐                                    │    │
│  │  │ macos-arm64 │ │ windows-x64 │                                    │    │
│  │  │   aarch64   │ │    msvc     │                                    │    │
│  │  └─────────────┘ └─────────────┘                                    │    │
│  └──────────────────────────────┬──────────────────────────────────────┘    │
│                                 │                                           │
│                    ┌────────────┴────────────┐                              │
│                    │                         │                              │
│                    ▼                         ▼                              │
│             ┌───────────┐             ┌───────────┐                         │
│             │  release  │             │  publish  │                         │
│             │ (GitHub)  │             │(crates.io)│                         │
│             └───────────┘             └───────────┘                         │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Supported Targets

| Target | OS | Architecture | Build Method |
|--------|-----|--------------|--------------|
| `x86_64-unknown-linux-gnu` | Linux | x86_64 | Native |
| `x86_64-unknown-linux-musl` | Linux (static) | x86_64 | Cross |
| `aarch64-unknown-linux-gnu` | Linux | ARM64 | Cross |
| `x86_64-apple-darwin` | macOS | x86_64 | Native |
| `aarch64-apple-darwin` | macOS | ARM64 | Native |
| `x86_64-pc-windows-msvc` | Windows | x86_64 | Native |

### Caching Strategy

All jobs utilize `Swatinem/rust-cache@v2` with job-specific cache keys:

| Job | Cache Key |
|-----|-----------|
| **clippy** | Default |
| **test** | Default |
| **doc** | Default |
| **msrv** | `msrv` |
| **machete** | `machete` |
| **doctest** | `doctest` |
| **semver** | `semver` |
| **release-build** | `release-{target}` |

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
