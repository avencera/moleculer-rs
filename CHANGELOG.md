# Changelog

## [0.4.0] – 2024-10-02

- Updated broken dependencies that were preventing compilation on newer versions of Rust, thanks to [@isaac-nls](https://github.com/isaac-nls), in [#27](https://github.com/avencera/moleculer-rs/pull/27)
- Fix clippy warnings
- Upgrade `async-nats` to `0.37` from `0.10`

## [0.3.5] – 2021-08-03

- Updated async-nats dependency

## [0.3.4] – 2021-07-15

- Updated dependencies and ran clippy-fix

## [0.3.3] – 2021-03-25

- Fix build status badge in README

## [0.3.2] – 2021-03-25

- Fix examples in docs

## [0.3.1] – 2021-03-25

- Update README

## [0.3.0] – 2021-03-24

- Replace custom `ConfigBuilder` with derived one
- Remove `bytes` library from deps

## [0.2.0] – 2021-03-24

- Improved cargo docs
- Created `CHANGELOG.md`
- Adjusted visibility, default to `pub(crate)` when `pub` not needed
- Add `call()` function to `Context`

## [0.1.1] – 2021-03-23

- Updated license in cargo.toml

## [0.1.0] – 2021-03-23

- Initial release
