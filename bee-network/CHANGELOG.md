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

## 0.1.0 - 2021-04-19

### Added

- Types exposed to clients libraries;
  - `PeerInfo` type;
  - `PeerRelation` type;

- Re-Exports;
  - `libp2p::core::identity::ed25519::KeyPair` type;
  - `libp2p::core::identity::ed25519::PublicKey` type;
  - `libp2p::multiaddr::Protocol` type;
  - `libp2p::Multiaddr` type;
  - `libp2p::PeerId` type;

- Macros;
  - `alias!` macro;