// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_message::{payload::indexation::PaddedIndex, util::hex_decode};
use bee_packable::{Packable, PackableExt};

use core::{ops::Deref, str::FromStr};

const PADDED_INDEX: &str = "52fdfc072182654f163f5f0f9a621d729566c74d10037c4d7bbb0407d1e2c64952fdfc072182654f163f5f0f9a621d729566c74d10037c4d7bbb0407d1e2c649";

#[test]
fn length() {
    assert_eq!(PaddedIndex::LENGTH, 64);
}

#[test]
fn display_impl() {
    assert_eq!(
        format!("{}", PaddedIndex::new(hex_decode(PADDED_INDEX).unwrap())),
        PADDED_INDEX
    );
}

#[test]
fn debug_impl() {
    assert_eq!(
        format!("{:?}", PaddedIndex::new(hex_decode(PADDED_INDEX).unwrap())),
        "PaddedIndex(".to_owned() + PADDED_INDEX + ")"
    );
}

#[test]
fn new_as_ref() {
    assert_eq!(
        PaddedIndex::new([42; PaddedIndex::LENGTH]).as_ref(),
        &[42; PaddedIndex::LENGTH]
    );
}

#[test]
fn new_deref() {
    assert_eq!(
        PaddedIndex::new([42; PaddedIndex::LENGTH]).deref(),
        &[42; PaddedIndex::LENGTH]
    );
}

#[test]
fn from_as_ref() {
    assert_eq!(
        PaddedIndex::from([42; PaddedIndex::LENGTH]).as_ref(),
        &[42; PaddedIndex::LENGTH]
    );
}

#[test]
fn from_str_as_ref() {
    assert_eq!(
        PaddedIndex::from_str(PADDED_INDEX).unwrap().as_ref(),
        &[
            0x52, 0xfd, 0xfc, 0x07, 0x21, 0x82, 0x65, 0x4f, 0x16, 0x3f, 0x5f, 0x0f, 0x9a, 0x62, 0x1d, 0x72, 0x95, 0x66,
            0xc7, 0x4d, 0x10, 0x03, 0x7c, 0x4d, 0x7b, 0xbb, 0x04, 0x07, 0xd1, 0xe2, 0xc6, 0x49, 0x52, 0xfd, 0xfc, 0x07,
            0x21, 0x82, 0x65, 0x4f, 0x16, 0x3f, 0x5f, 0x0f, 0x9a, 0x62, 0x1d, 0x72, 0x95, 0x66, 0xc7, 0x4d, 0x10, 0x03,
            0x7c, 0x4d, 0x7b, 0xbb, 0x04, 0x07, 0xd1, 0xe2, 0xc6, 0x49
        ]
    );
}

#[test]
fn from_to_str() {
    assert_eq!(PADDED_INDEX, PaddedIndex::from_str(PADDED_INDEX).unwrap().to_string());
}

#[test]
fn packed_len() {
    let padded_index = PaddedIndex::from_str(PADDED_INDEX).unwrap();

    assert_eq!(padded_index.packed_len(), PaddedIndex::LENGTH);
    assert_eq!(padded_index.pack_to_vec().len(), PaddedIndex::LENGTH);
}

#[test]
fn packable_round_trip() {
    let padded_index_1 = PaddedIndex::from_str(PADDED_INDEX).unwrap();
    let padded_index_2 = PaddedIndex::unpack_verified(padded_index_1.pack_to_vec()).unwrap();

    assert_eq!(padded_index_1, padded_index_2);
}

#[test]
fn serde_round_trip() {
    let padded_index_1 = PaddedIndex::from_str(PADDED_INDEX).unwrap();
    let json = serde_json::to_string(&padded_index_1).unwrap();
    let padded_index_2 = serde_json::from_str::<PaddedIndex>(&json).unwrap();

    assert_eq!(padded_index_1, padded_index_2);
}
