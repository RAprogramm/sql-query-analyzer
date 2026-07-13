<!--
SPDX-FileCopyrightText: 2026 RAprogramm
SPDX-License-Identifier: MIT
-->

# Style Rules

## STYLE001 — SELECT * (Info)

Explicit column lists survive schema changes, avoid over-fetching, and make
code reviews meaningful.

```sql
-- Flagged
SELECT * FROM users WHERE id = 1;

-- Better
SELECT id, email, created_at FROM users WHERE id = 1;
```

## STYLE002 — Missing table alias (Info)

In multi-table queries unqualified columns become ambiguous as the schema
grows.

```sql
-- Flagged
SELECT users.id, orders.total FROM users JOIN orders ON orders.user_id = users.id;

-- Better
SELECT u.id, o.total FROM users u JOIN orders o ON o.user_id = u.id;
```

## STYLE004 — Ordinal in ORDER BY/GROUP BY (Info)

`ORDER BY 1` sorts by SELECT-list position. Add or reorder selected columns
and the sort silently changes — no error, just wrong results.

```sql
-- Flagged
SELECT name, COUNT(*) FROM users GROUP BY 1;
SELECT id, name FROM users ORDER BY 1 DESC;

-- Better
SELECT name, COUNT(*) FROM users GROUP BY name;
SELECT id, name FROM users ORDER BY id DESC;
```

Function arguments and `LIMIT`/`OFFSET` counts are not mistaken for ordinals.
