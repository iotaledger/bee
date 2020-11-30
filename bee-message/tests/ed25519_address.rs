// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_message::prelude::*;

use core::str::FromStr;

const ED25519_ADDRESS: &str = "52fdfc072182654f163f5f0f9a621d729566c74d10037c4d7bbb0407d1e2c649";

#[test]
fn generate_address() {
    let address = Address::from(Ed25519Address::new([1; 32]));
    match address {
        Address::Ed25519(a) => assert_eq!(a.len(), 32),
        _ => panic!("Expect Ed25519 address"),
    }
}

#[test]
fn generate_bech32_string() {
    let mut bytes = [0; 32];
    let vec = hex::decode(ED25519_ADDRESS).unwrap();
    bytes.copy_from_slice(&vec);
    let address = Ed25519Address::new(bytes);
    let bech32_string = address.to_bech32();
    assert_eq!(
        bech32_string,
        "iot1q9f0mlq8yxpx2nck8a0slxnzr4ef2ek8f5gqxlzd0wasgp73utryjtzcp98"
    );
}

#[test]
fn from_to_str() {
    assert_eq!(
        ED25519_ADDRESS,
        Ed25519Address::from_str(ED25519_ADDRESS).unwrap().to_string()
    );
}
