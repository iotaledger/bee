// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_message::address::{Address, Bech32Address, BlsAddress};
use bee_packable::Packable;

use core::{convert::TryInto, ops::Deref, str::FromStr};

const BLS_ADDRESS: &str =
    "f30d5856a165c1cf27d088300e9506858c2f05d307d518be4c08b90935b983f733cfcaa80d6199a0062b4ccbe1c398f265";

#[test]
fn kind() {
    assert_eq!(BlsAddress::KIND, 1);
}

#[test]
fn length() {
    assert_eq!(BlsAddress::LENGTH, 49);
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
fn new_as_ref() {
    assert_eq!(
        BlsAddress::new([42; BlsAddress::LENGTH]).as_ref(),
        &[42; BlsAddress::LENGTH]
    );
}

#[test]
fn new_deref() {
    assert_eq!(
        BlsAddress::new([42; BlsAddress::LENGTH]).deref(),
        &[42; BlsAddress::LENGTH]
    );
}

#[test]
fn from_as_ref() {
    assert_eq!(
        BlsAddress::from([42; BlsAddress::LENGTH]).as_ref(),
        &[42; BlsAddress::LENGTH]
    );
}

#[test]
fn from_str_as_ref() {
    assert_eq!(
        BlsAddress::from_str(BLS_ADDRESS).unwrap().as_ref(),
        &[
            0xf3, 0x0d, 0x58, 0x56, 0xa1, 0x65, 0xc1, 0xcf, 0x27, 0xd0, 0x88, 0x30, 0x0e, 0x95, 0x06, 0x85, 0x8c, 0x2f,
            0x05, 0xd3, 0x07, 0xd5, 0x18, 0xbe, 0x4c, 0x08, 0xb9, 0x09, 0x35, 0xb9, 0x83, 0xf7, 0x33, 0xcf, 0xca, 0xa8,
            0x0d, 0x61, 0x99, 0xa0, 0x06, 0x2b, 0x4c, 0xcb, 0xe1, 0xc3, 0x98, 0xf2, 0x65
        ]
    );
}

#[test]
fn from_to_str() {
    assert_eq!(BLS_ADDRESS, BlsAddress::from_str(BLS_ADDRESS).unwrap().to_string());
}

#[test]
fn try_from_bech32() {
    let addr = Address::Bls(BlsAddress::from_str(BLS_ADDRESS).unwrap());
    let bech32 = Bech32Address::from_address("atoi", &addr);

    assert_eq!(addr, bech32.try_into().unwrap());
}

#[test]
fn packed_len() {
    let address = BlsAddress::from_str(BLS_ADDRESS).unwrap();

    assert_eq!(address.packed_len(), BlsAddress::LENGTH);
    assert_eq!(address.pack_to_vec().len(), BlsAddress::LENGTH);
}

#[test]
fn packable_round_trip() {
    let address_1 = BlsAddress::from_str(BLS_ADDRESS).unwrap();
    let address_2 = BlsAddress::unpack_from_slice(address_1.pack_to_vec()).unwrap();

    assert_eq!(address_1, address_2);
}

#[test]
fn serde_round_trip() {
    let bls_address_1 = BlsAddress::from_str(BLS_ADDRESS).unwrap();
    let json = serde_json::to_string(&bls_address_1).unwrap();
    let bls_address_2 = serde_json::from_str::<BlsAddress>(&json).unwrap();

    assert_eq!(bls_address_1, bls_address_2);
}
