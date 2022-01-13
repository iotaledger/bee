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

## 0.1.1 - 2022-01-13

### Fixed

- server panics when sending to an IPv6 address;
- spams port mismatch warnings
- filters out valid incoming peering requests 

## 0.1.0 - 2021-12-03

### Added

- Local entity and peer identities;
- Peer discovery;
- Neighbor selection;
- Packet/Message handling;
- Network I/O;
- Configuration;