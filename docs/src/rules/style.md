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
