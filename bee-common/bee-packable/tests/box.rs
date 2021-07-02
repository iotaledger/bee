// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

mod common;

#[test]
fn packable_box() {
    assert_eq!(
        common::generic_test(&(Box::new([]) as Box::<[u32]>)).0.len(),
        core::mem::size_of::<u64>()
    );
    assert_eq!(
        common::generic_test(&(Box::new([Some(0u32), None]) as Box::<[Option<u32>]>))
            .0
            .len(),
        core::mem::size_of::<u64>()
            + (core::mem::size_of::<u8>() + core::mem::size_of::<u32>())
            + core::mem::size_of::<u8>()
    );
}
