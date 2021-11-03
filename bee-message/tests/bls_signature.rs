// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_message::{signature::BlsSignature, util::hex_decode};
use bee_packable::Packable;

use core::ops::Deref;

const BLS_SIGNATURE: &str = "c6a40edf9a089f42c18f4ebccb35fe4b578d93b879e99b87f63573324a710d3456b03fb6d1fcc027e6401cbd9581f790ee3ed7a3f68e9c225fcb9f1cd7b7110d";

#[test]
fn kind() {
    assert_eq!(BlsSignature::KIND, 1);
}

#[test]
fn length() {
    assert_eq!(BlsSignature::LENGTH, 64);
}

#[test]
fn new_as_ref() {
    assert_eq!(BlsSignature::new([42; 64]).as_ref(), &[42; 64]);
}

#[test]
fn new_deref() {
    assert_eq!(BlsSignature::new([42; 64]).deref(), &[42; 64]);
}

#[test]
fn from_as_ref() {
    let signature = BlsSignature::from(hex_decode("c6a40edf9a089f42c18f4ebccb35fe4b578d93b879e99b87f63573324a710d3456b03fb6d1fcc027e6401cbd9581f790ee3ed7a3f68e9c225fcb9f1cd7b7110d").unwrap());

    assert_eq!(
        signature.as_ref(),
        &[
            0xc6, 0xa4, 0x0e, 0xdf, 0x9a, 0x08, 0x9f, 0x42, 0xc1, 0x8f, 0x4e, 0xbc, 0xcb, 0x35, 0xfe, 0x4b, 0x57, 0x8d,
            0x93, 0xb8, 0x79, 0xe9, 0x9b, 0x87, 0xf6, 0x35, 0x73, 0x32, 0x4a, 0x71, 0x0d, 0x34, 0x56, 0xb0, 0x3f, 0xb6,
            0xd1, 0xfc, 0xc0, 0x27, 0xe6, 0x40, 0x1c, 0xbd, 0x95, 0x81, 0xf7, 0x90, 0xee, 0x3e, 0xd7, 0xa3, 0xf6, 0x8e,
            0x9c, 0x22, 0x5f, 0xcb, 0x9f, 0x1c, 0xd7, 0xb7, 0x11, 0x0d
        ]
    );
}

#[test]
fn packed_len() {
    let signature = BlsSignature::new(hex_decode(BLS_SIGNATURE).unwrap());

    assert_eq!(signature.packed_len(), 64);
    assert_eq!(signature.pack_to_vec().len(), 64);
}

#[test]
fn packable_round_trip() {
    let signature_1 = BlsSignature::new(hex_decode(BLS_SIGNATURE).unwrap());
    let signature_2 = BlsSignature::unpack_from_slice(signature_1.pack_to_vec()).unwrap();

    assert_eq!(signature_1, signature_2);
}

#[test]
fn serde_round_trip() {
    let bls_signature_1 = BlsSignature::new(hex_decode(BLS_SIGNATURE).unwrap());
    let json = serde_json::to_string(&bls_signature_1).unwrap();
    let bls_signature_2 = serde_json::from_str::<BlsSignature>(&json).unwrap();

    assert_eq!(bls_signature_1, bls_signature_2);
}
