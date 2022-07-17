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

<!-- ## Unreleased - YYYY-MM-DD

### Removed

- Moved base types to crate `bee-ledger-types`;

-->

## 0.2.2 - 2022-03-07

### Changed

- Bump deps: 
  + `bee-autopeering` ~> 0.5.0;
  + `bee-gossip` ~> 0.6.0;

## 0.2.1 - 2022-02-28

### Changed

- Peer manager handles new bee-gossip `PeerUnreachable` event;

## 0.2.0 - 2022-01-27

### Added

- All protocol workers;

## 0.1.1 - 2021-08-26

- Update dependencies;

### Changed

## 0.1.0 - 2021-04-20

### Added

- Protocol types;
  - `NodeMetrics`;
  - `PeerMetrics`;
  - `MilestoneKeyManager`;
  - `MilestoneKeyRange`;
  - `Peer`;
