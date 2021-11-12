// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

mod common;

use bee_packable::{
    bounded::{
        Bounded, BoundedU16, BoundedU32, BoundedU64, BoundedU8, InvalidBoundedU16, InvalidBoundedU32,
        InvalidBoundedU64, InvalidBoundedU8,
    },
    error::UnpackError,
    PackableExt,
};

macro_rules! impl_bounds_test_for_bounded_integer {
    ($name:ident, $wrapper:ty, $error:ident, $wrapped:ty) => {
        #[test]
        fn $name() {
            let valid = <$wrapper>::MIN;

            let wrapper = <$wrapper>::try_from(valid);
            assert!(wrapper.is_ok());

            let wrapped: $wrapped = wrapper.unwrap().into();
            assert_eq!(wrapped, valid);

            let invalid = <$wrapper>::MAX + 1;
            assert!(matches!(
                <$wrapper>::try_from(invalid),
                Err($error(val))
                    if val == invalid,
            ));
        }
    };
}

macro_rules! impl_packable_test_for_bounded_integer {
    ($packable_valid_name:ident, $packable_invalid_name:ident, $wrapper:ty, $error:ident, $wrapped:ty) => {
        #[test]
        fn $packable_valid_name() {
            let valid = <$wrapper>::MIN;

            assert_eq!(
                common::generic_test(&<$wrapper>::try_from(valid).unwrap())
                    .0
                    .len(),
                core::mem::size_of::<$wrapped>()
            );
        }

        #[test]
        fn $packable_invalid_name() {
            let bytes = vec![0u8; core::mem::size_of::<$wrapped>()];
            let unpacked = <$wrapper>::unpack_verified(&bytes);

            assert!(
                matches!(unpacked, Err(UnpackError::Packable($error(0)))),
                "found {:?}",
                unpacked
            )
        }
    };
}

impl_bounds_test_for_bounded_integer!(bounded_u8, BoundedU8<1, 8>, InvalidBoundedU8, u8);
impl_bounds_test_for_bounded_integer!(bounded_u16, BoundedU16<1, 8>, InvalidBoundedU16, u16);
impl_bounds_test_for_bounded_integer!(bounded_u32, BoundedU32<1, 8>, InvalidBoundedU32, u32);
impl_bounds_test_for_bounded_integer!(bounded_u64, BoundedU64<1, 8>, InvalidBoundedU64, u64);

impl_packable_test_for_bounded_integer!(packable_bounded_u8, packable_bounded_u8_invalid, BoundedU8<1, 8>, InvalidBoundedU8, u8);
impl_packable_test_for_bounded_integer!(packable_bounded_u16, packable_bounded_u16_invalid, BoundedU16<1, 8>, InvalidBoundedU16, u16);
impl_packable_test_for_bounded_integer!(packable_bounded_u32, packable_bounded_u32_invalid, BoundedU32<1, 8>, InvalidBoundedU32, u32);
impl_packable_test_for_bounded_integer!(packable_bounded_u64, packable_bounded_u64_invalid, BoundedU64<1, 8>, InvalidBoundedU64, u64);
