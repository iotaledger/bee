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

## 1.0.0 - 2022-09-27

### Added

- `helper` module with a `network_name_to_id` function;
- `Error::NetworkIdMismatch`;
- `ProtocolParameters::network_id` method;
- `BlockBuilder::with_protocol_version`;
- `Output::verify_storage_deposit` new parameter `token_supply: u64`;

### Changed

- Updated dependencies;
- `block` module is now public;
- `Packable::UnpackVisitor` from `()` to `ProtocolParameters` for a lot of types;
- `ProtocolParameters::version` renamed to `ProtocolParameters::protocol_version`;
- Some DTO `TryFrom` have been changed to functions as they needed another parameters;
- `Output`s amount is now simply an `u64`;
- `OutputBuilder`s `finish` now takes a `token_supply: u64` parameter; 
- Adapt the `rand` module to all these changes;
- All DTO conversion free functions have been made type methods;
- `DEFAULT_BYTE_COST` from 500 to 100;
- Implement `Default` for `ProtocolParameters` and `RentStructure`;
- Return `U256` instead of `&U256` for `NativeToken` amounts;

### Removed

- `constant` module;
- `OutputAmount`, `StorageDepositAmount`, `TreasuryOutputAmount` and `MigratedFundsAmount`;
- `target_score` parameters from `BlockBuilder::with_nonce_provider`;

## 1.0.0-beta.7 - 2022-08-30

### Changed

- Updated dependencies (including `packable`);
- Restrict constraint to unlock an Alias address to Alias state transitions;
- Use new packable version with `Packable::UnpackVisitor`;

## 1.0.0-beta.6 - 2022-08-11

### Added

- `NativeTokensBuilder::finish_vec`;

### Changed

- Updated dependencies;

## 1.0.0-beta.5 - 2022-07-27

### Fixed

- `rand` feature;

## 1.0.0-beta.4 - 2022-07-26

### Changed

- Bump `inx` to `v1.0.0-beta.3`;

## 1.0.0-beta.3 - 2022-07-21

### Added

- Added conversions for `inx` types;
- `ProtocolParameters::new` and getters;

## 1.0.0-beta.2 - 2022-07-20

### Added

- `ProtocolParameters` type;

### Changed

- Add "No Native Tokens" rule for storage deposit returns;
- Rename `ByteCost*` to `Rent*`;
- Moved random generation of types from `bee-test` to `rand` module within crate;

### Fixed

- Add expiration check for input storage deposit returns selection;

## 1.0.0-beta.1 - 2022-07-19

Initial implementation of the `Block` related TIPs.
