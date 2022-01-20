// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_message::{payload::tagged_data::TaggedDataPayload, Error, Message};
use bee_test::rand::bytes::rand_bytes;

use packable::{
    bounded::{TryIntoBoundedU32Error, TryIntoBoundedU8Error},
    error::UnpackError,
    PackableExt,
};

#[test]
fn kind() {
    assert_eq!(TaggedDataPayload::KIND, 5);
}

#[test]
fn new_valid() {
    let tag = rand_bytes(64);
    let data = vec![0x42, 0xff, 0x84, 0xa2, 0x42, 0xff, 0x84, 0xa2];
    let tagged_data = TaggedDataPayload::new(tag.clone(), data.clone()).unwrap();

    assert_eq!(tagged_data.tag(), &tag);
    assert_eq!(tagged_data.data(), &data);
}

#[test]
fn new_valid_empty_data() {
    let tag = rand_bytes(64);
    let data = vec![];
    let tagged_data = TaggedDataPayload::new(tag.clone(), data.clone()).unwrap();

    assert_eq!(tagged_data.tag(), &tag);
    assert_eq!(tagged_data.data(), &data);
}

#[test]
fn new_valid_padded() {
    let tag = rand_bytes(32);
    let data = vec![];
    let tagged_data = TaggedDataPayload::new(tag.clone(), data.clone()).unwrap();

    assert_eq!(tagged_data.tag(), &tag);
    assert_eq!(tagged_data.data(), &data);
}

#[test]
fn new_invalid_tag_length_less_than_min() {
    assert!(matches!(
        TaggedDataPayload::new(vec![], vec![0x42, 0xff, 0x84, 0xa2, 0x42, 0xff, 0x84, 0xa2]),
        Err(Error::InvalidTagLength(TryIntoBoundedU8Error::Invalid(0)))
    ));
}

#[test]
fn new_invalid_tag_length_more_than_max() {
    assert!(matches!(
        TaggedDataPayload::new(rand_bytes(65), vec![0x42, 0xff, 0x84, 0xa2, 0x42, 0xff, 0x84, 0xa2]),
        Err(Error::InvalidTagLength(TryIntoBoundedU8Error::Invalid(65)))
    ));
}

#[test]
fn new_invalid_data_length_more_than_max() {
    assert!(matches!(
        TaggedDataPayload::new(rand_bytes(32), vec![0u8; Message::LENGTH_MAX + 42]),
        Err(Error::InvalidTaggedDataLength(TryIntoBoundedU32Error::Invalid(l))) if l == Message::LENGTH_MAX as u32 + 42
    ));
}

#[test]
fn packed_len() {
    let tagged_data =
        TaggedDataPayload::new(rand_bytes(10), vec![0x42, 0xff, 0x84, 0xa2, 0x42, 0xff, 0x84, 0xa2]).unwrap();

    assert_eq!(tagged_data.packed_len(), 10 + 1 + 4 + 8);
    assert_eq!(tagged_data.pack_to_vec().len(), 10 + 1 + 4 + 8);
}

#[test]
fn pack_unpack_valid() {
    let tagged_data_1 =
        TaggedDataPayload::new(rand_bytes(32), vec![0x42, 0xff, 0x84, 0xa2, 0x42, 0xff, 0x84, 0xa2]).unwrap();
    let tagged_data_2 = TaggedDataPayload::unpack_verified(&mut tagged_data_1.pack_to_vec().as_slice()).unwrap();

    assert_eq!(tagged_data_1.tag(), tagged_data_2.tag());
    assert_eq!(tagged_data_1.data(), tagged_data_2.data());
}

#[test]
fn unpack_invalid_tag_length_less_than_min() {
    assert!(matches!(
        TaggedDataPayload::unpack_verified(&mut vec![0x00, 0x00, 0x00, 0x00, 0x00, 0x00].as_slice()),
        Err(UnpackError::Packable(Error::InvalidTagLength(
            TryIntoBoundedU8Error::Invalid(0)
        )))
    ));
}

#[test]
fn unpack_invalid_tag_length_more_than_max() {
    assert!(matches!(
        TaggedDataPayload::unpack_verified(
            &mut vec![
                0x41, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00
            ]
            .as_slice()
        ),
        Err(UnpackError::Packable(Error::InvalidTagLength(
            TryIntoBoundedU8Error::Invalid(65)
        )))
    ));
}

#[test]
fn unpack_invalid_data_length_more_than_max() {
    assert!(matches!(
        TaggedDataPayload::unpack_verified(&mut vec![0x02, 0x00, 0x00, 0x35, 0x82, 0x00, 0x00].as_slice()),
        Err(UnpackError::Packable(Error::InvalidTaggedDataLength(
            TryIntoBoundedU32Error::Invalid(33333)
        )))
    ));
}
