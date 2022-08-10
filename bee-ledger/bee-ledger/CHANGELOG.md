# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

<!-- ## Unreleased - YYYY-MM-DD

### Added

### Changed

- Bump `bee-block` to `v1.0.0-beta.4`;

### Deprecated

### Removed

### Fixed

### Security -->

<!-- ## Unreleased - YYYY-MM-DD

### Removed

- Moved base types to crate `bee-ledger-types`;

-->

## Unreleased - 2022-XX-XX

### Changed

- Updated dependencies;

## 0.8.0 - 2022-XX-XX

### Added

- `CreatedOutput::milestone_index`, `CreatedOutput::milestone_timestamp` and their getters;

### Changed

- Complete refactoring of White Flag with Tokenization and Smart Contracts layouts;
- `BalanceDiff` is now an `u64` tuple struct;
- `Balance` is now an `u64` tuple struct;
- Serialize and deserialize the ledger types using `packable` instead of `bee-common::packable`;
- Dispatch `CreatedOutput` through `OutputCreated` event;

### Removed

- `Balance`, `BalanceDiff` and `BalanceDiffs`;
- `Error::InvalidLedgerDustState`;
- Pruning of `PaddedIndex`;

## 0.7.0 - 2022-XX-XX

### Changed

- Update dependencies;

### Fixed

- Emit warning when parsing snapshot header fails instead of crashing;

## 0.6.1 - 2021-12-29

### Changed

- Don't ignore source when header download of the delta snapshot fails;

## 0.6.0 - 2021-12-07

### Added

- Download snapshots by most recent ledger index;
- Derive `Clone` for `Balance`, `OutputDiff`, `TreasuryDiff` and `Unspent`;

### Changed

- Decouple snapshot names and download URLs;
- Reduced number of dependencies features;
- Update bee-tangle version and change `MsTangle` to `Tangle`;

## 0.5.0 - 2021-08-30

### Added

- `pruning` module;
- `snapshot::condition` module;
- `SnapshotInfo` methods;
  - `update_snapshot_index`;
  - `update_entry_point_index`;
  - `update_pruning_index`;
  - `update_timestamp`;

### Changed

- `consensus::worker` module to execute pruning if conditions are met;

## 0.4.0 - 2021-06-01

### Added

- `ConsensusWorkerCommand::{FetchBalance, FetchOutput, FetchOutputs}`;

### Changed

- `struct ConsensusWorkerEvent` to `enum ConsensusWorkerCommand`;

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
