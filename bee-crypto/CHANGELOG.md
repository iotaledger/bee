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

## 0.1.2-alpha - 2020-08-19

### Fixed

- Fix `try_from` method of `Hash`

## 0.1.1-alpha - 2020-08-11

### Changed

- `CurlP` uses `unsafe` access to its truth table for performance improvements;

## 0.1.0-alpha - 2020-07-23

### Added

- Ternary big integer utilities;
- Ternary cryptographic `Sponge` trait;
- `CurlP` ternary cryptographic function and its variants `CurlP27` and `CurlP81`;
- `Kerl` ternary cryptographic function;
