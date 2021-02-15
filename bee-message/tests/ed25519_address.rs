// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_message::prelude::*;

use core::str::FromStr;

const ED25519_ADDRESS: &str = "52fdfc072182654f163f5f0f9a621d729566c74d10037c4d7bbb0407d1e2c649";
const ED25519_ADDRESS_INVALID_HEX: &str = "52fdfc072182654f163f5f0f9a621d729566c74d10037c4d7bbb0407d1e2c64x";
const ED25519_ADDRESS_INVALID_LEN: &str = "52fdfc072182654f163f5f0f9a621d729566c74d10037c4d7bbb0407d1e2c6";

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
    let bech32_string = address.to_bech32("iota");
    assert_eq!(
        bech32_string,
        "iota1q9f0mlq8yxpx2nck8a0slxnzr4ef2ek8f5gqxlzd0wasgp73utryj0w6qwt"
    );
}

#[test]
fn generate_bech32_testnet_string() {
    let mut bytes = [0; 32];
    let vec = hex::decode(ED25519_ADDRESS).unwrap();
    bytes.copy_from_slice(&vec);
    let address = Ed25519Address::new(bytes);
    let bech32_string = address.to_bech32("atoi");
    assert_eq!(
        bech32_string,
        "atoi1q9f0mlq8yxpx2nck8a0slxnzr4ef2ek8f5gqxlzd0wasgp73utryjgqtp5x"
    );
}

#[test]
fn bech32_string_to_address() {
    let mut bytes = [0; 32];
    let vec = hex::decode(ED25519_ADDRESS).unwrap();
    bytes.copy_from_slice(&vec);
    let address = Address::try_from_bech32(&Ed25519Address::new(bytes).to_bech32("iota")).unwrap();
    if let Address::Ed25519(ed) = address {
        assert_eq!(
            ed.to_string(),
            "52fdfc072182654f163f5f0f9a621d729566c74d10037c4d7bbb0407d1e2c649"
        );
    } else {
        panic!("Expecting an Ed25519 address");
    }
    let address = Address::try_from_bech32(&Ed25519Address::new(bytes).to_bech32("atoi")).unwrap();
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
fn from_str_invalid_len() {
    assert!(matches!(
        Ed25519Address::from_str(ED25519_ADDRESS_INVALID_LEN),
        Err(Error::InvalidHexadecimalLength(expected, actual))
            if expected == ED25519_ADDRESS_LENGTH * 2 && actual == ED25519_ADDRESS_LENGTH * 2 - 2
    ));
}

#[test]
fn from_to_str() {
    assert_eq!(
        ED25519_ADDRESS,
        Ed25519Address::from_str(ED25519_ADDRESS).unwrap().to_string()
    );
}
