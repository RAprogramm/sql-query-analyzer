<!--
SPDX-FileCopyrightText: 2026 RAprogramm
SPDX-License-Identifier: MIT
-->

# Output Formats

Select with `-f, --output-format`.

## text (default)

Human-readable, colored report. Colors can be disabled with `--no-color`.

## json

Machine-readable report with queries, violations, and metadata. Suitable for
custom tooling and dashboards.

```bash
sql-query-analyzer analyze -s schema.sql -q queries.sql -f json | jq '.violations'
```

## yaml

The same structure as JSON, serialized as YAML.

## sarif

[SARIF 2.1.0](https://sarifweb.azurewebsites.net/) — the standard format for
static analysis results, understood by GitHub code scanning.

```bash
sql-query-analyzer analyze -s schema.sql -q queries.sql -f sarif > results.sarif
```

Upload in GitHub Actions:

```yaml
- name: Analyze SQL
  run: sql-query-analyzer analyze -s schema.sql -q queries.sql -f sarif > results.sarif

- name: Upload SARIF
  uses: github/codeql-action/upload-sarif@v4
  with:
    sarif_file: results.sarif
```

Violations then appear in the repository's **Security → Code scanning** tab and
as inline annotations in pull requests.
