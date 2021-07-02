// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

mod common;

use bee_packable::VecPrefix;

macro_rules! impl_packable_test_for_vec_prefix {
    ($name:ident, $ty:ty) => {
        #[test]
        fn $name() {
            assert_eq!(
                common::generic_test(&VecPrefix::<u32, $ty>::new()).0.len(),
                core::mem::size_of::<$ty>()
            );
            assert_eq!(
                common::generic_test(&VecPrefix::<Option<u32>, $ty>::from(vec![Some(0u32), None]))
                    .0
                    .len(),
                core::mem::size_of::<$ty>()
                    + (core::mem::size_of::<u8>() + core::mem::size_of::<u32>())
                    + core::mem::size_of::<u8>()
            );
        }
    };
}

impl_packable_test_for_vec_prefix!(packable_vec_prefix_u8, u8);
impl_packable_test_for_vec_prefix!(packable_vec_prefix_u16, u16);
impl_packable_test_for_vec_prefix!(packable_vec_prefix_u32, u32);
impl_packable_test_for_vec_prefix!(packable_vec_prefix_u64, u64);
#[cfg(has_u128)]
impl_packable_test_for_vec_prefix!(packable_vec_prefix_u128, u128);
