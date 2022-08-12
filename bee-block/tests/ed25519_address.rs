// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use core::str::FromStr;

use bee_block::address::{Address, Ed25519Address};
use packable::PackableExt;

const ED25519_ADDRESS: &str = "0x52fdfc072182654f163f5f0f9a621d729566c74d10037c4d7bbb0407d1e2c649";

#[test]
fn kind() {
    assert_eq!(Ed25519Address::KIND, 0);
}

#[test]
fn debug_impl() {
    assert_eq!(
        format!("{:?}", Ed25519Address::from_str(ED25519_ADDRESS).unwrap()),
        "Ed25519Address(0x52fdfc072182654f163f5f0f9a621d729566c74d10037c4d7bbb0407d1e2c649)"
    );
}

#[test]
fn generate_address() {
    let bytes = [1; 32];

    match Address::from(Ed25519Address::new(bytes)) {
        Address::Ed25519(a) => assert_eq!(a.as_ref(), bytes),
        _ => unreachable!(),
    }
}

#[test]
fn from_str_valid() {
    Ed25519Address::from_str(ED25519_ADDRESS).unwrap();
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
    let address1 = Address::Ed25519(Ed25519Address::from_str(ED25519_ADDRESS).unwrap());
    let (hrp, address2) = Address::try_from_bech32(&address1.to_bech32("atoi")).unwrap();

    assert_eq!(hrp, "atoi");
    assert_eq!(address1, address2);
}

#[test]
fn packed_len() {
    let address = Ed25519Address::from_str(ED25519_ADDRESS).unwrap();

    assert_eq!(address.packed_len(), 32);
    assert_eq!(address.pack_to_vec().len(), 32);
}

#[test]
fn pack_unpack_valid() {
    let address = Ed25519Address::from_str(ED25519_ADDRESS).unwrap();
    let packed_address = address.pack_to_vec();

    assert_eq!(
        address,
        PackableExt::unpack_verified(&mut packed_address.as_slice(), &()).unwrap()
    );
}
