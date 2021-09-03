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

## 0.5.0 - 2021-06-15

### Changed

- `MultiFetch::multi_fetch` returns an `Iterator` instead of a `Vec`;

## 0.4.0 - 2021-06-10

### Changed

- All interfaces have been made sync;

## 0.3.0 - 2021-06-08

### Changed

- `MultiFetch::multi_fetch`;
  - from `async fn multi_fetch(&self, keys: &[K]) -> Result<Vec<Option<V>>, Self::Error>;`;
  - to `async fn multi_fetch(&self, keys: &[K]) -> Result<Vec<Result<Option<V>, Self::Error>>, Self::Error>;`;

## 0.2.0 - 2021-05-27

### Added

- Implementation of the `MultiFetch` trait for all bee storable types;

### Changed

- `AsStream::Stream`
  - from  `Stream<Item = (K, V)> + Send + Sync + Unpin;`
  - to    `Stream<Item = Result<(K, V), Self::Error>> + Send + Sync + Unpin;`

## 0.1.0 - 2021-04-27

### Added

- Implementation of `StorageBackend` for `RocksDB`;
- Implementation of the following traits for all bee storable types:
  - `Batch`;
  - `Delete`;
  - `Exist`;
  - `Fetch`;
  - `Insert`;
  - `AsStream`;
  - `Truncate`;
