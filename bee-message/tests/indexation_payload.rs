// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_message::{error::IndexationUnpackError, prelude::*};
use bee_packable::{Packable, UnpackError};
use bee_test::rand::bytes::rand_bytes;

use core::convert::TryInto;

const PADDED_INDEX: &str = "52fdfc072182654f163f5f0f9a621d729566c74d10037c4d7bbb0407d1e2c64952fdfc072182654f163f5f0f9a621d729566c74d10037c4d7bbb0407d1e2c649";

#[test]
fn kind() {
    assert_eq!(IndexationPayload::KIND, 8);
}

#[test]
fn display_impl() {
    assert_eq!(
        format!(
            "{}",
            PaddedIndex::new(hex::decode(PADDED_INDEX).unwrap().try_into().unwrap())
        ),
        PADDED_INDEX
    );
}

#[test]
fn debug_impl() {
    assert_eq!(
        format!(
            "{:?}",
            PaddedIndex::new(hex::decode(PADDED_INDEX).unwrap().try_into().unwrap())
        ),
        "PaddedIndex(".to_owned() + PADDED_INDEX + ")"
    );
}

#[test]
fn new_valid() {
    let index = rand_bytes(64);
    let data = [0x42, 0xff, 0x84, 0xa2, 0x42, 0xff, 0x84, 0xa2].to_vec();
    let indexation = IndexationPayload::new(0, index.clone(), data.clone()).unwrap();

    assert_eq!(indexation.index(), &index);
    assert_eq!(indexation.padded_index().as_ref(), index.as_slice());
    assert_eq!(indexation.data(), &data);
}

#[test]
fn new_valid_empty_data() {
    let index = rand_bytes(64);
    let data = vec![];
    let indexation = IndexationPayload::new(0, index.clone(), data.clone()).unwrap();

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
    let indexation = IndexationPayload::new(0, index.clone(), data.to_vec()).unwrap();

    assert_eq!(indexation.index(), &index);
    assert_eq!(indexation.padded_index().as_ref(), padded_index.as_slice());
    assert_eq!(indexation.data(), &data);
}

#[test]
fn new_invalid_index_length_less_than_min() {
    assert!(matches!(
        IndexationPayload::new(
            0,
            [].to_vec(),
            [0x42, 0xff, 0x84, 0xa2, 0x42, 0xff, 0x84, 0xa2].to_vec()
        ),
        Err(ValidationError::InvalidIndexationIndexLength(0))
    ));
}

#[test]
fn new_invalid_index_length_more_than_max() {
    assert!(matches!(
        IndexationPayload::new(
            0,
            rand_bytes(65),
            [0x42, 0xff, 0x84, 0xa2, 0x42, 0xff, 0x84, 0xa2].to_vec()
        ),
        Err(ValidationError::InvalidIndexationIndexLength(65))
    ));
}

#[test]
fn new_invalid_data_length_more_than_max() {
    assert!(matches!(
        IndexationPayload::new(0, rand_bytes(32), [0u8; 32800].to_vec()),
        Err(ValidationError::InvalidIndexationDataLength(32800)),
    ));
}

#[test]
fn unpack_invalid_index_length_less_than_min() {
    assert!(matches!(
        IndexationPayload::unpack_from_slice(vec![0x00, 0x00, 0x00, 0x00, 0x00, 0x00])
            .err()
            .unwrap(),
        UnpackError::Packable(MessageUnpackError::ValidationError(
            ValidationError::InvalidIndexationIndexLength(0)
        )),
    ));
}

#[test]
fn unpack_invalid_index_length_more_than_max() {
    assert!(matches!(
        IndexationPayload::unpack_from_slice(vec![
            0x00, 0x41, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ])
        .err()
        .unwrap(),
        UnpackError::Packable(MessageUnpackError::Indexation(
            IndexationUnpackError::InvalidPrefixLength(65)
        ))
    ),);
}

#[test]
fn packed_len() {
    let indexation = IndexationPayload::new(
        0,
        rand_bytes(10),
        [0x42, 0xff, 0x84, 0xa2, 0x42, 0xff, 0x84, 0xa2].to_vec(),
    )
    .unwrap();

    assert_eq!(indexation.packed_len(), 1 + 10 + 4 + 4 + 8);
    assert_eq!(indexation.pack_to_vec().unwrap().len(), 1 + 10 + 4 + 4 + 8);
}

#[test]
fn packable_round_trip() {
    let index = rand_bytes(32);
    let mut padded_index = index.clone();
    padded_index.append(&mut vec![0u8; 32]);

    let indexation_1 =
        IndexationPayload::new(0, index, [0x42, 0xff, 0x84, 0xa2, 0x42, 0xff, 0x84, 0xa2].to_vec()).unwrap();

    let indexation_2 = IndexationPayload::unpack_from_slice(indexation_1.pack_to_vec().unwrap()).unwrap();

    assert_eq!(indexation_1, indexation_2);
}
