// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_block::{
    address::{Address, Ed25519Address},
    Error,
};
use packable::PackableExt;

const ED25519_ADDRESS: &str = "0x52fdfc072182654f163f5f0f9a621d729566c74d10037c4d7bbb0407d1e2c649";
const ED25519_ADDRESS_BAD: &str = "0x52fdfc072182654f163f5f0f9a621d729566c74d10037c4d7bbb0407d1e2c64x";

// The kind of an `Address` is the kind of the underlying address.
#[test]
fn kind() {
    let bytes: [u8; 32] = prefix_hex::decode(ED25519_ADDRESS).unwrap();
    let ed25519_address = Address::from(Ed25519Address::new(bytes));

    assert_eq!(ed25519_address.kind(), 0);
}

#[test]
fn generate_bech32_string() {
    let bytes: [u8; 32] = prefix_hex::decode(ED25519_ADDRESS).unwrap();
    let address = Address::from(Ed25519Address::new(bytes));
    let bech32_string = address.to_bech32("iota");

    assert_eq!(
        bech32_string,
        "iota1qpf0mlq8yxpx2nck8a0slxnzr4ef2ek8f5gqxlzd0wasgp73utryj430ldu"
    );
}

#[test]
fn generate_bech32_testnet_string() {
    let bytes: [u8; 32] = prefix_hex::decode(ED25519_ADDRESS).unwrap();
    let address = Address::from(Ed25519Address::new(bytes));
    let bech32_string = address.to_bech32("atoi");

    assert_eq!(
        bech32_string,
        "atoi1qpf0mlq8yxpx2nck8a0slxnzr4ef2ek8f5gqxlzd0wasgp73utryjjl77h3"
    );
}

#[test]
fn bech32_string_to_address() {
    let bytes: [u8; 32] = prefix_hex::decode(ED25519_ADDRESS).unwrap();

    let (hrp, address) =
        Address::try_from_bech32(&Address::from(Ed25519Address::new(bytes)).to_bech32("iota")).unwrap();
    let ed = match address {
        Address::Ed25519(ed) => ed,
        _ => unreachable!(),
    };

    assert_eq!(hrp, "iota");
    assert_eq!(ed.to_string(), ED25519_ADDRESS);

    let (hrp, address) =
        Address::try_from_bech32(&Address::from(Ed25519Address::new(bytes)).to_bech32("atoi")).unwrap();
    let ed = match address {
        Address::Ed25519(ed) => ed,
        _ => unreachable!(),
    };

    assert_eq!(hrp, "atoi");
    assert_eq!(ed.to_string(), ED25519_ADDRESS);
}

#[test]
fn invalid_bech32_string_to_address() {
    let address = Address::try_from_bech32(ED25519_ADDRESS_BAD);
    assert!(matches!(address, Err(Error::InvalidAddress)));
}

#[test]
fn pack_unpack_valid_ed25519() {
    let bytes: [u8; 32] = prefix_hex::decode(ED25519_ADDRESS).unwrap();
    let address = Address::from(Ed25519Address::new(bytes));
    let address_packed = address.pack_to_vec();

    assert_eq!(address_packed.len(), address.packed_len());
    assert_eq!(
        address,
        PackableExt::unpack_verified(&mut address_packed.as_slice()).unwrap()
    );
}
