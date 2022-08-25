# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

<!-- ## Unreleased - YYYY-MM-DD

### Added

### Changed

### Deprecated

### Removed

### Fixed

### Security -->

## Unreleased - 2022-MM-DD

### Changed

- Updated dependencies (including `packable`);
- Restrict constraint to unlock an Alias address to Alias state transitions;

## 1.0.0-beta.6 - 2022-08-11

### Added

- `NativeTokensBuilder::finish_vec`;

### Changed

- Updated dependencies;

## 1.0.0-beta.5 - 2022-07-27

### Fixed

- `rand` feature;

## 1.0.0-beta.4 - 2022-07-26

### Changed

- Bump `inx` to `v1.0.0-beta.3`;

## 1.0.0-beta.3 - 2022-07-21

### Added

- Added conversions for `inx` types;
- `ProtocolParameters::new` and getters;

## 1.0.0-beta.2 - 2022-07-20

### Added

- `ProtocolParameters` type;

### Changed

- Add "No Native Tokens" rule for storage deposit returns;
- Rename `ByteCost*` to `Rent*`;
- Moved random generation of types from `bee-test` to `rand` module within crate;

### Fixed

- Add expiration check for input storage deposit returns selection;

## 1.0.0-beta.1 - 2022-07-19

Initial implementation of the `Block` related TIPs.
