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

## 0.3.0 - 2021-11-17

### Deprecated

- Deprecated `CurlP` from `bee-crypto` (now uses `iota-crypto` instead);
- Deprecated `Sponge` and `SpongeKind`;

## 0.2.1-alpha - 2021-04-07

### Changed

- Transpose binary encoded trit representation to be more performant;
- Make `BatchedHasher` generic over the trit encoding;
- Rename types to have them compliant with Rust naming conventions;

## 0.2.0-alpha - 2020-08-20

### Added

- Batched version of `CurlP`;

## 0.1.1-alpha - 2020-08-11

### Changed

- `CurlP` uses `unsafe` access to its truth table for performance improvements;

## 0.1.0-alpha - 2020-07-23

### Added

- Ternary big integer utilities;
- Ternary cryptographic `Sponge` trait;
- `CurlP` ternary cryptographic function and its variants `CurlP27` and `CurlP81`;
- `Kerl` ternary cryptographic function;
