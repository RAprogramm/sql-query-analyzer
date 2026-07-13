<!--
SPDX-FileCopyrightText: 2026 RAprogramm
SPDX-License-Identifier: MIT
-->

# LLM Providers

Static analysis always runs. If an API key (or a local Ollama instance) is
available, the tool additionally sends compact schema and query summaries to an
LLM for context-aware recommendations: index strategy, query rewrites, and
dialect-specific advice.

No key, no call: without `LLM_API_KEY` the LLM step is skipped and a note is
printed.

## Providers

| Provider | Flag | Default model | Auth |
|----------|------|---------------|------|
| Ollama (default) | `--provider ollama` | `llama3.2` | none, local |
| OpenAI | `--provider open-ai` | `gpt-4` | `LLM_API_KEY` |
| Anthropic | `--provider anthropic` | `claude-sonnet-4-20250514` | `LLM_API_KEY` |

Override the model with `-m/--model`, the Ollama endpoint with `--ollama-url`.

## Examples

Local Ollama:

```bash
ollama pull llama3.2
sql-query-analyzer analyze -s schema.sql -q queries.sql
```

Anthropic:

```bash
export LLM_API_KEY=sk-ant-...
sql-query-analyzer analyze -s schema.sql -q queries.sql --provider anthropic
```

## Dry run

Inspect exactly what would be sent before spending tokens:

```bash
sql-query-analyzer analyze -s schema.sql -q queries.sql --dry-run
```

## Retries

Transient API failures are retried with exponential backoff, configurable via
the `[retry]` section — see [Configuration](configuration.md).
