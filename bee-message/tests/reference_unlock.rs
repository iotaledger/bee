// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_message::prelude::*;
use bee_packable::{Packable, UnpackError};

use core::convert::TryFrom;

#[test]
fn kind() {
    assert_eq!(ReferenceUnlock::KIND, 1);
}

#[test]
fn new_valid_min_index() {
    assert_eq!(ReferenceUnlock::new(0).unwrap().index(), 0);
}

#[test]
fn new_valid_max_index() {
    assert_eq!(ReferenceUnlock::new(126).unwrap().index(), 126);
}

#[test]
fn new_invalid_more_than_max_index() {
    assert!(matches!(
        ReferenceUnlock::new(127).err().unwrap(),
        ValidationError::InvalidReferenceIndex(127),
    ));
}

#[test]
fn try_from_valid() {
    assert_eq!(ReferenceUnlock::try_from(0).unwrap().index(), 0);
}

#[test]
fn try_from_invalid() {
    assert!(matches!(
        ReferenceUnlock::try_from(127).err().unwrap(),
        ValidationError::InvalidReferenceIndex(127),
    ));
}

#[test]
fn unpack_invalid_index() {
    assert!(matches!(
        ReferenceUnlock::unpack_from_slice(vec![0x2a, 0x2a]).err().unwrap(),
        UnpackError::Packable(MessageUnpackError::ValidationError(
            ValidationError::InvalidReferenceIndex(10794)
        )),
    ));
}

#[test]
fn packed_len() {
    let reference = ReferenceUnlock::new(0).unwrap();

    assert_eq!(reference.packed_len(), 2);
    assert_eq!(reference.pack_to_vec().unwrap().len(), 2);
}

#[test]
fn round_trip() {
    let reference_1 = ReferenceUnlock::try_from(42).unwrap();
    let reference_2 = ReferenceUnlock::unpack_from_slice(reference_1.pack_to_vec().unwrap()).unwrap();

    assert_eq!(reference_1, reference_2);
}
