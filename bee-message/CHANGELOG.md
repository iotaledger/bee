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

## 0.1.3 - 2021-04-22

### Changed

- `HashedIndex` replaced by `PaddedIndex`;
- `IndexationPayload::hash()` replaced by `IndexationPayload::padded_index()`;

## 0.1.2 - 2021-04-18

### Removed

- `ConsumedOutput`;
- `CreatedOutput`;

## 0.1.1 - 2021-04-18

### Fixed

- `MilestonePayload` unpacking;

## 0.1.0 - 2021-04-16

### Added

- First implementation of the Message RFC;
  - `address` module;
  - `input` module;
  - `milestone` module;
  - `output` module;
  - `payload` module;
  - `signature` module;
  - `unlock` module;
  - `message` module;
  - `parents` module;
