// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_common::packable::{PackError, Packable, Packer, UnpackError, Unpacker};

use core::{fmt::Debug, mem::size_of};

#[derive(Debug)]
enum Error {
    UnexpectedEof,
    InvalidVariant(&'static str, u64),
}

impl PackError for Error {}

impl UnpackError for Error {
    fn invalid_variant<P: Packable>(value: u64) -> Self {
        Self::InvalidVariant(core::any::type_name::<P>(), value)
    }
}

#[derive(Default)]
struct VecPacker {
    vec: Vec<u8>,
}

impl Packer for VecPacker {
    type Error = Error;

    fn pack_bytes(&mut self, bytes: &[u8]) -> Result<(), Self::Error> {
        self.vec.extend_from_slice(bytes);
        Ok(())
    }
}

struct SliceUnpacker<'un> {
    slice: &'un [u8],
}

impl<'un> SliceUnpacker<'un> {
    fn new(slice: &'un [u8]) -> Self {
        Self { slice }
    }
}

impl<'un> Unpacker for SliceUnpacker<'un> {
    type Error = Error;

    fn unpack_exact_bytes<const N: usize>(&mut self) -> Result<&[u8; N], Self::Error> {
        Ok(unsafe { &*(self.unpack_bytes(N)? as *const [u8] as *const [u8; N]) })
    }

    fn unpack_bytes(&mut self, n: usize) -> Result<&[u8], Self::Error> {
        let (head, tail) = self.slice.split_at(n);

        if head.len() == n {
            self.slice = tail;
            Ok(head)
        } else {
            Err(Error::UnexpectedEof)
        }
    }
}

fn pack_checked(value: impl Packable + Eq + Debug) -> Vec<u8> {
    let mut packer = VecPacker::default();
    value.pack(&mut packer).unwrap();

    let mut unpacker = SliceUnpacker::new(&packer.vec);
    let result = Packable::unpack(&mut unpacker).unwrap();

    assert_eq!(value, result);

    packer.vec
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

    let bytes = packer.vec.leak();

    let mut unpacker = SliceUnpacker::new(bytes);
    let result = bool::unpack(&mut unpacker).unwrap();

    assert!(result);
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
