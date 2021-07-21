// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_message::{
    address::{Address, Ed25519Address},
    util::hex_decode,
};
use bee_packable::Packable;

const ED25519_ADDRESS: &str = "52fdfc072182654f163f5f0f9a621d729566c74d10037c4d7bbb0407d1e2c649";

#[test]
fn kind() {
    let ed25519_address = Address::from(Ed25519Address::new(hex_decode(ED25519_ADDRESS).unwrap()));

    assert_eq!(ed25519_address.kind(), 0);
}

#[test]
fn generate_bech32_string() {
    let address = Address::from(Ed25519Address::new(hex_decode(ED25519_ADDRESS).unwrap()));
    let bech32_string = address.to_bech32("iota");

    assert_eq!(
        bech32_string,
        "iota1qpf0mlq8yxpx2nck8a0slxnzr4ef2ek8f5gqxlzd0wasgp73utryj430ldu"
    );
}

#[test]
fn generate_bech32_testnet_string() {
    let address = Address::from(Ed25519Address::new(hex_decode(ED25519_ADDRESS).unwrap()));
    let bech32_string = address.to_bech32("atoi");

    assert_eq!(
        bech32_string,
        "atoi1qpf0mlq8yxpx2nck8a0slxnzr4ef2ek8f5gqxlzd0wasgp73utryjjl77h3"
    );
}

#[test]
fn bech32_string_to_address() {
    let address = Address::from(Ed25519Address::new(hex_decode(ED25519_ADDRESS).unwrap()));

    let address = Address::try_from_bech32(&address.to_bech32("iota")).unwrap();
    assert_eq!(address.to_string(), ED25519_ADDRESS);

    let address = Address::try_from_bech32(&address.to_bech32("atoi")).unwrap();
    assert_eq!(address.to_string(), ED25519_ADDRESS);
}

#[test]
fn packed_len_ed25519() {
    let address = Address::from(Ed25519Address::new(hex_decode(ED25519_ADDRESS).unwrap()));

    assert_eq!(address.packed_len(), 32 + 1);
}

#[test]
fn packable_round_trip_ed25519() {
    let address = Address::from(Ed25519Address::new(hex_decode(ED25519_ADDRESS).unwrap()));
    let address_packed = address.pack_to_vec().unwrap();

    assert_eq!(address_packed.len(), address.packed_len());
    assert_eq!(address, Address::unpack_from_slice(address_packed).unwrap());
}
