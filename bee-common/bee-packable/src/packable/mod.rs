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

use crate::{
    error::{UnexpectedEOF, UnpackError},
    packer::{LenPacker, Packer, VecPacker},
    unpacker::{SliceUnpacker, Unpacker},
};

pub use bee_packable_derive::Packable;

use alloc::vec::Vec;
use core::{convert::AsRef, fmt::Debug};

/// A type that can be packed and unpacked.
///
/// Almost all basic sized types implement this trait. This trait can be derived using the
/// [`Packable`](bee_packable_derive::Packable) macro. The following example shows how to implement
/// this trait manually.
///
/// # Example
///
/// We will implement [`Packable`] for a type that encapsulates optional integer values (like
/// `Option<i32>`).
///
/// Following the conventions from the [IOTA protocol messages RFC](https:///github.com/iotaledger/protocol-rfcs/pull/0017),
/// we will use an integer prefix as a tag to determine which variant of the enum is being packed.
///
/// ```rust
/// use bee_packable::{
///     error::{UnknownTagError, UnpackError, UnpackErrorExt},
///     packer::Packer,
///     unpacker::Unpacker,
///     Packable,
/// };
///
/// use core::convert::Infallible;
///
/// pub enum Maybe {
///     Nothing,
///     Just(i32),
/// }
///
/// impl Packable for Maybe {
///     type UnpackError = UnknownTagError<u8>;
///
///     fn pack<P: Packer>(&self, packer: &mut P) -> Result<(), P::Error> {
///         match self {
///             // Pack a `0` byte and nothing else.
///             Self::Nothing => 0u8.pack(packer),
///             // Pack a `1` byte followed by the internal value.
///             Self::Just(value) => {
///                 1u8.pack(packer)?;
///                 value.pack(packer)
///             }
///         }
///     }
///
///     fn unpack<U: Unpacker, const VERIFY: bool>(
///         unpacker: &mut U,
///     ) -> Result<Self, UnpackError<Self::UnpackError, U::Error>> {
///         match u8::unpack::<_, VERIFY>(unpacker).infallible()? {
///             0u8 => Ok(Self::Nothing),
///             1u8 => Ok(Self::Just(i32::unpack::<_, VERIFY>(unpacker).coerce()?)),
///             tag => Err(UnpackError::Packable(UnknownTagError(tag))),
///         }
///     }
/// }
/// ```
/// To understand the behavior of `infallible` and `coerce` check the [`UnpackError`] and
/// [`UnpackErrorExt`](crate::error::UnpackErrorExt) documentation.
pub trait Packable: Sized {
    /// The error type that can be returned if some semantic error occurs while unpacking.
    ///
    /// It is recommended to use [`Infallible`](core::convert::Infallible) if this kind of error is
    /// impossible or [`UnknownTagError`](crate::error::UnknownTagError) when implementing this
    /// trait for an enum.
    type UnpackError: Debug;

    /// Packs this value into the given [`Packer`].
    fn pack<P: Packer>(&self, packer: &mut P) -> Result<(), P::Error>;

    /// Unpacks this value from the given [`Unpacker`]. The `VERIFY` generic parameter can be used
    /// to skip additional syntactic checks.
    fn unpack<U: Unpacker, const VERIFY: bool>(
        unpacker: &mut U,
    ) -> Result<Self, UnpackError<Self::UnpackError, U::Error>>;
}

/// Extension trait for types that implement [`Packable`].
pub trait PackableExt: Packable {
    /// Returns the length in bytes of the value after being packed. The returned value always
    /// matches the number of bytes written using `pack`.
    fn packed_len(&self) -> usize;

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
    fn packed_len(&self) -> usize {
        let mut packer = LenPacker(0);

        match self.pack(&mut packer) {
            Ok(_) => packer.0,
            Err(e) => match e {},
        }
    }

    fn pack_to_vec(&self) -> Vec<u8> {
        let mut packer = VecPacker::with_capacity(self.packed_len());

        // Packing to a `VecPacker` cannot fail.
        self.pack(&mut packer).unwrap();

        packer.into_vec()
    }

    /// Unpacks this value from a type that implements [`AsRef<[u8]>`].
    fn unpack_verified<T: AsRef<[u8]>>(
        bytes: T,
    ) -> Result<Self, UnpackError<<Self as Packable>::UnpackError, UnexpectedEOF>> {
        Self::unpack::<_, true>(&mut SliceUnpacker::new(bytes.as_ref()))
    }

    /// Unpacks this value from a type that implements [`AsRef<[u8]>`] skipping some syntatical
    /// checks.
    fn unpack_unverified<T: AsRef<[u8]>>(
        bytes: T,
    ) -> Result<Self, UnpackError<<Self as Packable>::UnpackError, UnexpectedEOF>> {
        Self::unpack::<_, false>(&mut SliceUnpacker::new(bytes.as_ref()))
    }
}
