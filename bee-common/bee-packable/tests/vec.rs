// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

mod common;

#[test]
fn packable_vec() {
    assert_eq!(
        common::generic_test(&Vec::<u32>::new()).0.len(),
        core::mem::size_of::<u64>()
    );
    assert_eq!(
        common::generic_test(&vec![Some(0u32), None]).0.len(),
        core::mem::size_of::<u64>()
            + (core::mem::size_of::<u8>() + core::mem::size_of::<u32>())
            + core::mem::size_of::<u8>()
    );
}
