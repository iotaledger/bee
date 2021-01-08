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

## 0.1.0-alpha - 2021-01-08

### Added

- Networking layer built on top of libp2p, that allows sending/receiving byte messages;
- Authenticated connections via Noise protocol;
- Multiplexed connections via Mplex protocol;
- Maintained peers and connections via Peer- and ConnectionManager;
- Interaction via commands and events;
- Auto-reconnect;
- Custom behavior for known and unknown peers;
