// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_message::{
    signature::{Ed25519Signature, Signature},
    unlock::SignatureUnlock,
};
use bee_packable::{Packable, PackableExt};
use bee_test::rand::bytes::rand_bytes_array;

use core::ops::Deref;

#[test]
fn kind() {
    assert_eq!(SignatureUnlock::KIND, 0);
}

#[test]
fn new_getter() {
    let signature = Signature::from(Ed25519Signature::new(rand_bytes_array(), rand_bytes_array()));
    let signature_unlock = SignatureUnlock::new(signature.clone());

    assert_eq!(signature_unlock.signature(), &signature);
}

#[test]
fn new_deref() {
    let signature = Signature::from(Ed25519Signature::new(rand_bytes_array(), rand_bytes_array()));
    let signature_unlock = SignatureUnlock::new(signature.clone());

    assert_eq!(signature_unlock.deref(), &signature);
}

#[test]
fn from() {
    let signature = Signature::from(Ed25519Signature::new(rand_bytes_array(), rand_bytes_array()));
    let signature_unlock = SignatureUnlock::from(signature.clone());

    assert_eq!(signature_unlock.signature(), &signature);
}

#[test]
fn packed_len() {
    let signature_unlock = SignatureUnlock::from(Signature::from(Ed25519Signature::new(
        rand_bytes_array(),
        rand_bytes_array(),
    )));

    assert_eq!(signature_unlock.packed_len(), 1 + 32 + 64);
    assert_eq!(signature_unlock.pack_to_vec().len(), 1 + 32 + 64);
}

#[test]
fn packable_round_trip() {
    let signature_unlock_1 = SignatureUnlock::from(Signature::from(Ed25519Signature::new(
        rand_bytes_array(),
        rand_bytes_array(),
    )));
    let signature_unlock_2 = SignatureUnlock::unpack_from_slice(signature_unlock_1.pack_to_vec()).unwrap();

    assert_eq!(signature_unlock_1, signature_unlock_2);
}

#[test]
fn serde_round_trip() {
    let signature_unlock_1 = SignatureUnlock::from(Signature::from(Ed25519Signature::new(
        rand_bytes_array(),
        rand_bytes_array(),
    )));
    let json = serde_json::to_string(&signature_unlock_1).unwrap();
    let signature_unlock_2 = serde_json::from_str::<SignatureUnlock>(&json).unwrap();

    assert_eq!(signature_unlock_1, signature_unlock_2);
}
