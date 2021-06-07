// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! A module that provides a `Packable` trait to serialize and deserialize types.

mod array;
mod bool;
mod r#box;
mod integer;
mod option;
mod vec;
mod vec_prefix;

pub use option::UnpackOptionError;
pub use vec_prefix::VecPrefix;

pub use crate::{
    error::{UnknownTagError, UnpackError},
    packer::{Packer, VecPacker},
    unpacker::{SliceUnpacker, UnexpectedEOF, Unpacker},
};

pub use bee_packable_derive::Packable;

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
