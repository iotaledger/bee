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

## 0.3.0 - 2021-11-25

### Added

- New PeerRelation variant: Discovered;
- Handling of discovered peers;

### Changed

- Renamed crate to `bee-gossip` and moved it into `bee-network` parent directory;

## 0.2.2 - 2021-08-26

### Changed

- Update dependencies;

## 0.2.1 - 2021-05-06

### Changed

- Rename `gossip` module to `iota_gossip` module;
- Overhaul of the whole `iota_gossip` module;
- Use `BufReader` and `BufWriter` for network I/O;
- Use `V1Lazy` variant for optimized outbound protocol negotiation;

### Removed

- Some redundant abstractions;

### Fixed

- Incoming connections from "unknown" peers not denied if limit was reached;

## 0.2.0 - 2021-04-29

### Added

- Types;
  - `NetworkConfig` and `NetworkConfigBuilder` types;
  - `Command` and `NetworkCommandSender` types;
  - `Event` and `NetworkEventReceiver` types;
  - `GossipReceiver` and `GossipSender` types;
  - `Origin` type;

- Modules;
  - `integrated` module with static `init` function and `NetworkService` and `NetworkHost` types;
  - `standalone` module with static `init` function;

## 0.1.0 - 2021-04-19

### Added

- Types;
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
