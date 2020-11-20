// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_common::packable::Packable;

macro_rules! impl_packable_test_for_num {
    ($name:ident, $ty:ident, $value:expr) => {
        #[test]
        fn $name() {
            let num: $ty = $value;
            let bytes = num.pack_new();

            assert_eq!(bytes.len(), num.packed_len());
            assert_eq!($ty::unpack(&mut bytes.as_slice()).unwrap(), num);
        }
    };
}

impl_packable_test_for_num!(packable_i8, i8, 0x6F);
impl_packable_test_for_num!(packable_u8, u8, 0x6F);
impl_packable_test_for_num!(packable_i16, i16, 0x6F7B);
impl_packable_test_for_num!(packable_u16, u16, 0x6F7B);
impl_packable_test_for_num!(packable_i32, i32, 0x6F7BD423);
impl_packable_test_for_num!(packable_u32, u32, 0x6F7BD423);
impl_packable_test_for_num!(packable_i64, i64, 0x6F7BD423100423DB);
impl_packable_test_for_num!(packable_u64, u64, 0x6F7BD423100423DB);
impl_packable_test_for_num!(packable_i128, i128, 0x6F7BD423100423DBFF127B91CA0AB123);
impl_packable_test_for_num!(packable_u128, u128, 0x6F7BD423100423DBFF127B91CA0AB123);

#[test]
fn packable_bool() {
    assert_eq!(false.packed_len(), 1);
    assert_eq!(bool::unpack(&mut false.pack_new().as_slice()).unwrap(), false);
    assert_eq!(bool::unpack(&mut 0u8.pack_new().as_slice()).unwrap(), false);

    assert_eq!(true.packed_len(), 1);
    assert_eq!(bool::unpack(&mut true.pack_new().as_slice()).unwrap(), true);
    assert_eq!(bool::unpack(&mut 1u8.pack_new().as_slice()).unwrap(), true);
    assert_eq!(bool::unpack(&mut 42u8.pack_new().as_slice()).unwrap(), true);
}

#[test]
fn packable_option() {
    assert_eq!(None::<u64>.packed_len(), 1);
    assert_eq!(
        Option::<u64>::unpack(&mut None::<u64>.pack_new().as_slice()).unwrap(),
        None
    );

    assert_eq!(Some(42u64).packed_len(), 9);
    assert_eq!(
        Option::<u64>::unpack(&mut Some(42u64).pack_new().as_slice()).unwrap(),
        Some(42u64)
    );
}
