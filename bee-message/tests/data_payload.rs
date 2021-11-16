// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_message::{
    error::ValidationError,
    payload::{data::DataPayload, MessagePayload},
    MessageUnpackError,
};
use bee_packable::{bounded::InvalidBoundedU32, error::UnpackError, prefix::TryIntoPrefixError, PackableExt};
use bee_test::rand::bytes::rand_bytes;

#[test]
fn kind() {
    assert_eq!(DataPayload::KIND, 1);
}

#[test]
fn version() {
    assert_eq!(DataPayload::VERSION, 0);
}

#[test]
fn new_valid() {
    let data = DataPayload::new(rand_bytes(255));

    assert!(data.is_ok());
}

#[test]
fn new_invalid_length() {
    let data_bytes = 65160;
    let data = DataPayload::new(rand_bytes(data_bytes));

    println!("Result: {:?}", data);

    assert!(matches!(
        data,
        Err(ValidationError::InvalidDataPayloadLength(TryIntoPrefixError::Invalid(
            InvalidBoundedU32(65160)
        ))),
    ));
}

#[test]
fn unpack_valid() {
    let mut bytes = vec![255, 0, 0, 0];

    bytes.extend(rand_bytes(255));

    let data = DataPayload::unpack_verified(bytes);

    assert!(data.is_ok());
}

#[test]
fn unpack_invalid_length() {
    let data_bytes = 65160;

    let mut bytes = vec![0x88, 0xfe, 0, 0];

    bytes.extend(rand_bytes(data_bytes));

    let data = DataPayload::unpack_verified(bytes);

    assert!(matches!(
        data,
        Err(UnpackError::Packable(MessageUnpackError::Validation(
            ValidationError::InvalidDataPayloadLength(TryIntoPrefixError::Invalid(InvalidBoundedU32(65160)))
        )))
    ));
}

#[test]
fn accessors_eq() {
    let bytes = rand_bytes(255);
    let data = DataPayload::new(bytes.clone()).unwrap();

    assert_eq!(data.data(), bytes);
}

#[test]
fn packed_len() {
    let data = DataPayload::new(rand_bytes(255)).unwrap();

    assert_eq!(data.packed_len(), 4 + 255);
}

#[test]
fn packable_round_trip() {
    let data_a = DataPayload::new(rand_bytes(255)).unwrap();
    let data_b = DataPayload::unpack_verified(data_a.pack_to_vec()).unwrap();

    assert_eq!(data_a, data_b);
}

#[test]
fn serde_round_trip() {
    let data_payload_1 = DataPayload::new(rand_bytes(255)).unwrap();
    let json = serde_json::to_string(&data_payload_1).unwrap();
    let data_payload_2 = serde_json::from_str::<DataPayload>(&json).unwrap();

    assert_eq!(data_payload_1, data_payload_2);
}
