// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_message::{
    error::MessageUnpackError,
    signature::{Ed25519Signature, SignatureUnlock, SignatureUnlockUnpackError},
};
use bee_packable::{Packable, UnpackError};
use bee_test::rand::bytes::rand_bytes_array;

#[test]
fn unlock_kind() {
    assert_eq!(SignatureUnlock::KIND, 0);
}

#[test]
fn signature_kind() {
    assert_eq!(
        SignatureUnlock::from(Ed25519Signature::new(rand_bytes_array(), rand_bytes_array(),)).kind(),
        0,
    );
}

#[test]
fn from_ed25519() {
    let public_key_bytes = rand_bytes_array();
    let signature_bytes: [u8; 64] = rand_bytes_array();
    let signature = SignatureUnlock::from(Ed25519Signature::new(public_key_bytes, signature_bytes));

    assert!(matches!(signature, SignatureUnlock::Ed25519(signature)
        if signature.public_key() == &public_key_bytes
        && signature.signature() == signature_bytes.as_ref()
    ));
}

#[test]
fn unpack_invalid_kind() {
    assert!(matches!(
        SignatureUnlock::unpack_from_slice(vec![
            1, 111, 225, 221, 28, 247, 253, 234, 110, 187, 52, 129, 153, 130, 84, 26, 7, 226, 27, 212, 145, 96, 151,
            196, 124, 135, 176, 31, 48, 0, 213, 200, 82, 227, 169, 21, 179, 253, 115, 184, 209, 107, 138, 0, 62, 252,
            20, 255, 24, 193, 203, 255, 137, 142, 158, 25, 171, 86, 195, 20, 70, 56, 136, 204, 2, 219, 254, 218, 2,
            234, 91, 56, 50, 122, 112, 200, 110, 181, 15, 166, 100, 53, 115, 124, 220, 90, 50, 188, 45, 124, 67, 30,
            124, 38, 34, 89, 92
        ])
        .err()
        .unwrap(),
        UnpackError::Packable(MessageUnpackError::SignatureUnlock(
            SignatureUnlockUnpackError::InvalidSignatureUnlockKind(1)
        )),
    ));
}

#[test]
fn packed_len() {
    let signature = SignatureUnlock::from(Ed25519Signature::new(rand_bytes_array(), rand_bytes_array()));

    assert_eq!(signature.packed_len(), 1 + 32 + 64);
    assert_eq!(signature.pack_to_vec().unwrap().len(), 1 + 32 + 64);
}

#[test]
fn packable_round_trip_ed25519() {
    let signature_1 = SignatureUnlock::from(Ed25519Signature::new(rand_bytes_array(), rand_bytes_array()));
    let signature_bytes = signature_1.pack_to_vec().unwrap();
    let signature_2 = SignatureUnlock::unpack_from_slice(signature_bytes.clone()).unwrap();

    assert_eq!(signature_bytes[0], 0);
    assert_eq!(signature_1, signature_2);
}
