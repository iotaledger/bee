# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

<!-- ## Unreleased - YYYY-MM-DD

### Added

LoggerConfigBuilder::with_output

### Changed

### Deprecated

### Removed

- Removed `event::Bus` in favour of its inclusion in `bee-runtime`
- Removed `shutdown_stream::ShutdownStream` in favour of its inclusion in `bee-runtime`
- Removed `shutdown::Shutdown`
- Removed `worker::Worker`

### Fixed

### Security -->

## 0.2.0-alpha - 2021-01-04

### Added

- Impl `Packable` for `Vec<T: Packable>`;
- `target_filters` option to the logger;
- Event bus;
- `ShutdownStream::split`;

### Changed

- `ShutdownStream::from_fused` takes a `future::Fuse<oneshot::Receiver<()>>` instead of a `oneshot::Receiver<()>`;

## 0.1.1-alpha - 2020-11-12

### Added

- Impl `Packable` for `bool`;
- Impl `Packable` for `Option<P: Packable>`;

### Changed

- Make `pack_new` return a `Vec<u8>` instead of a `Result`;
- Require `Packable::Error` to be `Debug`;

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
