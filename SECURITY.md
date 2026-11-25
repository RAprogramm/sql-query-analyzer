# Security Policy

This document describes how to report security vulnerabilities in
`sql-query-analyzer` and what you can expect in return.

## Supported Versions

Security fixes are provided for the latest released version and for the
current `main` branch. Older versions are not actively patched; update to
the latest release before reporting.

| Version | Supported          |
| ------- | ------------------ |
| latest  | :white_check_mark: |
| main    | :white_check_mark: |
| older   | :x:                |

## Reporting a Vulnerability

Report security vulnerabilities privately — do **not** open a public issue.

Use [GitHub Private Vulnerability Reporting](https://github.com/RAprogramm/sql-query-analyzer/security/advisories/new)
to submit a report. Every report is acknowledged within 5 business days.

Include, when possible:

- the affected version or commit;
- a minimal reproduction (the SQL input and the CLI invocation are usually
  enough — this tool parses untrusted SQL, so parser crashes, unbounded
  memory growth, and rule bypasses all count as security issues);
- the impact you observed or expect.

## Scope

In scope:

- memory-safety, crash, or resource-exhaustion issues reachable from
  SQL input (schema files, query files, stdin);
- logic flaws that make security rules (`SEC*`) silently pass dangerous SQL;
- credential handling in the LLM integrations (API keys, tokens);
- vulnerabilities in the shipped GitHub Action (`action.yml`).

Out of scope:

- findings that require the user to deliberately run the tool on
  attacker-controlled configuration files;
- social engineering of maintainers;
- third-party services (Ollama/OpenAI/Anthropic endpoints).

## Disclosure Policy

- We prepare a fix and a security advisory before any public disclosure.
- Coordinated disclosure: the advisory is published as soon as a patched
  version is released, or after 90 days, whichever comes first.
