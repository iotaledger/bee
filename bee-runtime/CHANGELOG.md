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

## 0.1.2-alpha - 2021-09-30

### Changed

- `TaskSpawner` method and unnecessary task instrumentation in favour of `tokio::spawn`;

## 0.1.1-alpha - 2021-02-12

### Added

- `NodeInfo` type;
- `Node::info` method;

## 0.1.0-alpha - 2021-01-08

### Added

- `Node` and `NodeBuilder` traits
- `Worker` trait
- `ResourceHandle` resource handle
- `Bus` event bus
- `ShutdownStream` stream
