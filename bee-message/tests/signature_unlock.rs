// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_message::{
    error::MessageUnpackError,
    signature::{Ed25519Signature, Signature, SignatureUnpackError},
    unlock::SignatureUnlock,
};
use bee_packable::{Packable, UnpackError};
use bee_test::rand::bytes::rand_bytes_array;

#[test]
fn kind() {
    assert_eq!(SignatureUnlock::KIND, 0);
}

#[test]
fn new_valid() {
    let public_key_bytes = rand_bytes_array();
    let signature_bytes: [u8; 64] = rand_bytes_array();
    let signature_unlock = SignatureUnlock::from(Signature::from(Ed25519Signature::new(
        public_key_bytes,
        signature_bytes,
    )));

    assert!(matches!(signature_unlock.signature(), Signature::Ed25519(signature)
        if signature.public_key() == &public_key_bytes
        && signature.signature() == signature_bytes.as_ref()
    ));
}

#[test]
fn from_valid() {
    let public_key_bytes = rand_bytes_array();
    let signature_bytes: [u8; 64] = rand_bytes_array();
    let signature_unlock = SignatureUnlock::from(Signature::from(Ed25519Signature::new(
        public_key_bytes,
        signature_bytes,
    )));

    assert!(matches!(signature_unlock.signature(), Signature::Ed25519(signature)
        if signature.public_key() == &public_key_bytes
        && signature.signature() == signature_bytes.as_ref()
    ));
}

#[test]
fn packed_len() {
    let signature_unlock = SignatureUnlock::from(Signature::from(Ed25519Signature::new(
        rand_bytes_array(),
        rand_bytes_array(),
    )));

    assert_eq!(signature_unlock.packed_len(), 1 + 32 + 64);
    assert_eq!(signature_unlock.pack_to_vec().unwrap().len(), 1 + 32 + 64);
}

#[test]
fn packable_round_trip() {
    let signature_unlock_1 = SignatureUnlock::from(Signature::from(Ed25519Signature::new(
        rand_bytes_array(),
        rand_bytes_array(),
    )));
    let signature_unlock_bytes = signature_unlock_1.pack_to_vec().unwrap();
    let signature_unlock_2 = SignatureUnlock::unpack_from_slice(signature_unlock_bytes.clone()).unwrap();

    assert_eq!(signature_unlock_bytes[0], 0);
    assert_eq!(signature_unlock_1, signature_unlock_2);
}
