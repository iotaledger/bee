// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_common::packable::Packable;
use bee_message::prelude::*;

use core::convert::TryInto;

const ED25519_ADDRESS: &str = "52fdfc072182654f163f5f0f9a621d729566c74d10037c4d7bbb0407d1e2c649";

// The kind of an `Address` is the kind of the underlying address.
#[test]
fn kind() {
    let bytes: [u8; 32] = hex::decode(ED25519_ADDRESS).unwrap().try_into().unwrap();
    let ed25519_address = Address::from(Ed25519Address::new(bytes));

    assert_eq!(0, ed25519_address.kind());
}

#[test]
fn generate_bech32_string() {
    let bytes: [u8; 32] = hex::decode(ED25519_ADDRESS).unwrap().try_into().unwrap();
    let address = Address::from(Ed25519Address::new(bytes));
    let bech32_string = address.to_bech32("iota");

    assert_eq!(
        bech32_string,
        "iota1qpf0mlq8yxpx2nck8a0slxnzr4ef2ek8f5gqxlzd0wasgp73utryj430ldu"
    );
}

#[test]
fn generate_bech32_testnet_string() {
    let bytes: [u8; 32] = hex::decode(ED25519_ADDRESS).unwrap().try_into().unwrap();
    let address = Address::from(Ed25519Address::new(bytes));
    let bech32_string = address.to_bech32("atoi");

    assert_eq!(
        bech32_string,
        "atoi1qpf0mlq8yxpx2nck8a0slxnzr4ef2ek8f5gqxlzd0wasgp73utryjjl77h3"
    );
}

#[test]
fn bech32_string_to_address() {
    let bytes: [u8; 32] = hex::decode(ED25519_ADDRESS).unwrap().try_into().unwrap();
    let address = Address::try_from_bech32(&Address::from(Ed25519Address::new(bytes)).to_bech32("iota")).unwrap();

    if let Address::Ed25519(ed) = address {
        assert_eq!(
            ed.to_string(),
            "52fdfc072182654f163f5f0f9a621d729566c74d10037c4d7bbb0407d1e2c649"
        );
    } else {
        panic!("Expecting an Ed25519 address");
    }
    let address = Address::try_from_bech32(&Address::from(Ed25519Address::new(bytes)).to_bech32("atoi")).unwrap();
    if let Address::Ed25519(ed) = address {
        assert_eq!(
            ed.to_string(),
            "52fdfc072182654f163f5f0f9a621d729566c74d10037c4d7bbb0407d1e2c649"
        );
    } else {
        panic!("Expecting an Ed25519 address");
    }
}

#[test]
fn pack_unpack_valid_ed25519() {
    let bytes: [u8; 32] = hex::decode(ED25519_ADDRESS).unwrap().try_into().unwrap();
    let address = Address::from(Ed25519Address::new(bytes));
    let address_packed = address.pack_new();

    assert_eq!(address_packed.len(), address.packed_len());
    assert_eq!(address, Packable::unpack(&mut address_packed.as_slice()).unwrap());
}
