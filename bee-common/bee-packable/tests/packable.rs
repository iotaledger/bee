// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

extern crate alloc;

use bee_packable::{packer::VecPacker, Packable, SliceUnpacker, VecPrefix};

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

fn pack_new_checked<P>(value: P) -> Vec<u8>
where
    P: Packable + Eq + Debug,
    P::Error: Debug,
{
    let bytes = value.pack_new();

    let mut unpacker = SliceUnpacker::new(&bytes.as_slice());
    let result: P = Packable::unpack(&mut unpacker).unwrap();

    assert_eq!(value.packed_len(), bytes.len());
    assert_eq!(value, result);

    bytes
}

macro_rules! impl_pack_new_test_for_num {
    ($name:ident, $ty:ident, $value:expr) => {
        #[test]
        fn $name() {
            let value: $ty = $value;
            let bytes = pack_new_checked(value);
            assert_eq!(bytes.len(), size_of::<$ty>());
        }
    };
}

impl_pack_new_test_for_num!(pack_new_i8, i8, 0x6F);
impl_pack_new_test_for_num!(pack_new_u8, u8, 0x6F);
impl_pack_new_test_for_num!(pack_new_i16, i16, 0x6F7B);
impl_pack_new_test_for_num!(pack_new_u16, u16, 0x6F7B);
impl_pack_new_test_for_num!(pack_new_i32, i32, 0x6F7BD423);
impl_pack_new_test_for_num!(pack_new_u32, u32, 0x6F7BD423);
impl_pack_new_test_for_num!(pack_new_i64, i64, 0x6F7BD423100423DB);
impl_pack_new_test_for_num!(pack_new_u64, u64, 0x6F7BD423100423DB);
#[cfg(has_i128)]
impl_pack_new_test_for_num!(pack_new_i128, i128, 0x6F7BD423100423DBFF127B91CA0AB123);
#[cfg(has_u128)]
impl_pack_new_test_for_num!(pack_new_u128, u128, 0x6F7BD423100423DBFF127B91CA0AB123);

#[test]
fn pack_new_bool() {
    assert_eq!(pack_new_checked(false).len(), size_of::<u8>());
    assert_eq!(pack_new_checked(true).len(), size_of::<u8>());
}

#[test]
fn pack_new_option() {
    assert_eq!(pack_new_checked(Option::<u64>::None).len(), size_of::<u8>());
    assert_eq!(
        pack_new_checked(Option::<u64>::Some(42)).len(),
        size_of::<u8>() + size_of::<u64>()
    );
}

#[test]
fn pack_new_vector() {
    assert_eq!(pack_new_checked(Vec::<u32>::new()).len(), size_of::<u64>());
    assert_eq!(
        pack_new_checked(vec![Some(0u32), None]).len(),
        size_of::<u64>() + (size_of::<u8>() + size_of::<u32>()) + size_of::<u8>()
    );
}

#[test]
fn pack_new_array() {
    assert_eq!(pack_new_checked([42u8; 1024]).len(), 1024 * size_of::<u8>());
}

macro_rules! packable_vec_prefix {
    ($name:ident, $ty:ty) => {
        #[test]
        fn $name() {
            assert_eq!(pack_checked(VecPrefix::<u32, $ty>::new()).len(), size_of::<$ty>());
            assert_eq!(
                pack_checked(VecPrefix::<Option<u32>, $ty>::from(vec![Some(0u32), None])).len(),
                size_of::<$ty>() + (size_of::<u8>() + size_of::<u32>()) + size_of::<u8>()
            );
        }
    };
}

packable_vec_prefix!(packable_vec_prefix_u8, u8);
packable_vec_prefix!(packable_vec_prefix_u16, u16);
packable_vec_prefix!(packable_vec_prefix_u32, u32);
packable_vec_prefix!(packable_vec_prefix_u64, u64);
#[cfg(has_u128)]
packable_vec_prefix!(packable_vec_prefix_u128, u128);

macro_rules! pack_new_vec_prefix {
    ($name:ident, $ty:ty) => {
        #[test]
        fn $name() {
            assert_eq!(
                pack_new_checked(VecPrefix::<u32, $ty>::new()).len(),
                size_of::<$ty>()
            );
            assert_eq!(
                pack_new_checked(VecPrefix::<Option<u32>, $ty>::from(vec![Some(0u32), None])).len(),
                size_of::<$ty>() + (size_of::<u8>() + size_of::<u32>()) + size_of::<u8>()
            );
        }
    };
}

pack_new_vec_prefix!(pack_new_vec_prefix_u8, u8);
pack_new_vec_prefix!(pack_new_vec_prefix_u16, u16);
pack_new_vec_prefix!(pack_new_vec_prefix_u32, u32);
pack_new_vec_prefix!(pack_new_vec_prefix_u64, u64);
#[cfg(has_u128)]
pack_new_vec_prefix!(pack_new_vec_prefix_u128, u128);

fn round_trip<P>(value: P)
where
    P: Packable + Eq + Debug,
    P::Error: Debug,
{
    let bytes = value.pack_new();
    let unpacked = Packable::unpack_from_bytes(&bytes).unwrap();

    assert_eq!(value, unpacked);
}

macro_rules! impl_round_trip_test {
    ($name:ident, $value:expr) => {
        #[test]
        fn $name() {
            round_trip($value);
        }
    };
}

impl_round_trip_test!(round_trip_i8, 0x6Fi8);
impl_round_trip_test!(round_trip_u8, 0x6Fu8);
impl_round_trip_test!(round_trip_i16, 0x6F7Bi16);
impl_round_trip_test!(round_trip_u16, 0x6F7Bu16);
impl_round_trip_test!(round_trip_i32, 0x6F7BD423i32);
impl_round_trip_test!(round_trip_u32, 0x6F7BD423u32);
impl_round_trip_test!(round_trip_i64, 0x6F7BD423100423DBi64);
impl_round_trip_test!(round_trip_u64, 0x6F7BD423100423DBu64);
#[cfg(has_i128)]
impl_round_trip_test!(round_trip_i128, 0x6F7BD423100423DBFF127B91CA0AB123i128);
#[cfg(has_u128)]
impl_round_trip_test!(round_trip_u128, 0x6F7BD423100423DBFF127B91CA0AB123u128);

impl_round_trip_test!(round_trip_bool, true);
impl_round_trip_test!(round_trip_option, Some(42u32));
impl_round_trip_test!(round_trip_vector, vec![Some(0u32), None]);
impl_round_trip_test!(round_trip_array, [42u8; 1024]);
