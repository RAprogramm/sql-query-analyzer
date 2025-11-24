# SQL Query Analyzer

[![Crates.io](https://img.shields.io/crates/v/sql-query-analyzer.svg)](https://crates.io/crates/sql-query-analyzer) [![Docs.rs](https://docs.rs/sql-query-analyzer/badge.svg)](https://docs.rs/sql-query-analyzer) [![Downloads](https://img.shields.io/crates/d/sql-query-analyzer.svg)](https://crates.io/crates/sql-query-analyzer) [![License](https://img.shields.io/crates/l/sql-query-analyzer.svg)](LICENSE)

[![CI](https://github.com/RAprogramm/sql-query-analyzer/actions/workflows/ci.yml/badge.svg)](https://github.com/RAprogramm/sql-query-analyzer/actions/workflows/ci.yml) [![Codecov](https://codecov.io/gh/RAprogramm/sql-query-analyzer/branch/main/graph/badge.svg)](https://codecov.io/gh/RAprogramm/sql-query-analyzer) [![REUSE](https://api.reuse.software/badge/github.com/RAprogramm/sql-query-analyzer)](https://api.reuse.software/info/github.com/RAprogramm/sql-query-analyzer) [![Hits-of-Code](https://hitsofcode.com/github/RAprogramm/sql-query-analyzer?branch=main)](https://hitsofcode.com/github/RAprogramm/sql-query-analyzer/view?branch=main)

A CLI tool that analyzes SQL queries for performance optimization using LLM. It takes your database schema and SQL queries, then identifies suboptimal queries—especially those not utilizing indexes effectively.

## Features

- Parses SQL schema (tables, columns, indexes)
- Extracts query patterns (WHERE, JOIN, ORDER BY columns)
- Supports multiple LLM providers: OpenAI, Anthropic Claude, Ollama
- Provides actionable optimization recommendations

## Installation

```bash
cargo build --release
```

## Usage

```bash
# With Ollama (local, no API key required)
sql-query-analyzer analyze -s schema.sql -q queries.sql

# With OpenAI
sql-query-analyzer analyze -s schema.sql -q queries.sql -p open-ai --api-key $OPENAI_KEY

# With Anthropic
sql-query-analyzer analyze -s schema.sql -q queries.sql -p anthropic --api-key $ANTHROPIC_KEY

# Custom model
sql-query-analyzer analyze -s schema.sql -q queries.sql -p ollama -m codellama
```

### Options

| Flag | Description | Default |
|------|-------------|---------|
| `-s, --schema` | Path to SQL schema file | required |
| `-q, --queries` | Path to SQL queries file | required |
| `-p, --provider` | LLM provider (ollama, open-ai, anthropic) | ollama |
| `-a, --api-key` | API key (or set `LLM_API_KEY` env var) | - |
| `-m, --model` | Model name | provider default |
| `--ollama-url` | Ollama base URL | http://localhost:11434 |

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
    created_at TIMESTAMP
);

CREATE INDEX idx_orders_user ON orders(user_id);
```

**queries.sql:**
```sql
SELECT * FROM users WHERE email = 'test@example.com';
SELECT * FROM orders WHERE user_id = 1 ORDER BY created_at DESC;
SELECT * FROM users WHERE created_at > '2024-01-01';
```

**Output:**
```
=== SQL Query Analysis ===

Query #1: SELECT * FROM users WHERE email = ?
- No index on 'email' column
- Recommendation: CREATE INDEX idx_users_email ON users(email)

Query #2: SELECT * FROM orders WHERE user_id = ? ORDER BY created_at
- Index on user_id exists but ORDER BY requires filesort
- Recommendation: CREATE INDEX idx_orders_user_created ON orders(user_id, created_at)

Query #3: SELECT * FROM users WHERE created_at > ?
- No index on 'created_at' column
- Recommendation: CREATE INDEX idx_users_created ON users(created_at)
```

## GitHub Action

Use in your CI/CD pipeline:

```yaml
name: Analyze SQL Queries

on:
  pull_request:
    paths:
      - '**/*.sql'

jobs:
  analyze:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Analyze SQL queries
        uses: RAprogramm/sql-query-analyzer@v1
        with:
          schema: db/schema.sql
          queries: db/queries.sql
          provider: anthropic
          api_key: ${{ secrets.ANTHROPIC_API_KEY }}
          post_comment: 'true'
```

### Inputs

| Input | Description | Required | Default |
|-------|-------------|----------|---------|
| `schema` | Path to SQL schema file | Yes | - |
| `queries` | Path to SQL queries file | Yes | - |
| `provider` | LLM provider (ollama, open-ai, anthropic) | No | anthropic |
| `api_key` | API key for LLM provider | Yes | - |
| `model` | Model name | No | provider default |
| `post_comment` | Post analysis as PR comment | No | false |
| `update_comment` | Update existing comment instead of creating new | No | true |

### Outputs

| Output | Description |
|--------|-------------|
| `analysis` | Analysis result from LLM |

## Acknowledgements

The idea for this tool came from [Yegor Bugayenko](https://www.yegor256.com/):

> It would be great to have a tool that takes two inputs: 1) the entire database schema in SQL, and 2) all SQL queries that my web app issues to the database during unit testing. The tool should use an LLM to analyze the queries and identify which ones are suboptimal, especially with respect to the existing indexes. ChatGPT 5.1 says that such a tool doesn't exist yet. Maybe you can build it?

## License

MIT
