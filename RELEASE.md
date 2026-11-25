<!--
SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
SPDX-License-Identifier: MIT
-->

# Release process

Releases of `sql_query_analyzer` are driven by [release-plz][rplz].
Conventional commits land on `main`; release-plz reads them and opens (or
updates) a "release PR" that bumps `Cargo.toml` and prepends a section to
`CHANGELOG.md`. That is all the automatic path ever does — **the workflow
never publishes anything by itself**.

Publishing requires two deliberate human actions:

1. **merge the release PR** — the reviewed changelog lands on `main`;
2. **run the `Release-plz` workflow manually** (`workflow_dispatch`) with
   the `publish` checkbox enabled — only then does `release-plz release`
   tag the commit, publish to crates.io, and create the GitHub release.

`release_always = false` in `release-plz.toml` makes the `release`
command act only on commits that belong to a merged `release-plz-*`
branch, so even a manual run cannot publish a version that never came
through a reviewed release PR. A feature PR that bumps the version
manually (breaking changes must, to satisfy the `Semver Checks` gate)
does not publish on merge either — the bumped, unpublished version simply
flows into the next release PR. Publish promptly after merging: until the
publish run tags the version, `release-pr` keeps the release PR open for
that same version.

[rplz]: https://release-plz.dev

## Versioning policy

The crate follows [Semantic Versioning 2.0.0](https://semver.org/spec/v2.0.0.html)
with Cargo's `0.x` interpretation:

- While the major is `0`, **breaking changes** (`feat!:`) bump the minor.
- **New features** (`feat:`) also bump the minor — `features_always_increment_minor = true`,
  matching the established cadence where every batch of new rules shipped
  as a minor (`0.14 -> 0.15 -> 0.16`).
- **Bug fixes** and other non-feature commits bump the patch.

After 1.0 the standard major/minor/patch rules apply.

`Semver Checks` (`cargo semver-checks`, the `_semver.yml` job of `ci.yml`)
runs on every PR and blocks a release whose API change exceeds the
version bump. release-plz reads the same conventional commits to decide
the bump, so by the time a release PR exists the version and the API diff
already agree.

## How a release happens

```
conventional commit on main
        |
        v
Release-plz workflow (on push to main)
   `release-pr` only -- opens or updates the single open release PR;
   nothing is tagged, nothing is published
        |
        v
release PR -- Cargo.toml bump + CHANGELOG.md section (label: release)
        |
        v  (maintainer reviews, CI runs, semver gate passes)
merge  [human action 1]
        |
        v
Release-plz workflow_dispatch with `publish` checked  [human action 2]
   `release` publishes to crates.io, tags vX.Y.Z, creates the GitHub release
        |
        +--> Release binaries    -- per-platform archives + provenance
        +--> Release attestations -- .crate + CycloneDX SBOM + provenance
```

If a publish run fails partway, re-run the `Release-plz` workflow with
`publish` checked — `release-plz release` is idempotent. Binary assets can
be rebuilt for an existing tag via `workflow_dispatch` on `Release
binaries`; attestations likewise on `Release attestations`.

## One-time setup

The `Release-plz` workflow authenticates as a dedicated GitHub App
(`RELEASE_PLZ_APP_ID` + `RELEASE_PLZ_APP_PRIVATE_KEY` repository
secrets), not as the default `GITHUB_TOKEN`: PRs and releases created
with the default token do not trigger downstream workflows, so the
release would not be fully automatic. The App needs exactly two
permissions — `Contents: read & write` and `Pull requests: read &
write` — and is installed on this repository only. `CRATES_IO_TOKEN`
must also be set as a repository secret.

Everything else the pipeline needs is self-service: the `release` label
is created by the workflow itself before release-plz runs. The only
manual steps are the two secrets above — GitHub deliberately keeps
App-key provisioning outside automation — and they are set once.

## Changelog style

`CHANGELOG.md` follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/)
and the section headings (`Added`, `Fixed`, `Changed`, ...) are produced
by the `[changelog]` table of `release-plz.toml`, which mirrors the
previous git-cliff configuration. Commit subjects must be Conventional
Commits (`feat:`/`fix:`/`docs:`/...); merge commits carrying `(#123)`
get an automatic issue link.
