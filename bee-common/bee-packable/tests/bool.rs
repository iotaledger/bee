// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

mod common;

use bee_packable::{packer::VecPacker, Packable, PackableExt};

#[test]
fn packable_bool() {
    assert_eq!(common::generic_test(&false).0.len(), core::mem::size_of::<u8>());
    assert_eq!(common::generic_test(&true).0.len(), core::mem::size_of::<u8>());
}

#[test]
fn packable_bool_packed_non_zero_bytes_are_truthy() {
    let mut packer = VecPacker::default();
    42u8.pack(&mut packer).unwrap();

    let is_true = bool::unpack_verified(&mut packer.as_slice()).unwrap();

    assert!(is_true);
}
