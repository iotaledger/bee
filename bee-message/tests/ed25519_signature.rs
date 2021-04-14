// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_common::packable::Packable;
use bee_message::prelude::*;

use core::convert::TryInto;

const ED25519_PULIC_KEY: &str = "1DA5DDD11BA3F961ACAB68FAFEE3177D039875EAA94AC5FDBFF8B53F0C50BFB9";
const ED25519_SIGNATURE: &str = "c6a40edf9a089f42c18f4ebccb35fe4b578d93b879e99b87f63573324a710d3456b03fb6d1fcc027e6401cbd9581f790ee3ed7a3f68e9c225fcb9f1cd7b7110d";

#[test]
fn kind() {
    assert_eq!(Ed25519Signature::KIND, 0);
}

#[test]
fn packed_len() {
    let pub_key_bytes: [u8; 32] = hex::decode(ED25519_PULIC_KEY).unwrap().try_into().unwrap();
    let sig_bytes: [u8; 64] = hex::decode(ED25519_SIGNATURE).unwrap().try_into().unwrap();
    let sig = Ed25519Signature::new(pub_key_bytes, Box::new(sig_bytes));

    assert_eq!(sig.packed_len(), 32 + 64);
    assert_eq!(sig.pack_new().len(), 32 + 64);
}

#[test]
fn pack_unpack_valid() {
    let pub_key_bytes: [u8; 32] = hex::decode(ED25519_PULIC_KEY).unwrap().try_into().unwrap();
    let sig_bytes: [u8; 64] = hex::decode(ED25519_SIGNATURE).unwrap().try_into().unwrap();
    let sig = Ed25519Signature::new(pub_key_bytes, Box::new(sig_bytes));
    let sig_packed = sig.pack_new();

    assert_eq!(sig, Packable::unpack(&mut sig_packed.as_slice()).unwrap());
}
