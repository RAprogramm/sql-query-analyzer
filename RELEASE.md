<!--
SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
SPDX-License-Identifier: MIT
-->

# Release process

Releases of `sql_query_analyzer` are driven by [release-plz][rplz].
Conventional commits land on `main`; release-plz reads them and opens (or
updates) a "release PR" that bumps `Cargo.toml` and prepends a section to
`CHANGELOG.md`. Merging that PR is the only human action a release needs:
the push it produces runs `release-plz release`, which tags the commit,
publishes to crates.io, and creates the GitHub release.

`release_always = false` in `release-plz.toml` makes the `release`
command act only on commits that belong to a merged `release-plz-*`
branch, so a push that is not a merged release PR publishes nothing.
Feature PRs never bump the version: release-plz computes every bump
itself in its release PR, and the `Semver Checks` gate runs only on that
PR, where the version is already bumped.

If a publish run fails, re-run the `Release-plz` workflow from `main`
(`workflow_dispatch`): `release-plz release` is idempotent, so a re-run
is always safe. See "Recovering a partial release" below for the case
where some of the effects were already produced.

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
   `release` -- publishes only if the pushed commit belongs to a merged
                release PR, otherwise a no-op
   `release-pr` -- opens or updates the single open release PR
        |
        v
release PR -- Cargo.toml bump + CHANGELOG.md section (label: release)
        |
        v  (maintainer reviews, CI runs, semver gate passes)
merge  [the only human action]
        |
        v
Release-plz workflow (push produced by the merge)
   `release` publishes to crates.io, tags vX.Y.Z, creates the GitHub release
        |
        +--> Release binaries    -- per-platform archives + provenance
        +--> Release attestations -- .crate + CycloneDX SBOM + provenance
```

## Recovering a partial release

If a publish run fails partway, re-running `Release-plz` from `main` is
sufficient while nothing was published yet: `release-plz release` skips
a version that already exists on crates.io and a tag that already
exists, so the missing later effects are never created by a retry.
Complete the missing steps by hand instead:

1. Crate published, tag missing — release-plz can publish from the release
   PR's head commit rather than the merge commit, so the packaged revision
   is recorded in the crate, not in the merge commit. Read the `git.sha1`
   field from `.cargo_vcs_info.json` inside the published crate:

   ```sh
   tar -xOf sql_query_analyzer-X.Y.Z.crate \
     sql_query_analyzer-X.Y.Z/.cargo_vcs_info.json
   ```

   Confirm the commit is reachable from `main`
   (`git merge-base --is-ancestor <sha> origin/main`), then tag it and
   push: `git tag -a vX.Y.Z <sha> && git push origin vX.Y.Z`.
2. Tag pushed, GitHub release missing — create it from the changelog
   section: copy the `## [X.Y.Z]` section of `CHANGELOG.md` into a file
   and run `gh release create vX.Y.Z --notes-file <section.md>`.
3. In every partial state, re-attach the artefacts by dispatching
   `Release binaries` and `Release attestations` with that tag.

Binary assets can be rebuilt for an existing tag via `workflow_dispatch`
on `Release binaries`; attestations likewise on `Release attestations`.

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
