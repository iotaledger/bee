// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_common::packable::Packable;
use bee_message::prelude::*;

use core::{convert::TryInto, str::FromStr};

const ED25519_ADDRESS: &str = "52fdfc072182654f163f5f0f9a621d729566c74d10037c4d7bbb0407d1e2c649";
const ED25519_ADDRESS_INVALID_HEX: &str = "52fdfc072182654f163f5f0f9a621d729566c74d10037c4d7bbb0407d1e2c64x";
const ED25519_ADDRESS_INVALID_LEN_TOO_SHORT: &str = "52fdfc072182654f163f5f0f9a621d729566c74d10037c4d7bbb0407d1e2c6";
const ED25519_ADDRESS_INVALID_LEN_TOO_LONG: &str = "52fdfc072182654f163f5f0f9a621d729566c74d10037c4d7bbb0407d1e2c64900";

#[test]
fn kind() {
    assert_eq!(Ed25519Address::KIND, 0);
}

#[test]
fn generate_address() {
    match Address::from(Ed25519Address::new([1; 32])) {
        Address::Ed25519(a) => assert_eq!(a.len(), 32),
        _ => panic!("Expect Ed25519 address"),
    }
}

#[test]
fn from_str_valid() {
    Ed25519Address::from_str(ED25519_ADDRESS).unwrap();
}

#[test]
fn from_str_invalid_hex() {
    assert!(matches!(
        Ed25519Address::from_str(ED25519_ADDRESS_INVALID_HEX),
        Err(Error::InvalidHexadecimalChar(hex))
            if hex == ED25519_ADDRESS_INVALID_HEX
    ));
}

#[test]
fn from_str_invalid_len_too_short() {
    assert!(matches!(
        Ed25519Address::from_str(ED25519_ADDRESS_INVALID_LEN_TOO_SHORT),
        Err(Error::InvalidHexadecimalLength(expected, actual))
            if expected == ED25519_ADDRESS_LENGTH * 2 && actual == ED25519_ADDRESS_LENGTH * 2 - 2
    ));
}

#[test]
fn from_str_invalid_len_too_long() {
    assert!(matches!(
        Ed25519Address::from_str(ED25519_ADDRESS_INVALID_LEN_TOO_LONG),
        Err(Error::InvalidHexadecimalLength(expected, actual))
            if expected == ED25519_ADDRESS_LENGTH * 2 && actual == ED25519_ADDRESS_LENGTH * 2 + 2
    ));
}

#[test]
fn from_to_str() {
    assert_eq!(
        ED25519_ADDRESS,
        Ed25519Address::from_str(ED25519_ADDRESS).unwrap().to_string()
    );
}

#[test]
fn try_from_bech32() {
    let addr = Address::Ed25519(Ed25519Address::from_str(ED25519_ADDRESS).unwrap());

    assert_eq!(addr, addr.to_bech32("atoi").try_into().unwrap());
}

#[test]
fn packed_len() {
    let address = Ed25519Address::from_str(ED25519_ADDRESS).unwrap();

    assert_eq!(address.packed_len(), 32);
    assert_eq!(address.pack_new().len(), 32);
}

#[test]
fn pack_unpack_valid() {
    let address = Ed25519Address::from_str(ED25519_ADDRESS).unwrap();
    let packed_address = address.pack_new();

    assert_eq!(address, Packable::unpack(&mut packed_address.as_slice()).unwrap());
}
