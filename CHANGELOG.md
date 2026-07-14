# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### CI/CD

- Bump the github-actions group across 1 directory with 2 updates ([953c181](https://github.com/RAprogramm/sql-query-analyzer/commit/953c1810f86a85a45917d60e05b1129b6a99b296))
- Bump the github-actions group across 1 directory with 5 updates ([2e6e200](https://github.com/RAprogramm/sql-query-analyzer/commit/2e6e200b399bc1de74f09cf84b7b6fee053bc388))

### Miscellaneous

- New banner ([d5a2d7c](https://github.com/RAprogramm/sql-query-analyzer/commit/d5a2d7cfd8d46456bd6de326a5325f869ac4bcc6))

### Deps

- Bump the rust-dependencies group across 1 directory with 2 updates ([1593445](https://github.com/RAprogramm/sql-query-analyzer/commit/15934456fb68a517d752d26e606e8c2a12a5b276))
- Bump bytes from 1.11.0 to 1.11.1 ([49fd5be](https://github.com/RAprogramm/sql-query-analyzer/commit/49fd5be8b393e293f51b1aecadbd4394ebd45ea8))
- Bump colored from 3.0.0 to 3.1.1 in the rust-dependencies group ([35f30d3](https://github.com/RAprogramm/sql-query-analyzer/commit/35f30d319b7d5a7c05551e875540557a71b50c18))
- Bump the rust-dependencies group with 4 updates ([66a94a1](https://github.com/RAprogramm/sql-query-analyzer/commit/66a94a1291c7ff78d3273a6bc2bf41ce76a2ffa2))
- Bump the rust-dependencies group with 2 updates ([9402f64](https://github.com/RAprogramm/sql-query-analyzer/commit/9402f64e56784b370c1f6232c17ce2bf364fb60e))
- Bump the rust-dependencies group with 3 updates ([78b4db0](https://github.com/RAprogramm/sql-query-analyzer/commit/78b4db0eae0b54c2c0e9150cfbaf8c2d98a26783))

## [0.5.2] - 2025-12-19

### Changed

- Move CLI logic to app.rs for testability ([323de1e](https://github.com/RAprogramm/sql-query-analyzer/commit/323de1ef501c2c01e506645cd0448c317ec107f6))

### Documentation

- Add ClickHouse support documentation ([0ed8828](https://github.com/RAprogramm/sql-query-analyzer/commit/0ed88284c7381998c1f4bd650eaca678deb718e0))

### Fixed

- Update deprecated cargo_bin, bump version 0.5.1 ([d1c420a](https://github.com/RAprogramm/sql-query-analyzer/commit/d1c420ade4a95880fa4b50c895dece398cce7627))

## [0.4.1] - 2025-12-18

### CI/CD

- Extract MSRV from Cargo.toml dynamically ([7b36bfe](https://github.com/RAprogramm/sql-query-analyzer/commit/7b36bfe9d0fbab2e8d611e61d26fcc54e855b464))
- Bump the github-actions group across 1 directory with 4 updates ([a592a67](https://github.com/RAprogramm/sql-query-analyzer/commit/a592a67a4a53f988cba98d375fdfe7b25360d0ee))

## [0.4.0] - 2025-11-30

### CI/CD

- Enable codecov for pull requests ([934ab8a](https://github.com/RAprogramm/sql-query-analyzer/commit/934ab8a62ccea13be741e83aed29a3506f81c49a))
- Enable CI for all pull requests ([6b30471](https://github.com/RAprogramm/sql-query-analyzer/commit/6b30471fd4a913181f4464dd867101fd4ff9f118))

### Changed

- Professional PR comment UI with GitHub alerts and collapsible sections ([7772e5f](https://github.com/RAprogramm/sql-query-analyzer/commit/7772e5f2190c9b7ffb3fcfc1569f5011fa8e354a))

### Fixed

- Assign explicit discriminants to QueryType enum for semver stability ([dc99f89](https://github.com/RAprogramm/sql-query-analyzer/commit/dc99f8949fc67579b5cd77fad834a7fc595ba6a1))

## [0.3.0] - 2025-11-28

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
- Enforce PR size check before lints and qual after clippy ([ef3f26a](https://github.com/RAprogramm/sql-query-analyzer/commit/ef3f26a7ea05d1ea0a253a16c9458ef417102627))
- Fix deny licenses and remove unused dependencies ([d639d9e](https://github.com/RAprogramm/sql-query-analyzer/commit/d639d9eced635a3c2ce17fe459e8051275544274))
- Add comprehensive quality gates and fix doctests ([f19dc9d](https://github.com/RAprogramm/sql-query-analyzer/commit/f19dc9d2bdbbb7e76adffd5816d4d999cd5f2150))
- Bump the github-actions group with 5 updates ([55f6edb](https://github.com/RAprogramm/sql-query-analyzer/commit/55f6edba22258b4e3b044ab50acb09af74f28bf4))
- Add dependabot configuration ([3fa8726](https://github.com/RAprogramm/sql-query-analyzer/commit/3fa87264595d6cc68dbe89f05b06a9b493f47a78))
- Add PR size check with 200 lines limit ([7afb7ac](https://github.com/RAprogramm/sql-query-analyzer/commit/7afb7acd2df2c157dd952b29fbee2d73e55dbd2a))
- Add PR size analysis with 200 lines limit ([da5d368](https://github.com/RAprogramm/sql-query-analyzer/commit/da5d368f88cd28074234844e552a4c640c4948c3))

### Changed

- Fix all cargo-qual issues and add quality CI job ([50c472a](https://github.com/RAprogramm/sql-query-analyzer/commit/50c472a10fad6b9475bf5e3ae59bda8041fc227a))

### Documentation

- Update changelog [skip ci] ([2112bbc](https://github.com/RAprogramm/sql-query-analyzer/commit/2112bbcf19778d93e300ff6535b453e7f49910c4))
- Add required permissions to GitHub Action example ([0ddc595](https://github.com/RAprogramm/sql-query-analyzer/commit/0ddc59516d1aa7118d2036bbc02ec1d622eb6f7b))
- Add static analysis example without LLM ([50ecb6d](https://github.com/RAprogramm/sql-query-analyzer/commit/50ecb6d8d409c0d653c007fa0c998f7be057b978))
- Update README with GitHub Action usage ([1865165](https://github.com/RAprogramm/sql-query-analyzer/commit/18651656ca2779ea8205becd49332cc8290108a4))
- Add contributing guidelines with PR size limits ([16f6e7f](https://github.com/RAprogramm/sql-query-analyzer/commit/16f6e7f3c364ccfbe75d2da18c1f304bff702030))
- Add REUSE compliance badge ([b7f838a](https://github.com/RAprogramm/sql-query-analyzer/commit/b7f838a3d2945352495cc17429eddfa92edb9e3f))
- Update changelog [skip ci] ([8160321](https://github.com/RAprogramm/sql-query-analyzer/commit/8160321282ba46a04e7f8b810e85621a92d6f3da))

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

[Unreleased]: https://github.com/RAprogramm/sql-query-analyzer/compare/v0.5.2...HEAD
[0.5.2]: https://github.com/RAprogramm/sql-query-analyzer/compare/v0.4.1...v0.5.2
[0.4.1]: https://github.com/RAprogramm/sql-query-analyzer/compare/v0.4.0...v0.4.1
[0.4.0]: https://github.com/RAprogramm/sql-query-analyzer/compare/v0.3.0...v0.4.0
[0.3.0]: https://github.com/RAprogramm/sql-query-analyzer/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/RAprogramm/sql-query-analyzer/compare/v0.1.1...v0.2.0
[0.1.1]: https://github.com/RAprogramm/sql-query-analyzer/compare/v1...v0.1.1
[1]: https://github.com/RAprogramm/sql-query-analyzer/compare/v0.1.0...v1
[0.1.0]: https://github.com/RAprogramm/sql-query-analyzer/releases/tag/v0.1.0

