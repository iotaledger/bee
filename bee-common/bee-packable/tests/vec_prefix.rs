// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

mod common;

use bee_packable::{error::UnpackPrefixError, Packable, PrefixedFromVecError, UnpackError, VecPrefix};

use core::convert::TryFrom;

const MAX_LENGTH: usize = 128;

macro_rules! impl_packable_test_for_vec_prefix {
    ($name:ident, $ty:ty) => {
        #[test]
        fn $name() {
            assert_eq!(
                common::generic_test(&VecPrefix::<u32, $ty, MAX_LENGTH>::new())
                    .0
                    .len(),
                core::mem::size_of::<$ty>()
            );
            assert_eq!(
                common::generic_test(
                    &VecPrefix::<Option<u32>, $ty, MAX_LENGTH>::try_from(vec![Some(0u32), None]).unwrap()
                )
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

macro_rules! impl_test_for_invalid_prefix_length {
    ($name:ident, $ty:ty) => {
        #[test]
        fn $name() {
            let mut bytes = vec![0u8; MAX_LENGTH + 2];
            bytes[0] = MAX_LENGTH as u8 + 1;

            let prefixed = VecPrefix::<u8, $ty, MAX_LENGTH>::unpack_from_slice(bytes);

            assert!(matches!(
                prefixed,
                Err(UnpackError::Packable(UnpackPrefixError::InvalidPrefixLength(l))) if l == MAX_LENGTH + 1, 
            ));
        }
    };
}

impl_test_for_invalid_prefix_length!(invalid_prefix_length_u8, u8);
impl_test_for_invalid_prefix_length!(invalid_prefix_length_u16, u16);
impl_test_for_invalid_prefix_length!(invalid_prefix_length_u32, u32);
impl_test_for_invalid_prefix_length!(invalid_prefix_length_u64, u64);
#[cfg(has_u128)]
impl_test_for_invalid_prefix_length!(invalid_prefix_length_u128, u128);

#[test]
fn from_vec_error() {
    let vec = vec![0u8; 16];
    let prefixed = VecPrefix::<u8, u32, 8>::try_from(vec);

    assert!(matches!(prefixed, Err(PrefixedFromVecError { max_len: 8, actual_len: 16 })));
}
