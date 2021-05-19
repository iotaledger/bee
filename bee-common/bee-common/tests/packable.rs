// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0
extern crate alloc;

use bee_common::packable::{Packable, VecPacker};

use alloc::vec::Vec;
use core::{fmt::Debug, mem::size_of};

fn pack_checked<P>(value: P) -> VecPacker
where
    P: Packable + Eq + Debug,
    P::Error: Debug,
{
    let mut packer = VecPacker::default();
    value.pack(&mut packer).unwrap();

    let result = Packable::unpack(&mut packer.as_slice()).unwrap();

    assert_eq!(value.packed_len(), packer.len());
    assert_eq!(value, result);

    packer
}

macro_rules! impl_packable_test_for_num {
    ($name:ident, $ty:ident, $value:expr) => {
        #[test]
        fn $name() {
            let value: $ty = $value;
            let bytes = pack_checked(value);
            assert_eq!(bytes.len(), size_of::<$ty>());
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
#[cfg(has_i128)]
impl_packable_test_for_num!(packable_i128, i128, 0x6F7BD423100423DBFF127B91CA0AB123);
#[cfg(has_u128)]
impl_packable_test_for_num!(packable_u128, u128, 0x6F7BD423100423DBFF127B91CA0AB123);

#[test]
fn packable_bool() {
    assert_eq!(pack_checked(false).len(), size_of::<u8>());
    assert_eq!(pack_checked(true).len(), size_of::<u8>());
}

#[test]
fn packed_non_zero_bytes_are_truthy() {
    let mut packer = VecPacker::default();
    42u8.pack(&mut packer).unwrap();

    let is_true = bool::unpack(&mut packer.as_slice()).unwrap();

    assert!(is_true);
}

#[test]
fn packable_option() {
    assert_eq!(pack_checked(Option::<u64>::None).len(), size_of::<u8>());
    assert_eq!(
        pack_checked(Option::<u64>::Some(42)).len(),
        size_of::<u8>() + size_of::<u64>()
    );
}

#[test]
fn packable_vector() {
    assert_eq!(pack_checked(Vec::<u32>::new()).len(), size_of::<u64>());
    assert_eq!(
        pack_checked(vec![Some(0u32), None]).len(),
        size_of::<u64>() + (size_of::<u8>() + size_of::<u32>()) + size_of::<u8>()
    );
}

#[test]
fn packable_array() {
    assert_eq!(pack_checked([42u8; 1024]).len(), 1024 * size_of::<u8>());
}
