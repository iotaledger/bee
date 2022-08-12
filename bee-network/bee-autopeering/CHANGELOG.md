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

## Unreleased - 2022-XX-XX

### Changed

- Updated dependencies;

## 0.5.0 - 2022-03-07

### Changed

- Bump dep: `libp2p-core` ~> 0.30.0;

## 0.4.1 - 2022-02-28

### Changed

- Displayed representation of `PeerId`s to start at the beginning of its significant part;

## 0.4.0 - 2022-02-11

### Added

- IPv6 support;

## 0.3.0 - 2022-01-20

### Added

- `rocksdb` as backend for the peer store;

## 0.2.0 - 2022-01-20

### Changed

- Improved logged messages;

### Fixed

- Slow in finding first peers;
- Accepting invalid peering requests;

## 0.1.1 - 2022-01-13

### Fixed

- Server panics when sending to an IPv6 address;
- Spams port mismatch warnings;
- Filters out valid incoming peering requests;

## 0.1.0 - 2021-12-03

### Added

- Local entity and peer identities;
- Peer discovery;
- Neighbor selection;
- Packet/Message handling;
- Network I/O;
- Configuration;
