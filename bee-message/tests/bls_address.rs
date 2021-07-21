// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_message::{
    address::{Address, BlsAddress, Ed25519Address, BLS_ADDRESS_LENGTH},
    error::ValidationError,
};
use bee_packable::Packable;

use core::{convert::TryInto, str::FromStr};

const BLS_ADDRESS: &str =
    "f30d5856a165c1cf27d088300e9506858c2f05d307d518be4c08b90935b983f733cfcaa80d6199a0062b4ccbe1c398f265";
const BLS_ADDRESS_INVALID_HEX: &str =
    "xde2e06e169c49ba368296d913372d5ca24947037cbc44926b58b344ba24c0f546a817ce77f69939d203352545d25a267d";
const BLS_ADDRESS_INVALID_LEN_TOO_SHORT: &str =
    "78c7ace49002a75d3d4f7a63ee9c1de1caf3e4f45458d11bfdf084990122dc468685ac191b5916bcabff377e88659f1e";
const BLS_ADDRESS_INVALID_LEN_TOO_LONG: &str =
    "2292d4052c13baf5a61032416da12bcd58ae87ea4afe31642a0b9781b2b5c69f126e9ba94c50ded40ec34f2dd4eca41bfc00";

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
