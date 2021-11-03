// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_message::{
    error::{MessageUnpackError, ValidationError},
    payload::{indexation::IndexationPayload, MessagePayload},
    MESSAGE_LENGTH_RANGE,
};
use bee_packable::{packable::VecPrefixLengthError, InvalidBoundedU32, Packable, UnpackError};
use bee_test::rand::bytes::rand_bytes;

use core::convert::TryFrom;

#[test]
fn kind() {
    assert_eq!(IndexationPayload::KIND, 8);
}

#[test]
fn version() {
    assert_eq!(IndexationPayload::VERSION, 0);
}

#[test]
fn new_valid() {
    let index = rand_bytes(64);
    let data = [0x42, 0xff, 0x84, 0xa2, 0x42, 0xff, 0x84, 0xa2].to_vec();
    let indexation = IndexationPayload::new(index.clone(), data.clone()).unwrap();

    assert_eq!(indexation.index(), &index);
    assert_eq!(indexation.padded_index().as_ref(), index.as_slice());
    assert_eq!(indexation.data(), &data);
}

#[test]
fn new_valid_empty_data() {
    let index = rand_bytes(64);
    let data = vec![];
    let indexation = IndexationPayload::new(index.clone(), data.clone()).unwrap();

    assert_eq!(indexation.index(), &index);
    assert_eq!(indexation.padded_index().as_ref(), index.as_slice());
    assert_eq!(indexation.data(), &data);
}

#[test]
fn new_valid_padded() {
    let index = rand_bytes(32);
    let mut padded_index = index.clone();
    padded_index.append(&mut vec![0u8; 32]);
    let data = [];
    let indexation = IndexationPayload::new(index.clone(), data.to_vec()).unwrap();

    assert_eq!(indexation.index(), &index);
    assert_eq!(indexation.padded_index().as_ref(), padded_index.as_slice());
    assert_eq!(indexation.data(), &data);
}

#[test]
fn new_invalid_index_length_less_than_min() {
    assert!(matches!(
        IndexationPayload::new([].to_vec(), [0x42, 0xff, 0x84, 0xa2, 0x42, 0xff, 0x84, 0xa2].to_vec()),
        Err(ValidationError::InvalidIndexationIndexLength(
            VecPrefixLengthError::Invalid(InvalidBoundedU32(0))
        ))
    ));
}

#[test]
fn new_invalid_index_length_more_than_max() {
    assert!(matches!(
        IndexationPayload::new(
            rand_bytes(65),
            [0x42, 0xff, 0x84, 0xa2, 0x42, 0xff, 0x84, 0xa2].to_vec()
        ),
        Err(ValidationError::InvalidIndexationIndexLength(
            VecPrefixLengthError::Invalid(InvalidBoundedU32(65))
        ))
    ));
}

#[test]
fn new_invalid_data_length_more_than_max() {
    assert!(matches!(
        IndexationPayload::new(rand_bytes(32), [0u8; 65540].to_vec()),
        Err(ValidationError::InvalidIndexationDataLength(
            VecPrefixLengthError::Invalid(InvalidBoundedU32(65540))
        )),
    ));
}

#[test]
fn unpack_invalid_index_length_less_than_min() {
    assert!(matches!(
        IndexationPayload::unpack_from_slice(vec![0x00, 0x00, 0x00, 0x00, 0x00, 0x00]),
        Err(UnpackError::Packable(MessageUnpackError::Validation(
            ValidationError::InvalidIndexationIndexLength(VecPrefixLengthError::Invalid(InvalidBoundedU32(0)))
        ))),
    ));
}

#[test]
fn unpack_invalid_index_length_more_than_max() {
    assert!(matches!(
        IndexationPayload::unpack_from_slice(vec![
            0x41, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ]),
        Err(UnpackError::Packable(MessageUnpackError::Validation(
            ValidationError::InvalidIndexationIndexLength(VecPrefixLengthError::Invalid(InvalidBoundedU32(65)))
        ))),
    ));
}

#[test]
fn unpack_invalid_data_length_more_than_max() {
    let data_len = MESSAGE_LENGTH_RANGE.end() + 5;

    let mut bytes = vec![0x0A, 0x00, 0x00, 0x00];
    bytes.extend(rand_bytes(10));
    bytes.extend(vec![0x05, 0x00, 0x01, 0x00]);
    bytes.extend(vec![0; data_len]);

    assert!(matches!(
        IndexationPayload::unpack_from_slice(bytes),
        Err(UnpackError::Packable(MessageUnpackError::Validation(
            ValidationError::InvalidIndexationDataLength(VecPrefixLengthError::Invalid(InvalidBoundedU32(n)))
        )))
            if n == u32::try_from(data_len).unwrap()
    ))
}

#[test]
fn packed_len() {
    let indexation = IndexationPayload::new(
        rand_bytes(10),
        [0x42, 0xff, 0x84, 0xa2, 0x42, 0xff, 0x84, 0xa2].to_vec(),
    )
    .unwrap();

    assert_eq!(indexation.packed_len(), 10 + 4 + 4 + 8);
    assert_eq!(indexation.pack_to_vec().len(), 10 + 4 + 4 + 8);
}

#[test]
fn packable_round_trip() {
    let index = rand_bytes(32);
    let mut padded_index = index.clone();
    padded_index.append(&mut vec![0u8; 32]);

    let indexation_1 =
        IndexationPayload::new(index, [0x42, 0xff, 0x84, 0xa2, 0x42, 0xff, 0x84, 0xa2].to_vec()).unwrap();

    let indexation_2 = IndexationPayload::unpack_from_slice(indexation_1.pack_to_vec()).unwrap();

    assert_eq!(indexation_1, indexation_2);
}

#[test]
fn serde_round_trip() {
    let indexation_payload_1 = IndexationPayload::new(
        rand_bytes(10),
        [0x42, 0xff, 0x84, 0xa2, 0x42, 0xff, 0x84, 0xa2].to_vec(),
    )
    .unwrap();
    let json = serde_json::to_string(&indexation_payload_1).unwrap();
    let indexation_payload_2 = serde_json::from_str::<IndexationPayload>(&json).unwrap();

    assert_eq!(indexation_payload_1, indexation_payload_2);
}
