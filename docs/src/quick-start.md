<!--
SPDX-FileCopyrightText: 2026 RAprogramm
SPDX-License-Identifier: MIT
-->

# Quick Start

## 1. Prepare input files

`schema.sql` — your DDL:

```sql
CREATE TABLE users (
    id INT PRIMARY KEY,
    email VARCHAR(255),
    created_at TIMESTAMP
);

CREATE TABLE orders (
    id INT PRIMARY KEY,
    user_id INT,
    total DECIMAL(10, 2),
    created_at TIMESTAMP
);
```

`queries.sql` — the queries to analyze:

```sql
SELECT u.id, u.email, o.total
FROM users u
JOIN orders o ON o.user_id = u.id
WHERE u.email LIKE '%@example.com'
ORDER BY o.created_at DESC
LIMIT 100;
```

## 2. Run static analysis

```bash
sql-query-analyzer analyze --schema schema.sql --queries queries.sql
```

Example output:

```text
=== Static Analysis ===
  [ WARN] PERF002: LIKE pattern starts with wildcard, preventing index usage
  [ WARN] SCHEMA001: Column 'user_id' in JOIN clause has no index
  [ INFO] SCHEMA003: ORDER BY column 'created_at' could benefit from index
```

## 3. Read queries from stdin

```bash
echo "DELETE FROM users;" | sql-query-analyzer analyze -s schema.sql -q -
```

## 4. Exit codes

| Code | Meaning |
|------|---------|
| `0`  | No violations above Info |
| `1`  | At least one Warning |
| `2`  | At least one Error |

This makes the tool usable as a CI gate out of the box.

## 5. Optional: AI-powered analysis

```bash
export LLM_API_KEY=sk-...
sql-query-analyzer analyze -s schema.sql -q queries.sql --provider openai
```

See [LLM Providers](llm.md) for details.
