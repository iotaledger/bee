// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_message::prelude::*;

const ED25519_ADDRESS: &str = "52fdfc072182654f163f5f0f9a621d729566c74d10037c4d7bbb0407d1e2c649";

#[test]
fn generate_bech32_string() {
    let mut bytes = [0; 32];
    let vec = hex::decode(ED25519_ADDRESS).unwrap();
    bytes.copy_from_slice(&vec);
    let address = Address::from(Ed25519Address::new(bytes));
    let bech32_string = address.to_bech32("iota");
    assert_eq!(
        bech32_string,
        "iota1qpf0mlq8yxpx2nck8a0slxnzr4ef2ek8f5gqxlzd0wasgp73utryj430ldu"
    );
}

#[test]
fn generate_bech32_testnet_string() {
    let mut bytes = [0; 32];
    let vec = hex::decode(ED25519_ADDRESS).unwrap();
    bytes.copy_from_slice(&vec);
    let address = Address::from(Ed25519Address::new(bytes));
    let bech32_string = address.to_bech32("atoi");
    assert_eq!(
        bech32_string,
        "atoi1qpf0mlq8yxpx2nck8a0slxnzr4ef2ek8f5gqxlzd0wasgp73utryjjl77h3"
    );
}

#[test]
fn bech32_string_to_address() {
    let mut bytes = [0; 32];
    let vec = hex::decode(ED25519_ADDRESS).unwrap();
    bytes.copy_from_slice(&vec);
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
