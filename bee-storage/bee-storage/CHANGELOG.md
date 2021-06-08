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

## 0.6.0 - 2021-06-07

### Added

- `crate::system::version`;

### Changed

- Moved `crate::health` to `crate::system`;

## 0.5.0 - 2021-05-27

### Changed

- `AsStream::Stream`
  - from  `Stream<Item = (K, V)> + Send + Sync + Unpin;`
  - to    `Stream<Item = Result<(K, V), Self::Error>> + Send + Sync + Unpin;`

## 0.4.0 - 2021-05-25

### Added

- `MultiFetch` trait;

## 0.3.0 - 2021-04-13

### Added

- `StorageHealth` type;
- `StorageBackend::get_health()` method;
- `StorageBackend::set_health()` method;

## 0.2.0-alpha - 2021-01-11

### Added

- `AsStream::Stream: Send + Sync + Unpin` bounds;
- `StorageBackend::size()` method;

### Changed

- `StorageBackend` methods return `Result<_, Self::Error>` instead of `Result<_, Box<dyn std::error::Error>>`;

## 0.1.0-alpha - 2021-01-06

### Added

- `StorageBackend` trait;
- `BatchBuilder`, `Batch`, `Delete`, `Exist`, `Fetch`, `Insert`, `AsStream` and `Truncate` traits;
