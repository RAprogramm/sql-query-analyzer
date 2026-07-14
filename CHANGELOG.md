# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Add PERF019 rule detecting large IN clauses ([#103](https://github.com/RAprogramm/sql-query-analyzer/issues/103)) ([4ad7e9b](https://github.com/RAprogramm/sql-query-analyzer/commit/4ad7e9b07e762b8c9d76e8d368d9a7fdf91a1947))

### Documentation

- Update changelog [skip ci] ([66bad57](https://github.com/RAprogramm/sql-query-analyzer/commit/66bad57b3fc6a98ea181197cab0c07174fad3d56))

## [0.8.0] - 2026-07-14

### Added

- Add SEC007 rule detecting dynamic SQL execution ([#102](https://github.com/RAprogramm/sql-query-analyzer/issues/102)) ([f3957e8](https://github.com/RAprogramm/sql-query-analyzer/commit/f3957e8b287e3b502ff64d07d93fbbe5e85e57d5))

### Documentation

- Update changelog [skip ci] ([67f0845](https://github.com/RAprogramm/sql-query-analyzer/commit/67f0845984a4297c889ffb78fbdd4a42bbb58e70))

## [0.7.0] - 2026-07-14

### Added

- Add SEC005 rule detecting GRANT/REVOKE privilege changes ([#99](https://github.com/RAprogramm/sql-query-analyzer/issues/99)) ([efbddd8](https://github.com/RAprogramm/sql-query-analyzer/commit/efbddd87e90fd0e59a66c7b6980256b5a0d1f746))

### CI/CD

- Cancel in-progress runs only for pull requests ([#101](https://github.com/RAprogramm/sql-query-analyzer/issues/101)) ([a318ba4](https://github.com/RAprogramm/sql-query-analyzer/commit/a318ba47939661b536c0550651a12eb3e33279ef))

### Documentation

- Update changelog [skip ci] ([e233647](https://github.com/RAprogramm/sql-query-analyzer/commit/e2336476716b9fdcde8ea79170faa62c7b599889))

## [0.6.0] - 2026-07-14

### Added

- Add PERF012 rule detecting COUNT(*) without WHERE ([#98](https://github.com/RAprogramm/sql-query-analyzer/issues/98)) ([07d0bfd](https://github.com/RAprogramm/sql-query-analyzer/commit/07d0bfd71377fde567c5e4b1f1906dc56f1b3041))
- Add SEC008 rule detecting hardcoded credentials ([#95](https://github.com/RAprogramm/sql-query-analyzer/issues/95)) ([f9d96f4](https://github.com/RAprogramm/sql-query-analyzer/commit/f9d96f4c09421cb66e68b24ad11ced60bc83c830))
- Add SEC006 rule detecting SQL injection tautologies ([#94](https://github.com/RAprogramm/sql-query-analyzer/issues/94)) ([4682b3e](https://github.com/RAprogramm/sql-query-analyzer/commit/4682b3e9b5ec4a8a11b105dddb647a614b5adced))
- Add STYLE004 rule detecting ordinals in ORDER BY and GROUP BY ([#93](https://github.com/RAprogramm/sql-query-analyzer/issues/93)) ([1c24318](https://github.com/RAprogramm/sql-query-analyzer/commit/1c24318bd6353a061d4936bf1afb999e5797ebe0))
- Add PERF013 rule detecting ORDER BY RAND() ([#92](https://github.com/RAprogramm/sql-query-analyzer/issues/92)) ([ea547d8](https://github.com/RAprogramm/sql-query-analyzer/commit/ea547d81a767abd2d5ee26c85f5de7d46a391d2f))

### CI/CD

- Restore tag and changelog push credentials, unskip changelog job ([#97](https://github.com/RAprogramm/sql-query-analyzer/issues/97)) ([18ca01a](https://github.com/RAprogramm/sql-query-analyzer/commit/18ca01a26412dd599a3fe9f4d50c53de583d2080))
- Add scorecard, zizmor, codeql and release attestations ([#91](https://github.com/RAprogramm/sql-query-analyzer/issues/91)) ([940f52e](https://github.com/RAprogramm/sql-query-analyzer/commit/940f52e6949fea2306d68266fb041a3c337feb01))
- Unpin cargo-quality action back to v0 ([#87](https://github.com/RAprogramm/sql-query-analyzer/issues/87)) ([ab35eb5](https://github.com/RAprogramm/sql-query-analyzer/commit/ab35eb511a487fe3c891a13d7d4cd69793585f10))
- Automate dependency updates and fix audit check publishing ([#85](https://github.com/RAprogramm/sql-query-analyzer/issues/85)) ([13b06a6](https://github.com/RAprogramm/sql-query-analyzer/commit/13b06a6804d95981f46341f2c5140b64fc6c6e9c))
- Bump the github-actions group across 1 directory with 2 updates ([953c181](https://github.com/RAprogramm/sql-query-analyzer/commit/953c1810f86a85a45917d60e05b1129b6a99b296))
- Bump the github-actions group across 1 directory with 5 updates ([2e6e200](https://github.com/RAprogramm/sql-query-analyzer/commit/2e6e200b399bc1de74f09cf84b7b6fee053bc388))

### Dependencies

- Bump the rust-dependencies group across 1 directory with 2 updates ([1593445](https://github.com/RAprogramm/sql-query-analyzer/commit/15934456fb68a517d752d26e606e8c2a12a5b276))
- Bump bytes from 1.11.0 to 1.11.1 ([49fd5be](https://github.com/RAprogramm/sql-query-analyzer/commit/49fd5be8b393e293f51b1aecadbd4394ebd45ea8))
- Bump colored from 3.0.0 to 3.1.1 in the rust-dependencies group ([35f30d3](https://github.com/RAprogramm/sql-query-analyzer/commit/35f30d319b7d5a7c05551e875540557a71b50c18))
- Bump the rust-dependencies group with 4 updates ([66a94a1](https://github.com/RAprogramm/sql-query-analyzer/commit/66a94a1291c7ff78d3273a6bc2bf41ce76a2ffa2))
- Bump the rust-dependencies group with 2 updates ([9402f64](https://github.com/RAprogramm/sql-query-analyzer/commit/9402f64e56784b370c1f6232c17ce2bf364fb60e))
- Bump the rust-dependencies group with 3 updates ([78b4db0](https://github.com/RAprogramm/sql-query-analyzer/commit/78b4db0eae0b54c2c0e9150cfbaf8c2d98a26783))

### Documentation

- Publish mdBook documentation site on GitHub Pages ([#89](https://github.com/RAprogramm/sql-query-analyzer/issues/89)) ([699c9a4](https://github.com/RAprogramm/sql-query-analyzer/commit/699c9a4729648e933be23a1a9d052dc1d0126ff8))
- Add project banner to README ([#83](https://github.com/RAprogramm/sql-query-analyzer/issues/83)) ([f653dca](https://github.com/RAprogramm/sql-query-analyzer/commit/f653dca7eccc88536929ecb2a1456b7bd4e61910))

### Fixed

- Resolve cargo-quality findings ([#81](https://github.com/RAprogramm/sql-query-analyzer/issues/81)) ([c88306c](https://github.com/RAprogramm/sql-query-analyzer/commit/c88306c108ba064d1c6e776024f6c22206b2fdc5))

### Miscellaneous

- New banner ([d5a2d7c](https://github.com/RAprogramm/sql-query-analyzer/commit/d5a2d7cfd8d46456bd6de326a5325f869ac4bcc6))
- Update dependencies to latest, bump MSRV to 1.97 ([#79](https://github.com/RAprogramm/sql-query-analyzer/issues/79)) ([4e9c4cc](https://github.com/RAprogramm/sql-query-analyzer/commit/4e9c4ccb7febc799af15a711130d329cab1fbe88))

## [0.5.2] - 2025-12-19

### Added

- Add SQL preprocessor for ClickHouse dialect ([86845b6](https://github.com/RAprogramm/sql-query-analyzer/commit/86845b6c8c3d616dac0863fafce2f457de9a7655))

### Changed

- Split app.rs into submodules ([dfaf345](https://github.com/RAprogramm/sql-query-analyzer/commit/dfaf34530d3276110282cc924b8984d7aaa643cd))
- Move CLI logic to app.rs for testability ([323de1e](https://github.com/RAprogramm/sql-query-analyzer/commit/323de1ef501c2c01e506645cd0448c317ec107f6))

### Documentation

- Add ClickHouse support documentation ([0ed8828](https://github.com/RAprogramm/sql-query-analyzer/commit/0ed88284c7381998c1f4bd650eaca678deb718e0))

### Fixed

- Update deprecated cargo_bin, bump version 0.5.1 ([d1c420a](https://github.com/RAprogramm/sql-query-analyzer/commit/d1c420ade4a95880fa4b50c895dece398cce7627))

## [0.4.1] - 2025-12-18

### CI/CD

- Extract MSRV from Cargo.toml dynamically ([7b36bfe](https://github.com/RAprogramm/sql-query-analyzer/commit/7b36bfe9d0fbab2e8d611e61d26fcc54e855b464))
- Bump the github-actions group across 1 directory with 4 updates ([a592a67](https://github.com/RAprogramm/sql-query-analyzer/commit/a592a67a4a53f988cba98d375fdfe7b25360d0ee))

### Fixed

- Hide API key from help output ([4f38d06](https://github.com/RAprogramm/sql-query-analyzer/commit/4f38d06f471666752770b167c8c52065ab591a5a))

## [0.4.0] - 2025-11-30

### Added

- Implement ClickHouse CREATE TABLE parsing ([ba84477](https://github.com/RAprogramm/sql-query-analyzer/commit/ba844779a04e2d33747df3595e80f50a3be616bc))
- Add dialect parameter to Schema::parse ([d45657c](https://github.com/RAprogramm/sql-query-analyzer/commit/d45657c301e8fb7d4bec59b06ea6907ec3487ef4))
- Add codec field to ColumnInfo ([6acfd17](https://github.com/RAprogramm/sql-query-analyzer/commit/6acfd17e118f062589b0160d41124387d213ce67))

### CI/CD

- Enable codecov for pull requests ([934ab8a](https://github.com/RAprogramm/sql-query-analyzer/commit/934ab8a62ccea13be741e83aed29a3506f81c49a))
- Enable CI for all pull requests ([6b30471](https://github.com/RAprogramm/sql-query-analyzer/commit/6b30471fd4a913181f4464dd867101fd4ff9f118))

### Changed

- Fix cargo qual empty_lines issues ([7871034](https://github.com/RAprogramm/sql-query-analyzer/commit/78710345301ccc074ef13826438046acca58d2dc))
- Professional PR comment UI with GitHub alerts and collapsible sections ([7772e5f](https://github.com/RAprogramm/sql-query-analyzer/commit/7772e5f2190c9b7ffb3fcfc1569f5011fa8e354a))

### Fixed

- Check entire project in cargo qual CI ([a12c930](https://github.com/RAprogramm/sql-query-analyzer/commit/a12c93013efeda165872d8a5fef0627c4781ad48))
- Assign explicit discriminants to QueryType enum for semver stability ([dc99f89](https://github.com/RAprogramm/sql-query-analyzer/commit/dc99f8949fc67579b5cd77fad834a7fc595ba6a1))

### Testing

- Add ClickHouse query parsing tests ([a6ce97c](https://github.com/RAprogramm/sql-query-analyzer/commit/a6ce97c38561e26a752dc186890f7e27f59d6e74))
- Add ClickHouse CREATE TABLE parsing tests ([fc36d25](https://github.com/RAprogramm/sql-query-analyzer/commit/fc36d25f6c81e3b2ea4cc0634bfc114af78b50b0))

## [0.3.0] - 2025-11-28

### Added

- Add SEC003 TRUNCATE statement detection ([a96d5c1](https://github.com/RAprogramm/sql-query-analyzer/commit/a96d5c1ce89e2f9e3433117f6b214a357046051f))

### CI/CD

- Fix job conditions to properly evaluate reusable workflow outputs ([4465e67](https://github.com/RAprogramm/sql-query-analyzer/commit/4465e67f2c684e0e197bfcf43f14b71c629f3e51))
- Fix detect workflow for push events ([7d792bf](https://github.com/RAprogramm/sql-query-analyzer/commit/7d792bff6c8cd8c3e5cc3760f9ac08d0a6f30cea))

### Changed

- Extract app logic from main.rs for testability ([6801dff](https://github.com/RAprogramm/sql-query-analyzer/commit/6801dff65003880678f0171f9110b6df608dfb4b))

### Fixed

- Replace clone() with copy semantics for Copy types in tests ([cefbdf3](https://github.com/RAprogramm/sql-query-analyzer/commit/cefbdf3a9bc10721118790bf2b1a3da467e94b8a))

### Miscellaneous

- Trigger cache refresh ([6ab4651](https://github.com/RAprogramm/sql-query-analyzer/commit/6ab465195c3ccc41714a33c537bae0503d8760c3))

### Testing

- Improve test coverage to 78% ([9bc32f6](https://github.com/RAprogramm/sql-query-analyzer/commit/9bc32f6ac83f0af1605d18808cd69549e3444193))

## [0.2.0] - 2025-11-28

### CI/CD

- Add always() to release jobs, bump to 0.2.0 ([1560999](https://github.com/RAprogramm/sql-query-analyzer/commit/156099938a99604f9fcec7643c882d5ec376aa6e))
- Add always() to should-release, bump to 0.1.9 ([b3b5735](https://github.com/RAprogramm/sql-query-analyzer/commit/b3b5735da00af39272d57782f4d991211ff6a399))
- Use intermediate job for release condition, bump to 0.1.8 ([6282abd](https://github.com/RAprogramm/sql-query-analyzer/commit/6282abdd4edbc4fa3918ba9778148754742edf4d))
- Add debug outputs job, bump to 0.1.7 ([4b854d5](https://github.com/RAprogramm/sql-query-analyzer/commit/4b854d5ab6ebf723d3434d30ed10cb5cabc2cedb))
- Remove if conditions from reusable workflows, bump to 0.1.6 ([002af97](https://github.com/RAprogramm/sql-query-analyzer/commit/002af97c1231d3e8568fceb355deddf999f1cc67))
- Add debug for tag outputs, bump to 0.1.5 ([8932de3](https://github.com/RAprogramm/sql-query-analyzer/commit/8932de344da43bbb210ce85be44542bdae0456ec))
- Fix tag workflow outputs, bump to 0.1.4 ([fbe9832](https://github.com/RAprogramm/sql-query-analyzer/commit/fbe98326f06619ac730fb58792984cf495b0ea49))
- Trigger release ([f9dada9](https://github.com/RAprogramm/sql-query-analyzer/commit/f9dada9b4caa741769dcc309bf8f5a81ccd789e4))
- Add description to reusable workflow outputs ([7ae1366](https://github.com/RAprogramm/sql-query-analyzer/commit/7ae13661099ef3c94911b0ddf59a712e11771166))
- Auto-create tag and release on version bump ([6e83e62](https://github.com/RAprogramm/sql-query-analyzer/commit/6e83e62cf07b529798ff197253fc8a7e7e8c5953))
- Add permissions for quality check PR comments ([cd6c085](https://github.com/RAprogramm/sql-query-analyzer/commit/cd6c08570927b71305d8a814d73a901dfb43f5ed))
- Enforce PR size check before lints and qual after clippy ([ef3f26a](https://github.com/RAprogramm/sql-query-analyzer/commit/ef3f26a7ea05d1ea0a253a16c9458ef417102627))
- Fix deny licenses and remove unused dependencies ([d639d9e](https://github.com/RAprogramm/sql-query-analyzer/commit/d639d9eced635a3c2ce17fe459e8051275544274))
- Add comprehensive quality gates and fix doctests ([f19dc9d](https://github.com/RAprogramm/sql-query-analyzer/commit/f19dc9d2bdbbb7e76adffd5816d4d999cd5f2150))
- Bump the github-actions group with 5 updates ([55f6edb](https://github.com/RAprogramm/sql-query-analyzer/commit/55f6edba22258b4e3b044ab50acb09af74f28bf4))
- Add dependabot configuration ([3fa8726](https://github.com/RAprogramm/sql-query-analyzer/commit/3fa87264595d6cc68dbe89f05b06a9b493f47a78))
- Add PR size check with 200 lines limit ([7afb7ac](https://github.com/RAprogramm/sql-query-analyzer/commit/7afb7acd2df2c157dd952b29fbee2d73e55dbd2a))
- Add PR size analysis with 200 lines limit ([da5d368](https://github.com/RAprogramm/sql-query-analyzer/commit/da5d368f88cd28074234844e552a4c640c4948c3))

### Changed

- Split CI into reusable workflows ([75bc5d2](https://github.com/RAprogramm/sql-query-analyzer/commit/75bc5d259b5a541cfc56d56b6e44dd1ad8b44d65))
- Fix all cargo-qual issues and add quality CI job ([50c472a](https://github.com/RAprogramm/sql-query-analyzer/commit/50c472a10fad6b9475bf5e3ae59bda8041fc227a))

### Documentation

- Update changelog [skip ci] ([2112bbc](https://github.com/RAprogramm/sql-query-analyzer/commit/2112bbcf19778d93e300ff6535b453e7f49910c4))
- Add required permissions to GitHub Action example ([0ddc595](https://github.com/RAprogramm/sql-query-analyzer/commit/0ddc59516d1aa7118d2036bbc02ec1d622eb6f7b))
- Add static analysis example without LLM ([50ecb6d](https://github.com/RAprogramm/sql-query-analyzer/commit/50ecb6d8d409c0d653c007fa0c998f7be057b978))
- Update README with GitHub Action usage ([1865165](https://github.com/RAprogramm/sql-query-analyzer/commit/18651656ca2779ea8205becd49332cc8290108a4))
- Add contributing guidelines with PR size limits ([16f6e7f](https://github.com/RAprogramm/sql-query-analyzer/commit/16f6e7f3c364ccfbe75d2da18c1f304bff702030))
- Add REUSE compliance badge ([b7f838a](https://github.com/RAprogramm/sql-query-analyzer/commit/b7f838a3d2945352495cc17429eddfa92edb9e3f))
- Update changelog [skip ci] ([8160321](https://github.com/RAprogramm/sql-query-analyzer/commit/8160321282ba46a04e7f8b810e85621a92d6f3da))

### Fixed

- Use github format for pr-size limit checking ([fde1a41](https://github.com/RAprogramm/sql-query-analyzer/commit/fde1a410e77657b392f804d0ac855e3635f016e4))
- Add pull-requests write permission for pr-size ([c697525](https://github.com/RAprogramm/sql-query-analyzer/commit/c6975258f73a42fb60edb396f617380d893c448f))
- Use correct input names for rust-prod-diff-checker ([79188e2](https://github.com/RAprogramm/sql-query-analyzer/commit/79188e267fcc80ee4757203296671690d79c82fc))
- Reorder CI stages and fix permissions ([dd75f2a](https://github.com/RAprogramm/sql-query-analyzer/commit/dd75f2aa66cfbdd71e83c6434b2de838e0c6bb08))
- Move qual permissions to caller workflow ([5e8c70b](https://github.com/RAprogramm/sql-query-analyzer/commit/5e8c70b0bc2722f0f209a09d29a0fc1132e4e531))
- Rename binary to sql-query-analyzer ([1fcfb83](https://github.com/RAprogramm/sql-query-analyzer/commit/1fcfb839e042f34524ab47dfaea159170728ac01))

### Miscellaneous

- Bump version to 0.1.3 ([cbe1541](https://github.com/RAprogramm/sql-query-analyzer/commit/cbe1541db1c93cbcfabe3b4f58abf45fe1549058))

### Rm

- Mod.rs files ([9a872f1](https://github.com/RAprogramm/sql-query-analyzer/commit/9a872f1d42f0cdeb59695d314263933e864b9b33))

## [0.1.1] - 2025-11-24

### CI/CD

- Add codecov configuration ([9b6570d](https://github.com/RAprogramm/sql-query-analyzer/commit/9b6570d467039c9de591a5d2493769318610c79a))
- Add Codecov coverage and test results ([413bd59](https://github.com/RAprogramm/sql-query-analyzer/commit/413bd59e866874dc14b4e91085ad23db61688262))
- Skip crates.io publish for action tags ([11e990e](https://github.com/RAprogramm/sql-query-analyzer/commit/11e990ed48b8dde2fbba6bdc694df5d312db9901))

### Documentation

- Add table of contents and back to top links ([ba9a9be](https://github.com/RAprogramm/sql-query-analyzer/commit/ba9a9beca8162481de6d2017f99bf1f7fcb6d1af))
- Add coverage graphs section to README ([e5ed530](https://github.com/RAprogramm/sql-query-analyzer/commit/e5ed530b79059ffc840ea8ab8a27a7c607bea1ec))

### Fixed

- Add lib target for docs.rs ([426cdd1](https://github.com/RAprogramm/sql-query-analyzer/commit/426cdd16da5035409dfea6d95140a32a62444067))

## [1] - 2025-11-24

### Added

- Update GitHub Action for marketplace v1 ([b8f9186](https://github.com/RAprogramm/sql-query-analyzer/commit/b8f91864a3f7cc2a25929eca5f1e37c0f9831af2))

### Documentation

- Update changelog [skip ci] ([bcfb063](https://github.com/RAprogramm/sql-query-analyzer/commit/bcfb063495b7130e4e5e2f2ce3055ae34cd78351))

### Testing

- Add comprehensive unit tests ([616fd9f](https://github.com/RAprogramm/sql-query-analyzer/commit/616fd9f4284eadcd2f33138558eb5ae9e5cdab6b))

## [0.1.0] - 2025-11-24

### CI/CD

- Add changelog generation with git-cliff ([5de70af](https://github.com/RAprogramm/sql-query-analyzer/commit/5de70af8d8d6b2c703c97aa48f3d8e321b95df97))
- Add release workflow with crates.io publish ([b8b7856](https://github.com/RAprogramm/sql-query-analyzer/commit/b8b78562a1d53fbed9b4fc4fc815973d1f5875bc))

### Fixed

- Use rustls instead of openssl for cross-compilation ([2bdf030](https://github.com/RAprogramm/sql-query-analyzer/commit/2bdf03097c563bce1c7b70eb613f6d58bc52b0c2))

[Unreleased]: https://github.com/RAprogramm/sql-query-analyzer/compare/v0.8.0...HEAD
[0.8.0]: https://github.com/RAprogramm/sql-query-analyzer/compare/v0.7.0...v0.8.0
[0.7.0]: https://github.com/RAprogramm/sql-query-analyzer/compare/v0.6.0...v0.7.0
[0.6.0]: https://github.com/RAprogramm/sql-query-analyzer/compare/v0.5.2...v0.6.0
[0.5.2]: https://github.com/RAprogramm/sql-query-analyzer/compare/v0.4.1...v0.5.2
[0.4.1]: https://github.com/RAprogramm/sql-query-analyzer/compare/v0.4.0...v0.4.1
[0.4.0]: https://github.com/RAprogramm/sql-query-analyzer/compare/v0.3.0...v0.4.0
[0.3.0]: https://github.com/RAprogramm/sql-query-analyzer/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/RAprogramm/sql-query-analyzer/compare/v0.1.1...v0.2.0
[0.1.1]: https://github.com/RAprogramm/sql-query-analyzer/compare/v1...v0.1.1
[1]: https://github.com/RAprogramm/sql-query-analyzer/compare/v0.1.0...v1
[0.1.0]: https://github.com/RAprogramm/sql-query-analyzer/releases/tag/v0.1.0

