<!--
SPDX-FileCopyrightText: 2026 RAprogramm
SPDX-License-Identifier: MIT
-->

# Performance Rules

## PERF001 — SELECT * without LIMIT (Warning)

Unbounded result sets consume memory and bandwidth.

```sql
-- Flagged
SELECT * FROM orders;

-- Better
SELECT id, total FROM orders LIMIT 100;
```

## PERF002 — Leading wildcard in LIKE (Warning)

`LIKE '%value'` cannot use a B-tree index.

```sql
-- Flagged
SELECT id FROM users WHERE email LIKE '%@example.com';

-- Better: full-text search, a generated reversed column, or a trigram index
```

## PERF003 — OR chain instead of IN (Info)

```sql
-- Flagged
SELECT id FROM users WHERE id = 1 OR id = 2 OR id = 3;

-- Better
SELECT id FROM users WHERE id IN (1, 2, 3);
```

## PERF004 — Large OFFSET (Warning)

`OFFSET n` reads and discards `n` rows; pagination degrades linearly.

```sql
-- Flagged
SELECT id FROM orders ORDER BY id LIMIT 20 OFFSET 100000;

-- Better: keyset pagination
SELECT id FROM orders WHERE id > :last_seen_id ORDER BY id LIMIT 20;
```

## PERF005 — Missing JOIN condition (Error)

A cartesian product multiplies row counts.

```sql
-- Flagged
SELECT * FROM users, orders;

-- Better
SELECT * FROM users u JOIN orders o ON o.user_id = u.id;
```

## PERF006 — DISTINCT with ORDER BY (Info)

Both operations sort or hash; combined they are often redundant.

## PERF007 — Scalar subquery in SELECT (Warning)

Executes once per row — the classic N+1 pattern.

```sql
-- Flagged
SELECT u.id, (SELECT COUNT(*) FROM orders o WHERE o.user_id = u.id) FROM users u;

-- Better
SELECT u.id, COUNT(o.id)
FROM users u LEFT JOIN orders o ON o.user_id = u.id
GROUP BY u.id;
```

## PERF008 — Function call on column in WHERE (Warning)

Wrapping an indexed column in a function disables the index.

```sql
-- Flagged
SELECT id FROM users WHERE LOWER(email) = 'a@b.c';

-- Better: a functional index, or store the normalized value
```

## PERF009 — NOT IN with subquery (Warning)

A single `NULL` in the subquery result makes `NOT IN` return no rows.

```sql
-- Flagged
SELECT id FROM users WHERE id NOT IN (SELECT user_id FROM orders);

-- Better
SELECT u.id FROM users u
WHERE NOT EXISTS (SELECT 1 FROM orders o WHERE o.user_id = u.id);
```

## PERF010 — UNION without ALL (Info)

`UNION` deduplicates (sort/hash); use `UNION ALL` when duplicates are
impossible or acceptable.

## PERF011 — SELECT without WHERE or LIMIT (Info)

Full table scan; intentional only for small reference tables.
