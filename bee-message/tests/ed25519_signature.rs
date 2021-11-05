// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_message::{signature::Ed25519Signature, util::hex_decode};
use bee_packable::{Packable, PackableExt};

const ED25519_PUBLIC_KEY: &str = "1da5ddd11ba3f961acab68fafee3177d039875eaa94ac5fdbff8b53f0c50bfb9";
const ED25519_SIGNATURE: &str = "c6a40edf9a089f42c18f4ebccb35fe4b578d93b879e99b87f63573324a710d3456b03fb6d1fcc027e6401cbd9581f790ee3ed7a3f68e9c225fcb9f1cd7b7110d";

#[test]
fn kind() {
    assert_eq!(Ed25519Signature::KIND, 0);
}

#[test]
fn public_key_length() {
    assert_eq!(Ed25519Signature::PUBLIC_KEY_LENGTH, 32);
}

#[test]
fn signature_length() {
    assert_eq!(Ed25519Signature::SIGNATURE_LENGTH, 64);
}

#[test]
fn new() {
    let signature = Ed25519Signature::new([21; 32], [42; 64]);

    assert_eq!(signature.public_key(), &[21; 32]);
    assert_eq!(signature.signature(), &[42; 64]);
}

#[test]
fn packed_len() {
    let signature = Ed25519Signature::new(
        hex_decode(ED25519_PUBLIC_KEY).unwrap(),
        hex_decode(ED25519_SIGNATURE).unwrap(),
    );

    assert_eq!(signature.packed_len(), 32 + 64);
    assert_eq!(signature.pack_to_vec().len(), 32 + 64);
}

#[test]
fn packable_round_trip() {
    let signature_1 = Ed25519Signature::new(
        hex_decode(ED25519_PUBLIC_KEY).unwrap(),
        hex_decode(ED25519_SIGNATURE).unwrap(),
    );
    let signature_2 = Ed25519Signature::unpack_from_slice(signature_1.pack_to_vec()).unwrap();

    assert_eq!(signature_1, signature_2);
}

#[test]
fn serde_round_trip() {
    let ed25519_signature_1 = Ed25519Signature::new(
        hex_decode(ED25519_PUBLIC_KEY).unwrap(),
        hex_decode(ED25519_SIGNATURE).unwrap(),
    );
    let json = serde_json::to_string(&ed25519_signature_1).unwrap();
    let ed25519_signature_2 = serde_json::from_str::<Ed25519Signature>(&json).unwrap();

    assert_eq!(ed25519_signature_1, ed25519_signature_2);
}
