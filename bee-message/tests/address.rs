// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_message::{
    address::{Address, AddressUnpackError, BlsAddress, Ed25519Address},
    util::hex_decode,
    MessageUnpackError,
};
use bee_packable::{Packable, UnpackError};
use bee_test::rand::bytes::rand_bytes_array;

use core::str::FromStr;

const ED25519_ADDRESS: &str = "52fdfc072182654f163f5f0f9a621d729566c74d10037c4d7bbb0407d1e2c649";
const BLS_ADDRESS: &str =
    "f30d5856a165c1cf27d088300e9506858c2f05d307d518be4c08b90935b983f733cfcaa80d6199a0062b4ccbe1c398f265";

// TODO add verify tests

#[test]
fn display_impl() {
    assert_eq!(
        format!("{}", Address::from(Ed25519Address::from_str(ED25519_ADDRESS).unwrap())),
        ED25519_ADDRESS
    );
    assert_eq!(
        format!("{}", Address::from(BlsAddress::from_str(BLS_ADDRESS).unwrap())),
        BLS_ADDRESS
    );
}

#[test]
fn debug_impl() {
    assert_eq!(
        format!(
            "{:?}",
            Address::from(Ed25519Address::from_str(ED25519_ADDRESS).unwrap())
        ),
        "Ed25519Address(".to_owned() + ED25519_ADDRESS + ")"
    );
    assert_eq!(
        format!("{:?}", Address::from(BlsAddress::from_str(BLS_ADDRESS).unwrap())),
        "BlsAddress(".to_owned() + BLS_ADDRESS + ")"
    );
}

#[test]
fn from_ed25519() {
    let ed25519_address = Ed25519Address::new(rand_bytes_array());
    let address = Address::from(ed25519_address);

    assert_eq!(address.kind(), 0);
    assert_eq!(address.length(), Ed25519Address::LENGTH);
    assert!(matches!(address, Address::Ed25519(address) if {address == ed25519_address}));
}

#[test]
fn from_bls() {
    let bls_address = BlsAddress::new(rand_bytes_array());
    let address = Address::from(bls_address);

    assert_eq!(address.kind(), 1);
    assert_eq!(address.length(), BlsAddress::LENGTH);
    assert!(matches!(address, Address::Bls(address) if {address == bls_address}));
}

#[test]
fn packed_len() {
    let address = Address::from(Ed25519Address::new(hex_decode(ED25519_ADDRESS).unwrap()));

    assert_eq!(address.packed_len(), 1 + 32);
    assert_eq!(address.pack_to_vec().unwrap().len(), 1 + 32);
}

#[test]
fn packable_round_trip() {
    let address_1 = Address::from(Ed25519Address::new(hex_decode(ED25519_ADDRESS).unwrap()));
    let address_2 = Address::unpack_from_slice(address_1.pack_to_vec().unwrap()).unwrap();

    assert_eq!(address_1, address_2);
}

#[test]
fn unpack_invalid_kind() {
    assert!(matches!(
        Address::unpack_from_slice(vec![
            0x04, 0x1d, 0x38, 0x9e, 0xa2, 0x7a, 0x77, 0xc9, 0x1d, 0x08, 0x40, 0xf9, 0x38, 0x61, 0x44, 0x2a, 0x95, 0xca,
            0x5e, 0x88, 0x2e, 0x0d, 0x9f, 0x9c, 0x2f, 0x99, 0x65, 0x81, 0x54, 0x09, 0xd9, 0x39, 0xe4
        ])
        .err()
        .unwrap(),
        UnpackError::Packable(MessageUnpackError::Address(AddressUnpackError::InvalidKind(4))),
    ));
}

#[test]
fn serde_round_trip() {
    let address_1 = Address::from(Ed25519Address::new(hex_decode(ED25519_ADDRESS).unwrap()));
    let json = serde_json::to_string(&address_1).unwrap();
    let address_2 = serde_json::from_str::<Address>(&json).unwrap();

    assert_eq!(address_1, address_2);
}
