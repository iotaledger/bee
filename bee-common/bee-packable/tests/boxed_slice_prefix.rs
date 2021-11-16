// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

mod common;

use bee_packable::{
    bounded::{
        BoundedU16, BoundedU32, BoundedU64, BoundedU8, InvalidBoundedU16, InvalidBoundedU32, InvalidBoundedU64,
        InvalidBoundedU8,
    },
    error::UnpackError,
    prefix::{BoxedSlicePrefix, TryIntoPrefixError, UnpackPrefixError},
    PackableExt,
};
macro_rules! impl_packable_test_for_boxed_slice_prefix {
    ($packable_boxed_slice_prefix:ident, $packable_boxed_slice_prefix_invalid_length:ident, $ty:ty) => {
        #[test]
        fn $packable_boxed_slice_prefix() {
            assert_eq!(
                common::generic_test(
                    <&BoxedSlicePrefix<Option<u32>, $ty>>::try_from(&vec![Some(0u32), None].into_boxed_slice())
                        .unwrap()
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

macro_rules! impl_packable_test_for_bounded_boxed_slice_prefix {
    ($packable_boxed_slice_prefix:ident, $packable_boxed_slice_prefix_invalid_length:ident, $ty:ty, $bounded:ident, $err:ident, $min:expr, $max:expr) => {
        #[test]
        fn $packable_boxed_slice_prefix() {
            assert_eq!(
                common::generic_test(
                    <&BoxedSlicePrefix<Option<u32>, $bounded<$min, $max>>>::try_from(
                        &vec![Some(0u32), None].into_boxed_slice()
                    )
                    .unwrap()
                )
                .0
                .len(),
                core::mem::size_of::<$ty>()
                    + (core::mem::size_of::<u8>() + core::mem::size_of::<u32>())
                    + core::mem::size_of::<u8>()
            );
        }

        #[test]
        fn $packable_boxed_slice_prefix_invalid_length() {
            const LEN: usize = $max + 1;

            let mut bytes = vec![0u8; LEN + 1].into_boxed_slice();
            bytes[0] = LEN as u8;

            let prefixed = BoxedSlicePrefix::<u8, $bounded<$min, $max>>::unpack_verified(bytes);

            const LEN_AS_TY: $ty = LEN as $ty;

            assert!(matches!(
                prefixed,
                Err(UnpackError::Packable(UnpackPrefixError::Prefix($err(LEN_AS_TY)))),
            ));
        }
    };
}

impl_packable_test_for_boxed_slice_prefix!(
    packable_boxed_slice_prefix_u8,
    packable_boxed_slice_prefix_invalid_length_u8,
    u8
);
impl_packable_test_for_boxed_slice_prefix!(
    packable_boxed_slice_prefix_u16,
    packable_boxed_slice_prefix_invalid_length_u16,
    u16
);
impl_packable_test_for_boxed_slice_prefix!(
    packable_boxed_slice_prefix_u32,
    packable_boxed_slice_prefix_invalid_length_u32,
    u32
);
impl_packable_test_for_boxed_slice_prefix!(
    packable_boxed_slice_prefix_u64,
    packable_boxed_slice_prefix_invalid_length_u64,
    u64
);

impl_packable_test_for_bounded_boxed_slice_prefix!(
    packable_boxed_slice_prefix_bounded_u8,
    packable_boxed_slice_prefix_invalid_length_bounded_u8,
    u8,
    BoundedU8,
    InvalidBoundedU8,
    1,
    64
);
impl_packable_test_for_bounded_boxed_slice_prefix!(
    packable_boxed_slice_prefix_bounded_u16,
    packable_boxed_slice_prefix_invalid_length_bounded_u16,
    u16,
    BoundedU16,
    InvalidBoundedU16,
    1,
    64
);
impl_packable_test_for_bounded_boxed_slice_prefix!(
    packable_boxed_slice_prefix_bounded_u32,
    packable_boxed_slice_prefix_invalid_length_bounded_u32,
    u32,
    BoundedU32,
    InvalidBoundedU32,
    1,
    64
);
impl_packable_test_for_bounded_boxed_slice_prefix!(
    packable_boxed_slice_prefix_bounded_u64,
    packable_boxed_slice_prefix_invalid_length_bounded_u64,
    u64,
    BoundedU64,
    InvalidBoundedU64,
    1,
    64
);

#[test]
fn packable_boxed_slice_prefix_from_boxed_slice_invalid_error() {
    let boxed_slice = vec![0u8; 16].into_boxed_slice();
    let prefixed = BoxedSlicePrefix::<u8, BoundedU32<1, 8>>::try_from(boxed_slice);

    assert!(matches!(
        prefixed,
        Err(TryIntoPrefixError::Invalid(InvalidBoundedU32(16)))
    ));
}

#[test]
fn packable_boxed_slice_prefix_from_boxed_slice_truncated_error() {
    let boxed_slice = vec![0u8; 257].into_boxed_slice();
    let prefixed = BoxedSlicePrefix::<u8, u8>::try_from(boxed_slice);

    assert!(matches!(prefixed, Err(TryIntoPrefixError::Truncated(257))));
}
