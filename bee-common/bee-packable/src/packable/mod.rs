// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! A module that provides a [`Packable`] trait to serialize and deserialize types.

extern crate alloc;

pub mod bounded;
pub mod option;
pub mod prefix;

mod array;
mod bool;
mod r#box;
mod integer;
mod vec;

use crate::{
    error::{UnexpectedEOF, UnpackError},
    packer::{Packer, VecPacker},
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
    /// [`UnknownTagError`](crate::error::UnknownTagError) when implementing this trait for an enum.
    type UnpackError: Debug;

    /// The size of the value in bytes after being packed.
    fn packed_len(&self) -> usize;

    /// Packs this value into the given [`Packer`].
    fn pack<P: Packer>(&self, packer: &mut P) -> Result<(), P::Error>;

    /// Unpacks this value from the given [`Unpacker`].
    fn unpack<U: Unpacker, const VERIFY: bool>(
        unpacker: &mut U,
    ) -> Result<Self, UnpackError<Self::UnpackError, U::Error>>;
}

/// Extension trait for types that implement [`Packable`].
pub trait PackableExt: Packable {
    /// Convenience method that packs this value into a [`Vec<u8>`].
    fn pack_to_vec(&self) -> Vec<u8>;

    /// Unpacks this value from a sequence of bytes doing syntactical checks.
    fn unpack_verified<T: AsRef<[u8]>>(
        bytes: T,
    ) -> Result<Self, UnpackError<<Self as Packable>::UnpackError, UnexpectedEOF>>;

    /// Unpacks this value from a sequence of bytes without doing syntactical checks.
    fn unpack_unverified<T: AsRef<[u8]>>(
        bytes: T,
    ) -> Result<Self, UnpackError<<Self as Packable>::UnpackError, UnexpectedEOF>>;
}

impl<P: Packable> PackableExt for P {
    fn pack_to_vec(&self) -> Vec<u8> {
        let mut packer = VecPacker::with_capacity(self.packed_len());

        // Packing to a `VecPacker` cannot fail.
        self.pack(&mut packer).unwrap();

        packer.into_vec()
    }

    fn unpack_verified<T: AsRef<[u8]>>(
        bytes: T,
    ) -> Result<Self, UnpackError<<Self as Packable>::UnpackError, UnexpectedEOF>> {
        Self::unpack::<_, true>(&mut SliceUnpacker::new(bytes.as_ref()))
    }

    fn unpack_unverified<T: AsRef<[u8]>>(
        bytes: T,
    ) -> Result<Self, UnpackError<<Self as Packable>::UnpackError, UnexpectedEOF>> {
        Self::unpack::<_, false>(&mut SliceUnpacker::new(bytes.as_ref()))
    }
}
