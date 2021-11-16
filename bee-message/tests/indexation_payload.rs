// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_common::packable::Packable;
use bee_message::prelude::*;
use bee_test::rand::bytes::{rand_bytes, rand_bytes_array};

const PADDED_INDEX: &str = "52fdfc072182654f163f5f0f9a621d729566c74d10037c4d7bbb0407d1e2c64952fdfc072182654f163f5f0f9a621d729566c74d10037c4d7bbb0407d1e2c649";

#[test]
fn kind() {
    assert_eq!(IndexationPayload::KIND, 2);
}

#[test]
fn debug_impl() {
    assert_eq!(
        format!(
            "{:?}",
            PaddedIndex::new(hex::decode(PADDED_INDEX).unwrap().try_into().unwrap())
        ),
        "PaddedIndex(52fdfc072182654f163f5f0f9a621d729566c74d10037c4d7bbb0407d1e2c64952fdfc072182654f163f5f0f9a621d729566c74d10037c4d7bbb0407d1e2c649)"
    );
}

#[test]
fn new_valid() {
    let index = rand_bytes(64);
    let index_array: [u8; 64] = index.as_slice().try_into().unwrap();
    let data = [0x42, 0xff, 0x84, 0xa2, 0x42, 0xff, 0x84, 0xa2];
    let indexation = IndexationPayload::new(&index, &data).unwrap();

    assert_eq!(indexation.index(), &index);
    assert_eq!(indexation.padded_index().as_ref(), index.as_slice());
    assert_eq!(*indexation.padded_index(), index_array);
    assert_eq!(indexation.data(), &data);
}

#[test]
fn new_valid_empty_data() {
    let index = rand_bytes(64);
    let index_array: [u8; 64] = index.as_slice().try_into().unwrap();
    let data = [];
    let indexation = IndexationPayload::new(&index, &data).unwrap();

    assert_eq!(indexation.index(), &index);
    assert_eq!(indexation.padded_index().as_ref(), index.as_slice());
    assert_eq!(*indexation.padded_index(), index_array);
    assert_eq!(indexation.data(), &data);
}

#[test]
fn new_valid_padded() {
    let index = rand_bytes(32);
    let mut padded_index = index.clone();
    padded_index.append(&mut vec![0u8; 32]);
    let data = [];
    let indexation = IndexationPayload::new(&index, &data).unwrap();

    assert_eq!(indexation.index(), &index);
    assert_eq!(indexation.padded_index().as_ref(), padded_index.as_slice());
    assert_eq!(indexation.data(), &data);
}

#[test]
fn new_invalid_index_length_less_than_min() {
    assert!(matches!(
        IndexationPayload::new(&[], &[0x42, 0xff, 0x84, 0xa2, 0x42, 0xff, 0x84, 0xa2]),
        Err(Error::InvalidIndexationIndexLength(0))
    ));
}

#[test]
fn new_invalid_index_length_more_than_max() {
    assert!(matches!(
        IndexationPayload::new(&rand_bytes(65), &[0x42, 0xff, 0x84, 0xa2, 0x42, 0xff, 0x84, 0xa2]),
        Err(Error::InvalidIndexationIndexLength(65))
    ));
}

#[test]
fn new_invalid_data_length_more_than_max() {
    assert!(matches!(
        IndexationPayload::new(&rand_bytes_array::<32>(), &[0u8; MESSAGE_LENGTH_MAX + 42]),
        Err(Error::InvalidIndexationDataLength(l)) if l == MESSAGE_LENGTH_MAX + 42
    ));
}

#[test]
fn packed_len() {
    let indexation =
        IndexationPayload::new(&rand_bytes(10), &[0x42, 0xff, 0x84, 0xa2, 0x42, 0xff, 0x84, 0xa2]).unwrap();

    assert_eq!(indexation.packed_len(), 10 + 2 + 4 + 8);
    assert_eq!(indexation.pack_new().len(), 10 + 2 + 4 + 8);
}

#[test]
fn pack_unpack_valid() {
    let indexation_1 = IndexationPayload::new(
        &rand_bytes_array::<32>(),
        &[0x42, 0xff, 0x84, 0xa2, 0x42, 0xff, 0x84, 0xa2],
    )
    .unwrap();
    let indexation_2 = IndexationPayload::unpack(&mut indexation_1.pack_new().as_slice()).unwrap();

    assert_eq!(indexation_1.index(), indexation_2.index());
    assert_eq!(indexation_1.data(), indexation_2.data());
}

#[test]
fn unpack_invalid_index_length_less_than_min() {
    assert!(matches!(
        IndexationPayload::unpack(&mut vec![0x00, 0x00, 0x00, 0x00, 0x00, 0x00].as_slice()),
        Err(Error::InvalidIndexationIndexLength(0))
    ));
}

#[test]
fn unpack_invalid_index_length_more_than_max() {
    assert!(matches!(
        IndexationPayload::unpack(
            &mut vec![
                0x41, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00
            ]
            .as_slice()
        ),
        Err(Error::InvalidIndexationIndexLength(65))
    ));
}

#[test]
fn unpack_invalid_data_length_more_than_max() {
    assert!(matches!(
        IndexationPayload::unpack(&mut vec![0x02, 0x00, 0x00, 0x00, 0x35, 0x82, 0x00, 0x00].as_slice()),
        Err(Error::InvalidIndexationDataLength(33333))
    ));
}

#[test]
fn unpack_valid_padded() {
    let index = rand_bytes(32);
    let mut padded_index = index.clone();
    padded_index.append(&mut vec![0u8; 32]);

    let indexation_1 = IndexationPayload::new(&index, &[0x42, 0xff, 0x84, 0xa2, 0x42, 0xff, 0x84, 0xa2]).unwrap();
    let indexation_2 = IndexationPayload::unpack(&mut indexation_1.pack_new().as_slice()).unwrap();

    assert_eq!(indexation_1.index(), indexation_2.index());
    assert_eq!(indexation_1.padded_index(), indexation_2.padded_index());
    assert_eq!(indexation_1.data(), indexation_2.data());
}
