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

## 1.0.0 - 2022-09-26

### Changed

- Update dependencies;

## 1.0.0-beta.1 - 2022-08-29

### Changed

- Type of `target_score` parameter of `NonceProvider::nonce` from `f64` to `u32` to better match TIP32;
- Updated dependencies;

### Fixed

- Clippy warning;

## 1.0.0-alpha.1 - 2022-07-15

First alpha release.

## 0.2.0 - 2021-11-19

### Changed

- Scoring of Proof of Work can now reuse hash functions;

## 0.1.0 - 2021-04-13

### Added

- Proof of Work scoring functions;
- NonceProviderBuilder/NonceProvider traits;
- MinerBuilder/Miner nonce provider;
- u64 nonce provider;
