// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

mod common;

#[cfg(feature = "primitive-types")]
use primitive_types::U256;

macro_rules! impl_packable_test_for_integer {
    ($name:ident, $ty:ident, $value:expr) => {
        #[test]
        fn $name() {
            let value: $ty = $value;
            let bytes = common::generic_test(&value);

            assert_eq!(bytes.0.len(), core::mem::size_of::<$ty>());
        }
    };
}

impl_packable_test_for_integer!(packable_i8, i8, 0x6F);
impl_packable_test_for_integer!(packable_u8, u8, 0x6F);
impl_packable_test_for_integer!(packable_i16, i16, 0x6F7B);
impl_packable_test_for_integer!(packable_u16, u16, 0x6F7B);
impl_packable_test_for_integer!(packable_i32, i32, 0x6F7BD423);
impl_packable_test_for_integer!(packable_u32, u32, 0x6F7BD423);
impl_packable_test_for_integer!(packable_i64, i64, 0x6F7BD423100423DB);
impl_packable_test_for_integer!(packable_u64, u64, 0x6F7BD423100423DB);
#[cfg(has_i128)]
impl_packable_test_for_integer!(packable_i128, i128, 0x6F7BD423100423DBFF127B91CA0AB123);
#[cfg(has_u128)]
impl_packable_test_for_integer!(packable_u128, u128, 0x6F7BD423100423DBFF127B91CA0AB123);
#[cfg(feature = "primitive-types")]
impl_packable_test_for_integer!(
    packable_u256,
    U256,
    U256([
        0x6F7BD423100423DB,
        0x6F7BD423100423DB,
        0x6F7BD423100423DB,
        0x6F7BD423100423DB
    ])
);
