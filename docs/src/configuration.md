<!--
SPDX-FileCopyrightText: 2026 RAprogramm
SPDX-License-Identifier: MIT
-->

# Configuration

Configuration is merged from several sources, highest priority first:

1. Command line arguments
2. Environment variables
3. `.sql-analyzer.toml` in the current directory
4. `~/.config/sql-analyzer/config.toml`

## Config file

```toml
[llm]
provider = "openai"
model = "gpt-4"
# api_key can be set here, but the environment variable is preferred
# ollama_url = "http://localhost:11434"

[retry]
max_retries = 3
initial_delay_ms = 500
max_delay_ms = 8000
backoff_factor = 2

[rules]
# Disable rules by ID
disabled = ["STYLE001", "PERF010"]

# Override severities: error | warning | info
[rules.severity]
PERF001 = "error"
SEC003 = "warning"
```

`backoff_factor` is an integer multiplier applied to `initial_delay_ms`
between retries. It used to accept floats (`2.0`); since 0.17 the field is
an integer, so configuration files that still write a float value must be
updated to the integer form (`2`) or loading fails with a deserialization
error.

## Environment variables

| Variable | Effect |
|----------|--------|
| `LLM_API_KEY` | API key for OpenAI / Anthropic |
| `LLM_PROVIDER` | Default provider name |

## Rule tuning

- `rules.disabled` — a list of rule IDs to skip entirely.
- `rules.severity` — per-rule severity overrides; affects both output and the
  process exit code (see [Quick Start](quick-start.md#4-exit-codes)).
