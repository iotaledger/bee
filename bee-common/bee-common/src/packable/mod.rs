// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! A module that provides a `Packable` trait to serialize and deserialize types.

extern crate alloc;

mod error;
#[cfg(feature = "io")]
mod io;
mod packer;
mod unpacker;

use alloc::{boxed::Box, vec::Vec};
use core::convert::Infallible;

pub use error::{UnknownTagError, UnpackError};
pub use packer::{Packer, VecPacker};
pub use unpacker::{SliceUnpacker, UnexpectedEOF, Unpacker};

pub use bee_common_derive::Packable;

/// A type that can be packed and unpacked.
///
/// Almost all basic sized types implement this trait. This trait can be derived using the
/// `bee_common_derive::Packable` macro. If you need to implement this trait manually, use the provided
/// implementations as a guide.
pub trait Packable: Sized {
    /// The error type that can be returned if some semantic error occurs while unpacking.
    ///
    /// It is recommended to use `core::convert::Infallible` if this kind of error cannot happen or
    /// `UnknownTagError` when implementing this trait for an enum.
    type Error;
    /// Pack this value into the given `Packer`.
    fn pack<P: Packer>(&self, packer: &mut P) -> Result<(), P::Error>;
    /// The size of the value in bytes after being packed.
    fn packed_len(&self) -> usize;
    /// Unpack this value from the given `Unpacker`.
    fn unpack<U: Unpacker>(unpacker: &mut U) -> Result<Self, UnpackError<Self::Error, U::Error>>;
}

macro_rules! impl_packable_for_int {
    ($ty:ty) => {
        impl Packable for $ty {
            type Error = Infallible;

            fn pack<P: Packer>(&self, packer: &mut P) -> Result<(), P::Error> {
                packer.pack_bytes(&self.to_le_bytes())
            }

            fn packed_len(&self) -> usize {
                core::mem::size_of::<Self>()
            }

            fn unpack<U: Unpacker>(unpacker: &mut U) -> Result<Self, UnpackError<Self::Error, U::Error>> {
                let mut bytes = [0u8; core::mem::size_of::<Self>()];
                unpacker.unpack_bytes(&mut bytes)?;
                Ok(Self::from_le_bytes(bytes))
            }
        }
    };
}

impl_packable_for_int!(u8);
impl_packable_for_int!(u16);
impl_packable_for_int!(u32);
impl_packable_for_int!(u64);
#[cfg(has_u128)]
impl_packable_for_int!(u128);

/// `usize` integers are packed and unpacked as `u64` integers according to the spec.
impl Packable for usize {
    type Error = Infallible;

    fn pack<P: Packer>(&self, packer: &mut P) -> Result<(), P::Error> {
        (*self as u64).pack(packer)
    }

    fn packed_len(&self) -> usize {
        core::mem::size_of::<u64>()
    }

    fn unpack<U: Unpacker>(unpacker: &mut U) -> Result<Self, UnpackError<Self::Error, U::Error>> {
        Ok(unpacker.unpack_infallible::<u64>().map_err(UnpackError::Unpacker)? as usize)
    }
}

impl_packable_for_int!(i8);
impl_packable_for_int!(i16);
impl_packable_for_int!(i32);
impl_packable_for_int!(i64);
#[cfg(has_i128)]
impl_packable_for_int!(i128);

/// `isize` integers are packed and unpacked as `i64` integers according to the spec.
impl Packable for isize {
    type Error = Infallible;

    fn pack<P: Packer>(&self, packer: &mut P) -> Result<(), P::Error> {
        (*self as i64).pack(packer)
    }

    fn packed_len(&self) -> usize {
        core::mem::size_of::<i64>()
    }

    fn unpack<U: Unpacker>(unpacker: &mut U) -> Result<Self, UnpackError<Self::Error, U::Error>> {
        Ok(unpacker.unpack_infallible::<i64>().map_err(UnpackError::Unpacker)? as isize)
    }
}

impl Packable for bool {
    type Error = Infallible;

    /// Booleans are packed as `u8` integers following Rust's data layout.
    fn pack<P: Packer>(&self, packer: &mut P) -> Result<(), P::Error> {
        (*self as u8).pack(packer)
    }

    fn packed_len(&self) -> usize {
        core::mem::size_of::<u8>()
    }

    /// Booleans are unpacked if the byte used to represent them is non-zero.
    fn unpack<U: Unpacker>(unpacker: &mut U) -> Result<Self, UnpackError<Self::Error, U::Error>> {
        Ok(unpacker.unpack_infallible::<u8>().map_err(UnpackError::Unpacker)? != 0)
    }
}

/// Error type raised when a semantic error occurs while unpacking an option.
#[derive(Debug)]
pub enum UnpackOptionError<T: Packable> {
    /// The tag found while unpacking is not valid.
    UnknownTag(u8),
    /// A semantic error for the underlying type was raised.
    Inner(T::Error),
}

/// Options are packed and unpacked using `0u8` as the prefix for `None` and `1u8` as the prefix
/// for `Some`.
impl<T: Packable> Packable for Option<T> {
    type Error = UnpackOptionError<T>;

    fn pack<P: Packer>(&self, packer: &mut P) -> Result<(), P::Error> {
        match self {
            None => 0u8.pack(packer),
            Some(item) => {
                1u8.pack(packer)?;
                item.pack(packer)
            }
        }
    }

    fn packed_len(&self) -> usize {
        0u8.packed_len()
            + match self {
                Some(item) => item.packed_len(),
                None => 0,
            }
    }

    fn unpack<U: Unpacker>(unpacker: &mut U) -> Result<Self, UnpackError<Self::Error, U::Error>> {
        match unpacker.unpack_infallible::<u8>()? {
            0 => Ok(None),
            1 => Ok(Some(
                T::unpack(unpacker).map_err(|err| err.map(UnpackOptionError::Inner))?,
            )),
            n => Err(UnpackError::Packable(Self::Error::UnknownTag(n))),
        }
    }
}

impl<T: Packable, const N: usize> Packable for [T; N] {
    type Error = T::Error;

    fn pack<P: Packer>(&self, packer: &mut P) -> Result<(), P::Error> {
        for item in self.iter() {
            item.pack(packer)?;
        }

        Ok(())
    }

    fn packed_len(&self) -> usize {
        self.iter().map(T::packed_len).sum::<usize>()
    }

    fn unpack<U: Unpacker>(unpacker: &mut U) -> Result<Self, UnpackError<Self::Error, U::Error>> {
        use core::mem::MaybeUninit;

        // Safety: an uninitialized array of `MaybeUninit`s is safe to be considered initialized.
        // FIXME: replace with [`uninit_array`](https://doc.rust-lang.org/std/mem/union.MaybeUninit.html#method.uninit_array)
        // when stabilized.
        let mut array = unsafe { MaybeUninit::<[MaybeUninit<T>; N]>::uninit().assume_init() };

        for item in array.iter_mut() {
            let unpacked = T::unpack(unpacker)?;
            // Safety: each `item` is only visited once so we are never overwriting nor dropping
            // values that are already initialized.
            unsafe {
                item.as_mut_ptr().write(unpacked);
            }
        }

        // Safety: We traversed the whole array and initialized every item.
        // FIXME: replace with [`array_assume_init`](https://doc.rust-lang.org/std/mem/union.MaybeUninit.html#method.array_assume_init)
        // when stabilized.
        Ok(unsafe { (&array as *const [MaybeUninit<T>; N] as *const Self).read() })
    }
}

impl<T: Packable> Packable for Vec<T> {
    type Error = T::Error;

    fn pack<P: Packer>(&self, packer: &mut P) -> Result<(), P::Error> {
        // The length of any dynamically-sized sequence must be prefixed.
        self.len().pack(packer)?;

        for item in self.iter() {
            item.pack(packer)?;
        }

        Ok(())
    }

    fn packed_len(&self) -> usize {
        0usize.packed_len() + self.iter().map(T::packed_len).sum::<usize>()
    }

    fn unpack<U: Unpacker>(unpacker: &mut U) -> Result<Self, UnpackError<Self::Error, U::Error>> {
        // The length of any dynamically-sized sequence must be prefixed.
        let len = unpacker.unpack_infallible::<usize>().map_err(UnpackError::Unpacker)?;

        let mut vec = Self::with_capacity(len);

        for _ in 0..len {
            let item = T::unpack(unpacker)?;
            vec.push(item);
        }

        Ok(vec)
    }
}

impl<T: Packable> Packable for Box<[T]> {
    type Error = T::Error;

    fn pack<P: Packer>(&self, packer: &mut P) -> Result<(), P::Error> {
        // The length of any dynamically-sized sequence must be prefixed.
        self.len().pack(packer)?;

        for item in self.iter() {
            item.pack(packer)?;
        }

        Ok(())
    }

    fn packed_len(&self) -> usize {
        0usize.packed_len() + self.iter().map(T::packed_len).sum::<usize>()
    }

    fn unpack<U: Unpacker>(unpacker: &mut U) -> Result<Self, UnpackError<Self::Error, U::Error>> {
        Ok(Vec::<T>::unpack(unpacker)?.into_boxed_slice())
    }
}
