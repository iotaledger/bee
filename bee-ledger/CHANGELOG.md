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

## 0.3.1 - 2021-05-12

### Added

- `Balance::dust_allowed`;

## 0.3.0 - 2021-05-12

### Added

- `SnapshotWorker` and associated types and operations;
- `ConsensusWorker` (White Flag) and associated types and operations;
- `BalanceDiffs::{negate, negated, output_add, output_sub}`;
- `impl core::fmt::{Debug, Display} for Unspent`;

### Removed

- Moved `ConflictReason` to bee-tangle;

## 0.2.0 - 2021-04-26

### Added

- Snapshot types;
  - `SnapshotHeader`;
  - `FullSnapshotHeader`;
  - `DeltaSnapshotHeader`;
  - `SnapshotInfo`;
  - `SnapshotKind`;
  - `MilestoneDiff`;

## 0.1.0 - 2021-04-20

### Added

- Ledger types;
  - `BalanceDiff`;
  - `BalanceDiffs`;
  - `Balance`;
  - `ConflictReason`;
  - `ConsumedOutput`;
  - `CreatedOutput`;
  - `LedgerIndex`;
  - `Migration`;
  - `OutputDiff`;
  - `Receipt`;
  - `TreasuryDiff`;
  - `TreasuryOutput`;
  - `Unspent`;
