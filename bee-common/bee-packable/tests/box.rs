// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

mod common;

#[test]
fn packable_box() {
    assert_eq!(
        common::generic_test(&(Box::new(42u64))).0.len(),
        core::mem::size_of::<u64>()
    );
    assert_eq!(
        common::generic_test(&(Box::new(Some([0u8; 5])))).0.len(),
        (core::mem::size_of::<u8>() + 5 * core::mem::size_of::<u8>())
    );
}
