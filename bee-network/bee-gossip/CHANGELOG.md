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

## 1.0.0-beta.2 - 2022-08-15

### Changed

- Updated dependencies;

### Fixed

- Clippy warnings;

## 1.0.0-beta.1 - 2022-07-20

First beta release.

## 0.6.0 - 2022-03-07

### Changed

- Bump deps: 
  + `libp2p-core` ~> 0.30;
  + `libp2p` ~> 0.41.0;

## 0.5.0 - 2022-02-28

### Added

- `PeerMetrics` type that keeps a count of dial attempts and identification timestamp;
- `PeerUnreachable` event that is fired after a certain number of unsuccessful dial attempts;

### Changed

- `PeerList` type keeps metrics for each peer;

### Fixed

- Missing `PeerRemoved` event for disconnected unknown peers;

## 0.4.0 - 2022-01-20

### Fixed

- Autopeering integration;

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
