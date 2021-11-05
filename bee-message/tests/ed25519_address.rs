// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_message::address::{Address, Bech32Address, Ed25519Address};
use bee_packable::{Packable, PackableExt};

use core::{ops::Deref, str::FromStr};

const ED25519_ADDRESS: &str = "1d389ea27a77c91d0840f93861442a95ca5e882e0d9f9c2f9965815409d939e4";

// TODO: add `verify` tests

#[test]
fn kind() {
    assert_eq!(Ed25519Address::KIND, 0);
}

#[test]
fn length() {
    assert_eq!(Ed25519Address::LENGTH, 32);
}

#[test]
fn display_impl() {
    assert_eq!(
        format!("{}", Ed25519Address::from_str(ED25519_ADDRESS).unwrap()),
        ED25519_ADDRESS
    );
}

#[test]
fn debug_impl() {
    assert_eq!(
        format!("{:?}", Ed25519Address::from_str(ED25519_ADDRESS).unwrap()),
        "Ed25519Address(".to_owned() + ED25519_ADDRESS + ")"
    );
}

#[test]
fn new_as_ref() {
    assert_eq!(
        Ed25519Address::new([42; Ed25519Address::LENGTH]).as_ref(),
        &[42; Ed25519Address::LENGTH]
    );
}

#[test]
fn new_deref() {
    assert_eq!(
        Ed25519Address::new([42; Ed25519Address::LENGTH]).deref(),
        &[42; Ed25519Address::LENGTH]
    );
}

#[test]
fn from_as_ref() {
    assert_eq!(
        Ed25519Address::from([42; Ed25519Address::LENGTH]).as_ref(),
        &[42; Ed25519Address::LENGTH]
    );
}

#[test]
fn from_str_as_ref() {
    assert_eq!(
        Ed25519Address::from_str(ED25519_ADDRESS).unwrap().as_ref(),
        &[
            0x1d, 0x38, 0x9e, 0xa2, 0x7a, 0x77, 0xc9, 0x1d, 0x08, 0x40, 0xf9, 0x38, 0x61, 0x44, 0x2a, 0x95, 0xca, 0x5e,
            0x88, 0x2e, 0x0d, 0x9f, 0x9c, 0x2f, 0x99, 0x65, 0x81, 0x54, 0x09, 0xd9, 0x39, 0xe4
        ]
    );
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
    let bech32 = Bech32Address::from_address("atoi", &addr);

    assert_eq!(addr, bech32.try_into().unwrap());
}

#[test]
fn packed_len() {
    let address = Ed25519Address::from_str(ED25519_ADDRESS).unwrap();

    assert_eq!(address.packed_len(), Ed25519Address::LENGTH);
    assert_eq!(address.pack_to_vec().len(), Ed25519Address::LENGTH);
}

#[test]
fn packable_round_trip() {
    let address_1 = Ed25519Address::from_str(ED25519_ADDRESS).unwrap();
    let address_2 = Ed25519Address::unpack_from_slice(address_1.pack_to_vec()).unwrap();

    assert_eq!(address_1, address_2);
}

#[test]
fn serde_round_trip() {
    let ed25519_address_1 = Ed25519Address::from_str(ED25519_ADDRESS).unwrap();
    let json = serde_json::to_string(&ed25519_address_1).unwrap();
    let ed25519_address_2 = serde_json::from_str::<Ed25519Address>(&json).unwrap();

    assert_eq!(ed25519_address_1, ed25519_address_2);
}
