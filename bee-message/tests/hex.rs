// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_message::{hex::FromHexPrefix, Error};
use primitive_types::U256;

#[test]
fn from_str_bytes() {
    assert_eq!(
        <[u8; 3] as FromHexPrefix>::from_hex_prefix("0xffffff"),
        Ok([255, 255, 255])
    );
}

#[test]
fn from_str_u256() {
    assert_eq!(
        U256::from_hex_prefix("0x1027000000000000000000000000000000000000000000000000000000000000"),
        Ok(U256::from(10000))
    );
}

// TODO: Fix
#[test]
fn from_str_u256_truncated() {
    assert_eq!(U256::from_hex_prefix("0x1027"), Ok(U256::from(10000)));
}

#[test]
fn from_str_invalid_hex() {
    assert_eq!(
        <[u8; 3] as FromHexPrefix>::from_hex_prefix("0x52fd6y"),
        Err(Error::HexInvalidHexCharacter { c: 'y', index: 7 })
    );
}

#[test]
fn from_str_invalid_length_too_short() {
    assert_eq!(
        <[u8; 3] as FromHexPrefix>::from_hex_prefix("0x52fd6"),
        Err(Error::HexInvalidStringLengthSlice { expected: 8, actual: 6 })
    );
}

#[test]
fn from_str_invalid_length_too_long() {
    assert_eq!(
        <[u8; 3] as FromHexPrefix>::from_hex_prefix("0x52fd643"),
        Err(Error::HexInvalidStringLengthSlice { expected: 8, actual: 6 })
    );
}

#[test]
fn from_str_invalid_length_slice() {
    assert_eq!(
        <[u8; 3] as FromHexPrefix>::from_hex_prefix("0x52fd64"),
        Err(Error::HexInvalidStringLengthSlice { expected: 8, actual: 6 })
    );
}

#[test]
fn from_str_no_prefix() {
    assert_eq!(
        <[u8; 3] as FromHexPrefix>::from_hex_prefix("ffffff"),
        Err(Error::HexInvalidPrefix { c0: 'f', c1: 'f' })
    );
}

#[test]
fn from_str_wrong_prefix() {
    assert_eq!(
        <[u8; 3] as FromHexPrefix>::from_hex_prefix("0yffffff"),
        Err(Error::HexInvalidPrefix { c0: '0', c1: 'y' })
    );
}

// TODO test truncating 0s (different for bytes vs integers)
