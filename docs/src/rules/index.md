<!--
SPDX-FileCopyrightText: 2026 RAprogramm
SPDX-License-Identifier: MIT
-->

# Rules Overview

27 built-in rules across four categories. Every rule has a stable ID, a default
severity, and a suggestion attached to each violation. Rules can be disabled or
re-weighted via [configuration](../configuration.md).

| Category | IDs | Focus |
|----------|-----|-------|
| [Performance](performance.md) | `PERF001`–`PERF013` | Index usage, table scans, N+1 patterns |
| [Style](style.md) | `STYLE001`–`STYLE004` | Readability and maintainability |
| [Security](security.md) | `SEC001`–`SEC008` | Destructive statements without guards |
| [Schema-Aware](schema.md) | `SCHEMA001`–`SCHEMA003` | Cross-checking queries against DDL |

## Severities

| Severity | Exit code contribution | Meaning |
|----------|------------------------|---------|
| Error | `2` | Almost certainly a bug or dangerous operation |
| Warning | `1` | Likely performance or correctness problem |
| Info | `0` | Improvement opportunity |
