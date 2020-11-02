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

## 0.1.0-alpha - 2020-11-02

### Added

- Logger backend based on [fern](https://crates.io/crates/fern) for the [log](https://crates.io/crates/log) crate;
- `Packable` trait to pack and unpack types to and from bytes;
- `ShutdownStream` helper to join a shutdown receiver and a regular stream;
- Shutdown mechanism to deal with the graceful shutdown of asynchronous workers;
- Worker `Error`;

### Changed

### Deprecated

### Removed

### Fixed

### Security
