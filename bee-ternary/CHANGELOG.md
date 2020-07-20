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

## 0.3.0-alpha - 2020-07-20

### Added

- Support for arbitrary trit to numeric type conversion

## 0.2.0-alpha - 2020-07-17

### Added

- Binary/ternary numeric conversion
- FromStr implementation for TryteBuf
- TritBuf::from_i8s and TritBuf::from_u8s

## 0.1.0-alpha - 2020-06-12

### Added

- Efficient manipulation of ternary buffers (trits and trytes).
- Multiple encoding schemes.
- Extensible design that allows it to sit on top of existing data structures, avoiding unnecessary allocation and copying.
- An array of utility functions to allow for easy manipulation of ternary data.
- Zero-cost conversion between trit and tryte formats (i.e: no slower than the equivalent code would be if hand-written).
