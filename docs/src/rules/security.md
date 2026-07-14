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

## SEC006 — SQL injection pattern

An always-true `OR` tautology almost never appears in legitimate queries; it
is the classic fingerprint of injected input widening a `WHERE` clause to
match every row.

```sql
-- Flagged
SELECT * FROM users WHERE name = '' OR '1' = '1';
SELECT * FROM users WHERE id = 5 OR 1 = 1;

-- Not flagged: comparing a column is not a tautology
SELECT * FROM users WHERE id = 1 OR id = 2;
```

Comment-marker and statement-stacking heuristics are not needed at this
layer: the analyzer works on parsed statements, where comments are already
stripped and stacked statements are split apart. If this pattern shows up in
extracted application queries, replace string concatenation with
parameterized queries.

## SEC008 — Hardcoded credential

Secrets embedded in SQL leak through source control, slow query logs, and
error logs, and violate PCI-DSS/SOC2/HIPAA plaintext storage rules.

```sql
-- Flagged
CREATE USER app IDENTIFIED BY 'super_secret';
ALTER USER admin WITH PASSWORD 'changeme';
INSERT INTO users (email, password) VALUES ('a@b.c', 'admin123');
UPDATE users SET api_key = 'sk-live-abc123' WHERE id = 1;

-- Not flagged: parameterized values
INSERT INTO users (email, password) VALUES ($1, $2);
```

Detected signals: `IDENTIFIED BY` / `WITH PASSWORD` / `SET PASSWORD` clauses,
and string literals assigned or inserted into sensitive columns (`password`,
`passwd`, `pwd`, `secret`, `api_key`, `apikey`, `token`, `auth`,
`credential`, including prefixed names like `user_password`). Use environment
variables, a secret manager, or parameterized values instead.
