<!--
SPDX-FileCopyrightText: 2026 RAprogramm
SPDX-License-Identifier: MIT
-->

# Installation

## From crates.io

```bash
cargo install sql-query-analyzer
```

The binary is installed as `sql-query-analyzer`.

## From source

```bash
git clone https://github.com/RAprogramm/sql-query-analyzer
cd sql-query-analyzer
cargo install --path .
```

## Requirements

- Rust 1.97 or newer (matches the crate's `rust-version`)
- No runtime dependencies; LLM providers are optional

## Verify

```bash
sql-query-analyzer --version
```
