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

## 0.4.1-alpha - 2021-03-15

### Fixed

- T6B1 decoding.

## 0.4.0-alpha - 2021-01-18

### Added

- T6B1 bytes-as-trits encoding and decoding support.

## 0.3.4-alpha - 2020-11-13

### Added

- Added proper `i128`/`u128` support detection.

## 0.3.3-alpha - 2020-11-06

### Fixed

- `TryFrom<Trits>` implemented for `u128` and `i128` only when `cfg(has_i128)`.

## 0.3.2-alpha - 2020-10-19

### Added

- `with_capacity` constructor for the buffers of every trit encoding.

## 0.3.1-alpha - 2020-07-23

### Added

- Conversions between `&[Trit]` and `&Trits<T1B1<T>>`.

### Removed

- A useless conversion to same type.

## 0.3.0-alpha - 2020-07-20

### Added

- Support for arbitrary trit to numeric type conversion.

## 0.2.0-alpha - 2020-07-17

### Added

- Binary/ternary numeric conversion.
- FromStr implementation for TryteBuf.
- TritBuf::from_i8s and TritBuf::from_u8s.

## 0.1.0-alpha - 2020-06-12

### Added

- Efficient manipulation of ternary buffers (trits and trytes).
- Multiple encoding schemes.
- Extensible design that allows it to sit on top of existing data structures, avoiding unnecessary allocation and copying.
- An array of utility functions to allow for easy manipulation of ternary data.
- Zero-cost conversion between trit and tryte formats (i.e: no slower than the equivalent code would be if hand-written).
