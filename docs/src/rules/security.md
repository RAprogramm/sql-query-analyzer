<!--
SPDX-FileCopyrightText: 2026 RAprogramm
SPDX-License-Identifier: MIT
-->

# Security Rules

All security rules are **Error** severity: they make the process exit with
code `2`, failing CI.

## SEC001 — UPDATE without WHERE

```sql
-- Flagged: updates every row in the table
UPDATE users SET active = false;

-- Intentional bulk update? Make it explicit:
UPDATE users SET active = false WHERE true;
```

## SEC002 — DELETE without WHERE

```sql
-- Flagged: removes all rows
DELETE FROM sessions;
```

## SEC003 — TRUNCATE

`TRUNCATE` bypasses row-level triggers and, on most engines, cannot be limited
or easily audited.

## SEC004 — DROP

`DROP TABLE` / `DROP DATABASE` permanently destroys data and schema. The rule
flags any DROP statement found in analyzed query files — migration tooling
should own such statements, not application query sets.
