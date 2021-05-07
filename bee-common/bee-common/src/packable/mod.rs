// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! A module that provides a `Packable` trait to serialize and deserialize types.

#[cfg(feature = "alloc")]
mod alloc;
mod error;
#[cfg(feature = "io")]
mod io;
mod packer;
mod unpacker;

pub use error::{UnknownVariant, UnpackError};
pub use packer::Packer;
pub use unpacker::Unpacker;

/// A type that can be packed and unpacked.
///
/// Almost all basic sized types implement this trait. This trait can be derived using the
/// `bee_common_derive::Packable` macro. If you need to implement this trait manually, use the provided
/// implementations as a guide.
pub trait Packable: Sized {
    /// Pack this value into the given `Packer`.
    fn pack<P: Packer>(&self, packer: &mut P) -> Result<(), P::Error>;
    /// Unpack this value from the given `Unpacker`.
    fn unpack<U: Unpacker>(unpacker: &mut U) -> Result<Self, U::Error>;
    /// The size of the value in bytes after being packed.
    fn packed_len(&self) -> usize;
}

macro_rules! impl_packable_for_int {
    ($ty:ty) => {
        impl Packable for $ty {
            fn pack<P: Packer>(&self, packer: &mut P) -> Result<(), P::Error> {
                packer.pack_bytes(&self.to_le_bytes())
            }

            fn unpack<U: Unpacker>(unpacker: &mut U) -> Result<Self, U::Error> {
                let bytes: [u8; core::mem::size_of::<Self>()] = unpacker.unpack_exact_bytes()?;
                Ok(Self::from_le_bytes(bytes))
            }

            fn packed_len(&self) -> usize {
                core::mem::size_of::<Self>()
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
    fn pack<P: Packer>(&self, packer: &mut P) -> Result<(), P::Error> {
        (*self as u64).pack(packer)
    }

    fn unpack<U: Unpacker>(unpacker: &mut U) -> Result<Self, U::Error> {
        Ok(u64::unpack(unpacker)? as usize)
    }

    fn packed_len(&self) -> usize {
        core::mem::size_of::<u64>()
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
    fn pack<P: Packer>(&self, packer: &mut P) -> Result<(), P::Error> {
        (*self as i64).pack(packer)
    }

    fn unpack<U: Unpacker>(unpacker: &mut U) -> Result<Self, U::Error> {
        Ok(u64::unpack(unpacker)? as isize)
    }

    fn packed_len(&self) -> usize {
        core::mem::size_of::<i64>()
    }
}

impl Packable for bool {
    /// Booleans are packed as `u8` integers following Rust's data layout.
    fn pack<P: Packer>(&self, packer: &mut P) -> Result<(), P::Error> {
        (*self as u8).pack(packer)
    }

    /// Booleans are unpacked if the byte used to represent them is non-zero.
    fn unpack<U: Unpacker>(unpacker: &mut U) -> Result<Self, U::Error> {
        Ok(u8::unpack(unpacker)? != 0)
    }

    fn packed_len(&self) -> usize {
        core::mem::size_of::<u8>()
    }
}

/// Options are packed and unpacked using `0u8` as the prefix for `None` and `1u8` as the prefix
/// for `Some`.
impl<T: Packable> Packable for Option<T> {
    fn pack<P: Packer>(&self, packer: &mut P) -> Result<(), P::Error> {
        match self {
            None => 0u8.pack(packer),
            Some(item) => {
                1u8.pack(packer)?;
                item.pack(packer)
            }
        }
    }

    fn unpack<U: Unpacker>(unpacker: &mut U) -> Result<Self, U::Error> {
        match u8::unpack(unpacker)? {
            0 => Ok(None),
            1 => Ok(Some(T::unpack(unpacker)?)),
            n => Err(U::Error::custom(UnknownVariant::new::<Self>(n.into()))),
        }
    }

    fn packed_len(&self) -> usize {
        0u8.packed_len()
            + match self {
                Some(item) => item.packed_len(),
                None => 0,
            }
    }
}

impl<T: Packable, const N: usize> Packable for [T; N] {
    fn pack<P: Packer>(&self, packer: &mut P) -> Result<(), P::Error> {
        for item in self.iter() {
            item.pack(packer)?;
        }

        Ok(())
    }

    fn unpack<U: Unpacker>(unpacker: &mut U) -> Result<Self, U::Error> {
        use core::mem::MaybeUninit;

        // Safety: an uninitialized array of `MaybeUninit`s is safe to be considered initialized.
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
        Ok(unsafe { (&array as *const [MaybeUninit<T>; N] as *const Self).read() })
    }

    fn packed_len(&self) -> usize {
        self.iter().map(T::packed_len).sum::<usize>()
    }
}
