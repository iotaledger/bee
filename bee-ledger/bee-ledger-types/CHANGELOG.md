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

## 1.0.0-beta.6 - 2022-XX-XX

### Changed

- Updated dependencies;
- `ProtocolParameters` from `()` to `Packable::UnpackVisitor` for `MilestoneDiff` and `CreatedOutput`;
- `Receipt::validate` new parameter `token_supply: u64`;
- Adapt `rand` module to these changes;

## 1.0.0-beta.5 - 2022-08-30

### Changed

- Updated dependencies (including `packable`);

## 1.0.0-beta.4 - 2022-08-15

### Changed

- Updated dependencies;

## 1.0.0-beta.3 - 2022-07-21

### Changed

- Bump `bee-block` dependency;

## 1.0.0-beta.2 - 2022-07-20

### Changed

- Bump `bee-block` dependency;
- Moved random generation of types from `bee-test` to `rand` module within crate;

## 1.0.0-beta.1 - 2022-07-19

First beta release.
