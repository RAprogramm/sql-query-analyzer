<!--
SPDX-FileCopyrightText: 2026 RAprogramm
SPDX-License-Identifier: MIT
-->

# Command Line

The tool has a single subcommand: `analyze`.

```bash
sql-query-analyzer analyze [OPTIONS] --schema <SCHEMA> --queries <QUERIES>
```

## Options

| Option | Default | Description |
|--------|---------|-------------|
| `-s, --schema <PATH>` | required | Path to the SQL schema file |
| `-q, --queries <PATH>` | required | Path to the queries file, `-` for stdin |
| `-p, --provider <PROVIDER>` | `ollama` | LLM provider: `open-ai`, `anthropic`, `ollama` |
| `-a, --api-key <KEY>` | env `LLM_API_KEY` | API key for OpenAI or Anthropic |
| `-m, --model <MODEL>` | provider default | Model name override |
| `--ollama-url <URL>` | `http://localhost:11434` | Ollama base URL |
| `--dialect <DIALECT>` | `generic` | SQL dialect: `generic`, `mysql`, `postgresql`, `sqlite`, `clickhouse` |
| `-f, --output-format <FMT>` | `text` | Output: `text`, `json`, `yaml`, `sarif` |
| `-v, --verbose` | off | Include per-query complexity scores |
| `--dry-run` | off | Show what would be sent to the LLM without calling it |
| `--no-color` | off | Disable colored output |

## Examples

Static analysis only (no API key set, LLM step is skipped):

```bash
sql-query-analyzer analyze -s schema.sql -q queries.sql
```

PostgreSQL dialect with JSON output:

```bash
sql-query-analyzer analyze -s schema.sql -q queries.sql \
  --dialect postgresql -f json
```

SARIF for GitHub code scanning:

```bash
sql-query-analyzer analyze -s schema.sql -q queries.sql -f sarif > results.sarif
```

Verbose mode with complexity scores:

```bash
sql-query-analyzer analyze -s schema.sql -q queries.sql -v
```
