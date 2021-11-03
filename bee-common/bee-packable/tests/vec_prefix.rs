// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

mod common;

use bee_packable::{
    error::UnpackPrefixError, packable::VecPrefixLengthError, BoundedU16, BoundedU32, BoundedU64, BoundedU8,
    InvalidBoundedU16, InvalidBoundedU32, InvalidBoundedU64, InvalidBoundedU8, Packable, UnpackError, VecPrefix,
};

macro_rules! impl_packable_test_for_vec_prefix {
    ($packable_vec_prefix:ident, $packable_vec_prefix_invalid_length:ident, $ty:ty, $bounded:ident, $err:ident, $min:expr, $max:expr) => {
        #[test]
        fn $packable_vec_prefix() {
            assert_eq!(
                common::generic_test(
                    <&VecPrefix<Option<u32>, $bounded<$min, $max>>>::try_from(&vec![Some(0u32), None]).unwrap()
                )
                .0
                .len(),
                core::mem::size_of::<$ty>()
                    + (core::mem::size_of::<u8>() + core::mem::size_of::<u32>())
                    + core::mem::size_of::<u8>()
            );
        }

        #[test]
        fn $packable_vec_prefix_invalid_length() {
            const LEN: usize = 65;

            let mut bytes = vec![0u8; LEN + 1];
            bytes[0] = LEN as u8;

            let prefixed = VecPrefix::<u8, $bounded<$min, $max>>::unpack_from_slice(bytes);

            const LEN_AS_TY: $ty = LEN as $ty;

            assert!(matches!(
                prefixed,
                Err(UnpackError::Packable(UnpackPrefixError::Prefix($err(LEN_AS_TY)))),
            ));
        }
    };
}

impl_packable_test_for_vec_prefix!(
    packable_vec_prefix_u8,
    packable_vec_prefix_invalid_length_u8,
    u8,
    BoundedU8,
    InvalidBoundedU8,
    1,
    64
);
impl_packable_test_for_vec_prefix!(
    packable_vec_prefix_u16,
    packable_vec_prefix_invalid_length_u16,
    u16,
    BoundedU16,
    InvalidBoundedU16,
    1,
    64
);
impl_packable_test_for_vec_prefix!(
    packable_vec_prefix_u32,
    packable_vec_prefix_invalid_length_u32,
    u32,
    BoundedU32,
    InvalidBoundedU32,
    1,
    64
);
impl_packable_test_for_vec_prefix!(
    packable_vec_prefix_u64,
    packable_vec_prefix_invalid_length_u64,
    u64,
    BoundedU64,
    InvalidBoundedU64,
    1,
    64
);

#[test]
fn packable_vec_prefix_from_vec_error() {
    let vec = vec![0u8; 16];
    let prefixed = VecPrefix::<u8, BoundedU32<1, 8>>::try_from(vec);

    assert!(matches!(
        prefixed,
        Err(VecPrefixLengthError::Invalid(InvalidBoundedU32(16)))
    ));
}
