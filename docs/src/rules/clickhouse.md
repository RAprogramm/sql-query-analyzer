<!--
SPDX-FileCopyrightText: 2026 RAprogramm
SPDX-License-Identifier: MIT
-->

# ClickHouse Rules

These rules activate only with `--dialect clickhouse`. They replace the OLTP
index-suggestion rules (`SCHEMA001`, `SCHEMA003`, `SCHEMA004`) with checks that
understand ClickHouse's sparse primary index, partition pruning, and columnar
storage. The OLTP rules assume `CREATE INDEX`, which does not exist in
ClickHouse, so they are skipped for this dialect.

## CH001 — SELECT * on a columnar store (Info)

`SELECT *` reads and decompresses every column file. List only the columns you
need.

## CH002 — Filter does not use the sort key (Warning)

`WHERE` does not reference any `ORDER BY`/`PRIMARY KEY` column, so no granules
are pruned.

```sql
-- schema.sql
CREATE TABLE t (d Date, x UInt64) ENGINE = MergeTree ORDER BY d;

-- Flagged: x is not part of the sort key
SELECT d FROM t WHERE x = 1;
```

## CH003 — Partition pruning missed (Warning)

`WHERE` does not match the `PARTITION BY` expression, so whole partitions are
scanned.

## CH004 — Function on a date column in WHERE (Warning)

Wrapping a date column in `toDate`/`toYear`/... disables the sparse index. Use a
range predicate instead.

## CH005 — No skipping index for filter column (Info)

The filter column is in neither the sort key nor a skipping index. Suggests an
`ADD INDEX ... TYPE minmax/bloom_filter`.

## CH006 — LowCardinality on a high-cardinality column (Info)

`LowCardinality` helps below roughly 10k distinct values; on `id`/`uuid`/`url`
columns it usually hurts.

## CH007 — Nullable column in a hot path (Info)

`Nullable` adds a null-mask read and blocks optimizations; prefer a sentinel
default.

## CH009 — uniqExact usage (Info)

`uniqExact` tracks every distinct value. Prefer `uniq`/`uniqCombined` for
analytics.

## CH010 — Conditional counts via subqueries (Info)

Several `count()` subqueries scan the table separately; `countIf(condition)`
does it in one pass.

## CH011 — ORDER BY in a subquery without LIMIT (Warning)

Sorting a subquery result is wasted work when the outer query re-sorts.

## CH012 — GROUP BY does not match the sort key (Info)

Aligning the `GROUP BY` prefix with `ORDER BY` enables streaming aggregation
without a hash table.

## CH016 — Multi-table JOIN (Info)

ClickHouse builds an in-memory hash table per join. Keep the large table on the
left or replace dimension joins with a dictionary.

## CH019 — FINAL modifier usage (Warning)

`FINAL` merges parts at read time. Prefer `argMax`/`argMin` or
`ReplacingMergeTree` with background merges.

## CH021 — PREWHERE opportunity (Info)

A `WHERE` filter reads all selected columns before filtering. Moving the
selective filter columns to `PREWHERE` lets ClickHouse read them first and skip
granules early.

## CH022 — Heavy mutation (Warning)

`ALTER TABLE ... UPDATE/DELETE` rewrites whole parts asynchronously and is
expensive. Prefer a `ReplacingMergeTree` with a version column, or drop whole
partitions.

## CH023 — Manual OPTIMIZE FINAL (Warning)

`OPTIMIZE TABLE ... FINAL` rewrites parts synchronously. Rely on background
merges and reserve `OPTIMIZE FINAL` for maintenance windows.

## CH024 — Dictionary lookup opportunity (Info)

A `JOIN` against a table that is also defined as a dictionary pays full-table
cost. Use `dictGet`/`dictHas` on the dictionary instead.

## CH025 — GROUP BY on a plain String key (Info)

Grouping on a plain `String` builds large hash tables. Use
`LowCardinality(String)` for low-cardinality grouping keys.
