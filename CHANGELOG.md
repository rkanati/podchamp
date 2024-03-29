# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.5.1] - 2022-01-11
### Changed
- Moved to rust edition 2021.
- Updated dependencies.

## [0.5.0] - 2022-01-11
### Changed
- Updated dependencies.
### Fixed
- Updated `clap` dependency from pre-release version that broke our build.
- Actually run database migrations.

## [0.4.0] - 2021-04-29
### Changed
- Updated dependencies.
### Added
- Feeds are now fetched concurrently.
- A single running instance is ensured with a lock directory.

## [0.3.0] - 2021-01-06
### Changed
- Updated dependencies, including Tokio to 1.0.
### Removed
- dotenv support.
### Fixed
- `mod ... backlog` no longer lies in its success message.

## [0.2.1] - 2020-11-08
### Changed
- Updated dependencies.

## [0.2.0] - 2020-10-30
### Added
- Support for Atom and JSON feeds.

## [0.1.0] - 2020-10-28
Initial release.

[Unreleased]: https://github.com/rkanati/podchamp/tree/master
[0.5.1]: https://github.com/rkanati/podchamp/releases/tag/0.5.1
[0.5.0]: https://github.com/rkanati/podchamp/releases/tag/0.5.0
[0.4.0]: https://github.com/rkanati/podchamp/releases/tag/0.4.0
[0.3.0]: https://github.com/rkanati/podchamp/releases/tag/0.3.0
[0.2.1]: https://github.com/rkanati/podchamp/releases/tag/0.2.1
[0.2.0]: https://github.com/rkanati/podchamp/tree/662f12ec382167d0f458272c26102d38d50f1577
[0.1.0]: https://github.com/rkanati/podchamp/tree/06aeee5a1b5d37ba537c5295c9e2c35f0c873e2a

