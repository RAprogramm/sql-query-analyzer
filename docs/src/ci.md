<!--
SPDX-FileCopyrightText: 2026 RAprogramm
SPDX-License-Identifier: MIT
-->

# CI and GitHub Action

## GitHub Action

The repository ships a ready-made action:

```yaml
name: SQL Analysis

on: pull_request

jobs:
  sql:
    runs-on: ubuntu-latest
    permissions:
      contents: read
      pull-requests: write
      security-events: write
    steps:
      - uses: actions/checkout@v7

      - uses: RAprogramm/sql-query-analyzer@main
        with:
          schema: db/schema.sql
          queries: db/queries.sql
          dialect: postgresql
          fail-on-warning: 'false'
          fail-on-error: 'true'
          post-comment: 'true'
          upload-sarif: 'true'
```

### Inputs

| Input | Default | Description |
|-------|---------|-------------|
| `schema` | required | Path to the schema file |
| `queries` | required | Path to the queries file |
| `dialect` | `generic` | SQL dialect |
| `format` | `text` | Output format |
| `fail-on-warning` | `false` | Fail the job on warnings |
| `fail-on-error` | `true` | Fail the job on errors |
| `disabled-rules` | — | Comma-separated rule IDs to skip |
| `post-comment` | `false` | Post the report as a PR comment |
| `update-comment` | `true` | Update the existing comment in place |
| `upload-sarif` | `false` | Upload SARIF to the Security tab |

### Outputs

`analysis`, `error-count`, `warning-count`, `exit-code`.

## Plain CI usage

Any CI can rely on exit codes (`0` clean / `1` warnings / `2` errors):

```bash
cargo install sql-query-analyzer
sql-query-analyzer analyze -s db/schema.sql -q db/queries.sql
```

## Pre-commit hook

```bash
#!/bin/sh
git diff --cached --name-only -z -- '*.sql' | grep -qz . || exit 0
sql-query-analyzer analyze -s db/schema.sql -q db/queries.sql
```
