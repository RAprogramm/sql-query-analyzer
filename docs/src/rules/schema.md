<!--
SPDX-FileCopyrightText: 2026 RAprogramm
SPDX-License-Identifier: MIT
-->

# Schema-Aware Rules

These rules cross-check queries against the DDL passed via `--schema`. They
parse `CREATE TABLE` and `CREATE INDEX` statements to build a model of tables,
columns, and indexes.

## SCHEMA001 — Missing index on filter column (Warning)

A column used in `WHERE` or `JOIN` has no index.

```sql
-- schema.sql
CREATE TABLE orders (id INT PRIMARY KEY, user_id INT);

-- Flagged: user_id is filtered but not indexed
SELECT * FROM orders WHERE user_id = 42;

-- Fix in DDL
CREATE INDEX idx_orders_user_id ON orders (user_id);
```

## SCHEMA002 — Column not found in schema (Warning)

The query references a column that does not exist in the declared schema —
usually a typo or a stale query after a migration.

## SCHEMA003 — Index suggestion for ORDER BY (Info)

An `ORDER BY` column without an index forces a sort; with one, rows can be
read in order.
