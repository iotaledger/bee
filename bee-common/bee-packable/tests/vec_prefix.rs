// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

mod common;

use bee_packable::{
    error::UnpackPrefixError, BoundedU16, BoundedU32, BoundedU64, BoundedU8, InvalidBoundedU32, Packable, UnpackError,
    VecPrefix,
};

use core::convert::TryFrom;

macro_rules! impl_packable_test_for_vec_prefix {
    ($packable_vec_prefix:ident, $packable_vec_prefix_invalid_length:ident, $ty:ty, $bounded:ty) => {
        #[test]
        fn $packable_vec_prefix() {
            assert_eq!(
                common::generic_test(<&VecPrefix<Option<u32>, $bounded>>::try_from(&vec![Some(0u32), None]).unwrap())
                    .0
                    .len(),
                core::mem::size_of::<$ty>()
                    + (core::mem::size_of::<u8>() + core::mem::size_of::<u32>())
                    + core::mem::size_of::<u8>()
            );
        }

        #[test]
        fn $packable_vec_prefix_invalid_length() {
            let mut bytes = vec![0u8; 65];
            bytes[0] = 65;

            let prefixed = VecPrefix::<u8, $bounded>::unpack_from_slice(bytes);

            assert!(matches!(
                prefixed,
                Err(UnpackError::Packable(UnpackPrefixError::InvalidPrefixLength(65))),
            ));
        }
    };
}

impl_packable_test_for_vec_prefix!(packable_vec_prefix_u8, packable_vec_prefix_invalid_length_u8, u8, BoundedU8<1, 64>);
impl_packable_test_for_vec_prefix!(packable_vec_prefix_u16, packable_vec_prefix_invalid_length_u16, u16, BoundedU16<1, 64>);
impl_packable_test_for_vec_prefix!(packable_vec_prefix_u32, packable_vec_prefix_invalid_length_u32, u32, BoundedU32<1, 64>);
impl_packable_test_for_vec_prefix!(packable_vec_prefix_u64, packable_vec_prefix_invalid_length_u64, u64, BoundedU64<1, 64>);

#[test]
fn packable_vec_prefix_from_vec_error() {
    let vec = vec![0u8; 16];
    let prefixed = VecPrefix::<u8, BoundedU32<1, 8>>::try_from(vec);

    assert!(matches!(prefixed, Err(InvalidBoundedU32(16))));
}
