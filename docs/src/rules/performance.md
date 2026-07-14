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

## PERF012 — COUNT(*) without WHERE (Warning)

Counting every row cannot take an index shortcut on most engines; time grows
linearly with table size and the scan can block writes on busy tables.

```sql
-- Flagged
SELECT COUNT(*) FROM users;

-- Better: existence check
SELECT EXISTS(SELECT 1 FROM users LIMIT 1);

-- Better: bounded count
SELECT COUNT(*) FROM users WHERE created_at > '2026-01-01';
```

## PERF013 — ORDER BY RAND() (Warning)

The database generates a random value for every row and sorts the whole set
before applying `LIMIT` — O(n log n) regardless of how few rows are returned.
Detects `RAND()`, `RANDOM()`, `NEWID()`, and `DBMS_RANDOM`.

```sql
-- Flagged
SELECT * FROM products ORDER BY RAND() LIMIT 5;

-- Better: random id range
SELECT * FROM products
WHERE id >= FLOOR(RAND() * (SELECT MAX(id) FROM products))
LIMIT 5;

-- Better: pre-generated indexed random column
SELECT * FROM products ORDER BY random_sort LIMIT 5;
```

## PERF014 — Potentially unnecessary DISTINCT (Info)

`DISTINCT` combined with `JOIN` usually hides duplicate rows produced by join
fan-out; deduplication then costs a sort or hash over the whole result.
`SELECT DISTINCT *` escalates to Warning.

```sql
-- Flagged (Info): fix the join instead of deduplicating
SELECT DISTINCT u.name FROM users u JOIN orders o ON o.user_id = u.id;

-- Flagged (Warning)
SELECT DISTINCT * FROM users u JOIN orders o ON o.user_id = u.id;

-- Not flagged: unique-value enumeration on one table
SELECT DISTINCT status FROM orders;
```

## PERF015 — Implicit type conversion (Warning, needs schema)

Comparing a text column to a bare number forces a cast on every row; on most
engines the column side is cast, which disables its index and can silently
change matching semantics.

```sql
-- schema.sql
CREATE TABLE users (id INT PRIMARY KEY, phone VARCHAR(20));

-- Flagged
SELECT id FROM users WHERE phone = 5551234;

-- Better
SELECT id FROM users WHERE phone = '5551234';
```

## PERF016 — Multiple scans of same table (Info)

Repeated `FROM`/`JOIN` references to one table multiply I/O. A CTE, window
function, or conditional aggregation usually reads the table once.

```sql
-- Flagged
SELECT * FROM orders o1 WHERE amount > (SELECT AVG(amount) FROM orders);

-- Better: one scan with a window function
SELECT * FROM (
    SELECT o.*, AVG(amount) OVER () AS avg_amount FROM orders o
) t WHERE amount > avg_amount;
```

## PERF017 — Correlated subquery (Warning)

A subquery that references a table or alias of the outer query cannot be
evaluated once; the engine re-runs it for every candidate row.

```sql
-- Flagged: the subquery references outer alias u
SELECT * FROM users u
WHERE EXISTS (SELECT 1 FROM orders o WHERE o.user_id = u.id);

-- Not flagged: independent subquery, evaluated once
SELECT * FROM users WHERE id IN (SELECT user_id FROM orders);

-- Better: single pass
SELECT DISTINCT u.* FROM users u JOIN orders o ON o.user_id = u.id;
```

## PERF018 — HAVING without aggregate (Warning)

`HAVING` filters after grouping; a condition on plain columns forces the
engine to group rows it could have discarded up front.

```sql
-- Flagged
SELECT status, COUNT(*) FROM orders GROUP BY status HAVING status = 'active';

-- Better: prune before grouping
SELECT status, COUNT(*) FROM orders WHERE status = 'active' GROUP BY status;

-- Correct HAVING usage is not flagged
SELECT status, COUNT(*) FROM orders GROUP BY status HAVING COUNT(*) > 10;
```

## PERF019 — Large IN clause (Warning)

Very long `IN` lists blow up parse and plan time, defeat plan caching, and on
some engines hit hard parameter limits. Severity scales with size: more than
50 items is Info, more than 200 Warning, more than 1000 Error. Subqueries
(`IN (SELECT …)`) are not counted.

```sql
-- Flagged
SELECT * FROM users WHERE id IN (1, 2, 3, /* …60 more… */ 64);

-- Better: JOIN against a temporary table, or batch the lookups
```

## PERF020 — Deeply nested subqueries (Warning)

Each nesting level multiplies planning complexity and usually hides a JOIN or
CTE that would express the same logic flatter. Severity scales with total
SELECT depth: three levels is Info, four Warning, five or more Error.

```sql
-- Flagged (3 levels)
SELECT * FROM a WHERE x IN (
    SELECT y FROM b WHERE z IN (SELECT w FROM c WHERE v = 1));

-- Better: name the steps
WITH matching_c AS (SELECT w FROM c WHERE v = 1),
     matching_b AS (SELECT y FROM b JOIN matching_c ON b.z = matching_c.w)
SELECT a.* FROM a JOIN matching_b ON a.x = matching_b.y;
```
