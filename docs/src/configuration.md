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
backoff_factor = 2.0

[rules]
# Disable rules by ID
disabled = ["STYLE001", "PERF010"]

# Override severities: error | warning | info
[rules.severity]
PERF001 = "error"
SEC003 = "warning"
```

## Environment variables

| Variable | Effect |
|----------|--------|
| `LLM_API_KEY` | API key for OpenAI / Anthropic |
| `LLM_PROVIDER` | Default provider name |

## Rule tuning

- `rules.disabled` — a list of rule IDs to skip entirely.
- `rules.severity` — per-rule severity overrides; affects both output and the
  process exit code (see [Quick Start](quick-start.md#4-exit-codes)).
