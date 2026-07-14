<!--
SPDX-FileCopyrightText: 2026 RAprogramm
SPDX-License-Identifier: MIT
-->

# Introduction

**SQL Query Analyzer** is a static analysis tool for SQL queries. It combines
fast, deterministic rule-based analysis with optional AI-powered insights to
identify performance issues, style violations, and security risks before they
reach production.

<p align="center">
  <img src="https://raw.githubusercontent.com/RAprogramm/sql-query-analyzer/main/.github/assets/banner.png" alt="SQL Query Analyzer" width="100%">
</p>

## Highlights

- **28 built-in rules** across performance, style, security, and schema-aware
  categories
- **Schema-aware analysis** — detects missing indexes and unknown columns by
  parsing your `CREATE TABLE` statements
- **Multi-dialect** — Generic, MySQL, PostgreSQL, SQLite, and ClickHouse
- **Fast and parallel** — rule execution is parallelized with Rayon
- **LLM-powered insights** — optional deep analysis via OpenAI, Anthropic, or a
  local Ollama instance
- **CI-ready** — SARIF output integrates with GitHub code scanning; exit codes
  reflect severity

## How it works

1. The tool parses your schema file and extracts tables, columns, and indexes.
2. Each query is parsed into an AST using [sqlparser](https://crates.io/crates/sqlparser).
3. Every enabled rule runs against each query in parallel.
4. Violations are reported with rule ID, severity, message, and suggestion.
5. Optionally, the schema and query summaries are sent to an LLM for deeper,
   context-aware recommendations.

## License

MIT. Source code is available on
[GitHub](https://github.com/RAprogramm/sql-query-analyzer).
