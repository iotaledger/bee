// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_message::{prelude::*, DataUnpackError};
use bee_packable::{Packable, UnpackError};

use bee_test::rand::bytes::rand_bytes;

#[test]
fn kind() {
    assert_eq!(DataPayload::KIND, 1);
}

#[test]
fn new_valid() {
    let data = DataPayload::new(0, rand_bytes(255));

    assert!(data.is_ok());
}

#[test]
fn new_invalid_length() {
    let data_bytes = 65160;
    let data = DataPayload::new(0, rand_bytes(data_bytes));

    println!("Result: {:?}", data);

    assert!(matches!(
        data,
        Err(ValidationError::InvalidPayloadLength(65160)),
    ));
}

#[test]
fn unpack_valid() {
    let mut bytes = vec![0u8, 255, 0, 0, 0];

    bytes.extend(rand_bytes(255));

    let data = DataPayload::unpack_from_slice(bytes);

    assert!(data.is_ok());
}

#[test]
fn unpack_invalid_length() {
    let data_bytes = 65160;

    let mut bytes = vec![0u8, 0x88, 0xfe, 0, 0];

    bytes.extend(rand_bytes(data_bytes));

    let data = DataPayload::unpack_from_slice(bytes);

    assert!(matches!(
        data,
        Err(UnpackError::Packable(MessageUnpackError::Data(DataUnpackError::InvalidPrefixLength(65160))))
    ));
}

#[test]
fn packed_len() {
    let data = DataPayload::new(0, rand_bytes(255)).unwrap();

    assert_eq!(data.packed_len(), 1 + 4 + 255);
}

#[test]
fn round_trip() {
    let data_a = DataPayload::new(0, rand_bytes(255)).unwrap();
    let data_b = DataPayload::unpack_from_slice(data_a.pack_to_vec().unwrap()).unwrap();

    assert_eq!(data_a, data_b);
}
