// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! A module that provides a [`Packable`] trait to serialize and deserialize types.

extern crate alloc;

mod array;
mod bool;
mod bounded;
mod r#box;
mod integer;
mod option;
mod vec;
mod vec_prefix;

pub use bounded::{
    Bounded, BoundedU16, BoundedU32, BoundedU64, BoundedU8, InvalidBoundedU16, InvalidBoundedU32, InvalidBoundedU64,
    InvalidBoundedU8,
};
pub use option::UnpackOptionError;
pub use vec_prefix::{VecPrefix, VecPrefixLengthError};

pub use crate::{
    error::{UnexpectedEOF, UnknownTagError, UnpackError, UnpackErrorExt},
    packer::{Packer, SlicePacker, VecPacker},
    unpacker::{SliceUnpacker, Unpacker},
};

pub use bee_packable_derive::Packable;

use alloc::vec::Vec;
use core::{convert::AsRef, fmt::Debug};

/// A type that can be packed and unpacked.
///
/// Almost all basic sized types implement this trait. This trait can be derived using the
/// [`Packable`](bee_packable_derive::Packable) macro. If you need to implement this trait manually, use the provided
/// implementations as a guide.
pub trait Packable: Sized {
    /// The error type that can be returned if some semantic error occurs while unpacking.
    ///
    /// It is recommended to use [`Infallible`](core::convert::Infallible) if this kind of error cannot happen or
    /// [`UnknownTagError`] when implementing this trait for an enum.
    type UnpackError: Debug;

    /// Packs this value into the given [`Packer`].
    fn pack<P: Packer>(&self, packer: &mut P) -> Result<(), P::Error>;

    /// The size of the value in bytes after being packed.
    fn packed_len(&self) -> usize;

    /// Convenience method that packs this value into a [`Vec<u8>`].
    fn pack_to_vec(&self) -> Vec<u8> {
        let mut packer = VecPacker::with_capacity(self.packed_len());

        // Packing to a `VecPacker` cannot fail.
        self.pack(&mut packer).unwrap();

        packer.into_vec()
    }

    /// Unpacks this value from the given [`Unpacker`].
    fn unpack<U: Unpacker>(unpacker: &mut U) -> Result<Self, UnpackError<Self::UnpackError, U::Error>>;

    /// Unpacks this value from a type that implements [`AsRef<[u8]>`].
    fn unpack_from_slice<T: AsRef<[u8]>>(bytes: T) -> Result<Self, UnpackError<Self::UnpackError, UnexpectedEOF>> {
        let mut unpacker = SliceUnpacker::new(bytes.as_ref());
        Packable::unpack(&mut unpacker)
    }
}
