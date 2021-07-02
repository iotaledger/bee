// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

mod common;

#[test]
fn packable_option() {
    assert_eq!(
        common::generic_test(&Option::<u64>::None).0.len(),
        core::mem::size_of::<u8>()
    );
    assert_eq!(
        common::generic_test(&Option::<u64>::Some(42)).0.len(),
        core::mem::size_of::<u8>() + core::mem::size_of::<u64>()
    );
}
