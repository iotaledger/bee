// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_message::{
    error::MessageUnpackError,
    signature::{BlsSignature, Ed25519Signature, Signature, SignatureUnpackError},
};
use bee_packable::{Packable, UnpackError};
use bee_test::rand::bytes::rand_bytes_array;

#[test]
fn from_ed25519() {
    let ed25519_signature = Ed25519Signature::new(rand_bytes_array(), rand_bytes_array());
    let signature = Signature::from(ed25519_signature.clone());

    assert_eq!(signature.kind(), 0);
    assert!(matches!(signature, Signature::Ed25519(signature) if {signature == ed25519_signature}));
}

#[test]
fn from_bls() {
    let bls_signature = BlsSignature::new(rand_bytes_array());
    let signature = Signature::from(bls_signature.clone());

    assert_eq!(signature.kind(), 1);
    assert!(matches!(signature, Signature::Bls(signature) if {signature == bls_signature}));
}

#[test]
fn packed_len() {
    let signature = Signature::from(Ed25519Signature::new(rand_bytes_array(), rand_bytes_array()));

    assert_eq!(signature.packed_len(), 1 + 32 + 64);
    assert_eq!(signature.pack_to_vec().unwrap().len(), 1 + 32 + 64);
}

#[test]
fn packable_round_trip() {
    let signature_1 = Signature::from(Ed25519Signature::new(rand_bytes_array(), rand_bytes_array()));
    let signature_2 = Signature::unpack_from_slice(signature_1.pack_to_vec().unwrap()).unwrap();

    assert_eq!(signature_1, signature_2);
}

#[test]
fn unpack_invalid_kind() {
    assert!(matches!(
        Signature::unpack_from_slice(vec![
            4, 111, 225, 221, 28, 247, 253, 234, 110, 187, 52, 129, 153, 130, 84, 26, 7, 226, 27, 212, 145, 96, 151,
            196, 124, 135, 176, 31, 48, 0, 213, 200, 82, 227, 169, 21, 179, 253, 115, 184, 209, 107, 138, 0, 62, 252,
            20, 255, 24, 193, 203, 255, 137, 142, 158, 25, 171, 86, 195, 20, 70, 56, 136, 204, 2, 219, 254, 218, 2,
            234, 91, 56, 50, 122, 112, 200, 110, 181, 15, 166, 100, 53, 115, 124, 220, 90, 50, 188, 45, 124, 67, 30,
            124, 38, 34, 89, 92
        ]),
        Err(UnpackError::Packable(MessageUnpackError::Signature(
            SignatureUnpackError::InvalidKind(4)
        ))),
    ));
}

#[test]
fn serde_round_trip() {
    let signature_1 = Signature::from(Ed25519Signature::new(rand_bytes_array(), rand_bytes_array()));
    let json = serde_json::to_string(&signature_1).unwrap();
    let signature_2 = serde_json::from_str::<Signature>(&json).unwrap();

    assert_eq!(signature_1, signature_2);
}
