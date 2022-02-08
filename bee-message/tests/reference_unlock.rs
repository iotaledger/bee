// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_message::{unlock_block::ReferenceUnlockBlock, Error};

use packable::{bounded::InvalidBoundedU16, error::UnpackError, PackableExt};

#[test]
fn kind() {
    assert_eq!(ReferenceUnlockBlock::KIND, 1);
}

#[test]
fn new_valid_min_index() {
    assert_eq!(ReferenceUnlockBlock::new(0).unwrap().index(), 0);
}

#[test]
fn new_valid_max_index() {
    assert_eq!(ReferenceUnlockBlock::new(126).unwrap().index(), 126);
}

#[test]
fn new_invalid_more_than_max_index() {
    assert!(matches!(
        ReferenceUnlockBlock::new(128),
        Err(Error::InvalidReferenceIndex(InvalidBoundedU16(128)))
    ));
}

#[test]
fn try_from_valid() {
    assert_eq!(ReferenceUnlockBlock::try_from(0).unwrap().index(), 0);
}

#[test]
fn try_from_invalid() {
    assert!(matches!(
        ReferenceUnlockBlock::try_from(128),
        Err(Error::InvalidReferenceIndex(InvalidBoundedU16(128)))
    ));
}

#[test]
fn packed_len() {
    let reference = ReferenceUnlockBlock::new(0).unwrap();

    assert_eq!(reference.packed_len(), 2);
    assert_eq!(reference.pack_to_vec().len(), 2);
}

#[test]
fn pack_unpack_valid() {
    let reference_1 = ReferenceUnlockBlock::try_from(42).unwrap();
    let reference_2 = ReferenceUnlockBlock::unpack_verified(&mut reference_1.pack_to_vec().as_slice()).unwrap();

    assert_eq!(reference_1, reference_2);
}

#[test]
fn pack_unpack_invalid_index() {
    assert!(matches!(
        ReferenceUnlockBlock::unpack_verified(&mut vec![0x2a, 0x2a].as_slice()),
        Err(UnpackError::Packable(Error::InvalidReferenceIndex(InvalidBoundedU16(
            10794
        ))))
    ));
}
