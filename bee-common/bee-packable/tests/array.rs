// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

mod common;

#[test]
fn packable_array() {
    assert_eq!(
        common::generic_test(&[42u8; 1024]).0.len(),
        1024 * core::mem::size_of::<u8>()
    );
}
