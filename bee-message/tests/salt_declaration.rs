// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_message::prelude::*;
use bee_packable::Packable;

use bee_test::rand::{
    bytes::{rand_bytes, rand_bytes_array},
    number::rand_number,
};

#[test]
fn kind() {
    assert_eq!(SaltDeclarationPayload::KIND, 7);
}

#[test]
fn new_valid() {
    let salt_declaration = SaltDeclarationPayload::builder()
        .with_version(0)
        .with_node_id(32)
        .with_salt(Salt::new(rand_bytes(64), rand_number()).unwrap())
        .with_timestamp(rand_number())
        .with_signature(rand_bytes_array())
        .finish();

    assert!(salt_declaration.is_ok());
}

#[test]
fn unpack_valid() {
    let mut bytes = vec![0u8, 32, 0, 0, 0, 64, 0, 0, 0];

    bytes.extend(rand_bytes_array::<64>());
    bytes.extend(vec![0, 0, 0, 0, 0, 0, 0, 0]);
    bytes.extend(vec![0, 0, 0, 0, 0, 0, 0, 0]);
    bytes.extend(rand_bytes_array::<32>());

    let salt_declaration = SaltDeclarationPayload::unpack_from_slice(bytes);

    assert!(salt_declaration.is_ok());
}

#[test]
fn packed_len() {
    let salt_declaration = SaltDeclarationPayload::builder()
        .with_version(0)
        .with_node_id(32)
        .with_salt(Salt::new(rand_bytes(64), rand_number()).unwrap())
        .with_timestamp(rand_number())
        .with_signature(rand_bytes_array())
        .finish()
        .unwrap();

    assert_eq!(salt_declaration.packed_len(), 1 + 4 + 4 + 64 + 8 + 8 + 32);
}

#[test]
fn round_trip() {
    let salt_declaration_a = SaltDeclarationPayload::builder()
        .with_version(0)
        .with_node_id(32)
        .with_salt(Salt::new(rand_bytes(64), rand_number()).unwrap())
        .with_timestamp(rand_number())
        .with_signature(rand_bytes_array())
        .finish()
        .unwrap();

    let salt_declaration_b =
        SaltDeclarationPayload::unpack_from_slice(salt_declaration_a.pack_to_vec().unwrap()).unwrap();

    assert_eq!(salt_declaration_a, salt_declaration_b);
}
