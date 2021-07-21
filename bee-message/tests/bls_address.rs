// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_message::{
    address::{Address, BlsAddress, Ed25519Address, BLS_ADDRESS_LENGTH},
    error::ValidationError,
};
use bee_packable::Packable;

use core::{convert::TryInto, str::FromStr};

const BLS_ADDRESS: &str =
    "58501b4f0c2d569f3db0c09c85c74c5f9fbf1e89bc30aae12f02212a86a6412ca69903941fa870ff64c01697d7c62b2cd7";
const BLS_ADDRESS_INVALID_HEX: &str =
    "58501b4f0c2d569f3db0c09c85c74c5f9fbf1e89bc30aae12f02212a86a6412ca69903941fa870ff64c01697d7c62b2cdx";
const BLS_ADDRESS_INVALID_LEN_TOO_SHORT: &str =
    "58501b4f0c2d569f3db0c09c85c74c5f9fbf1e89bc30aae12f02212a86a6412ca69903941fa870ff64c01697d7c62b2c";
const BLS_ADDRESS_INVALID_LEN_TOO_LONG: &str =
    "58501b4f0c2d569f3db0c09c85c74c5f9fbf1e89bc30aae12f02212a86a6412ca69903941fa870ff64c01697d7c62b2cd700";

#[test]
fn kind() {
    assert_eq!(BlsAddress::KIND, 1);
}

#[test]
fn display_impl() {
    assert_eq!(format!("{}", BlsAddress::from_str(BLS_ADDRESS).unwrap()), BLS_ADDRESS);
}

#[test]
fn debug_impl() {
    assert_eq!(
        format!("{:?}", BlsAddress::from_str(BLS_ADDRESS).unwrap()),
        "BlsAddress(".to_owned() + BLS_ADDRESS + ")"
    );
}

#[test]
fn from_str_valid() {
    BlsAddress::from_str(BLS_ADDRESS).unwrap();
}

#[test]
fn from_str_invalid_hex() {
    assert!(matches!(
        Ed25519Address::from_str(BLS_ADDRESS_INVALID_HEX),
        Err(ValidationError::InvalidHexadecimalChar(hex))
            if hex == BLS_ADDRESS_INVALID_HEX
    ));
}

#[test]
fn from_str_invalid_len_too_short() {
    assert!(matches!(
        BlsAddress::from_str(BLS_ADDRESS_INVALID_LEN_TOO_SHORT),
        Err(ValidationError::InvalidHexadecimalLength(expected, actual))
            if expected == BLS_ADDRESS_LENGTH * 2 && actual == BLS_ADDRESS_LENGTH * 2 - 2
    ));
}

#[test]
fn from_str_invalid_len_too_long() {
    assert!(matches!(
        BlsAddress::from_str(BLS_ADDRESS_INVALID_LEN_TOO_LONG),
        Err(ValidationError::InvalidHexadecimalLength(expected, actual))
            if expected == BLS_ADDRESS_LENGTH * 2 && actual == BLS_ADDRESS_LENGTH * 2 + 2
    ));
}

#[test]
fn from_to_str() {
    assert_eq!(BLS_ADDRESS, BlsAddress::from_str(BLS_ADDRESS).unwrap().to_string());
}

#[test]
fn try_from_bech32() {
    let addr = Address::Bls(BlsAddress::from_str(BLS_ADDRESS).unwrap());

    assert_eq!(addr, addr.to_bech32("atoi").try_into().unwrap());
}

#[test]
fn packed_len() {
    let address = BlsAddress::from_str(BLS_ADDRESS).unwrap();

    assert_eq!(address.packed_len(), 49);
    assert_eq!(address.pack_to_vec().unwrap().len(), 49);
}

#[test]
fn packable_round_trip() {
    let address = BlsAddress::from_str(BLS_ADDRESS).unwrap();
    let packed_address = address.pack_to_vec().unwrap();

    assert_eq!(address, BlsAddress::unpack_from_slice(packed_address).unwrap());
}
