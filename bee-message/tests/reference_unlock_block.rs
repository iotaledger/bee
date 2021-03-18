// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_common::packable::Packable;
use bee_message::prelude::*;

use core::convert::TryFrom;

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
        ReferenceUnlock::new(127),
        Err(Error::InvalidReferenceIndex(127))
    ));
}

#[test]
fn try_from_valid() {
    assert_eq!(ReferenceUnlock::try_from(0).unwrap().index(), 0);
}

#[test]
fn try_from_invalid() {
    assert!(matches!(
        ReferenceUnlock::try_from(127),
        Err(Error::InvalidReferenceIndex(127))
    ));
}

#[test]
fn packed_len() {
    assert_eq!(ReferenceUnlock::new(0).unwrap().packed_len(), 2);
}

#[test]
fn pack_unpack_valid() {
    let reference_1 = ReferenceUnlock::try_from(42).unwrap();
    let reference_2 = ReferenceUnlock::unpack(&mut reference_1.pack_new().as_slice()).unwrap();

    assert_eq!(reference_1, reference_2);
}

#[test]
fn pack_unpack_invalid_index() {
    assert!(matches!(
        ReferenceUnlock::unpack(&mut vec![0x2a, 0x2a].as_slice()),
        Err(Error::InvalidReferenceIndex(10794))
    ));
}
