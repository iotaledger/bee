// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

pub use crate::{
    error::{PackError, UnknownTagError, UnpackError},
    packer::{Packer, VecPacker},
    unpacker::{SliceUnpacker, UnexpectedEOF, Unpacker},
    Packable,
};

use core::convert::Infallible;

impl Packable for bool {
    type PackError = Infallible;
    type UnpackError = Infallible;

    /// Booleans are packed as `u8` integers following Rust's data layout.
    fn pack<P: Packer>(&self, packer: &mut P) -> Result<(), PackError<Self::PackError, P::Error>> {
        (*self as u8).pack(packer)
    }

    fn packed_len(&self) -> usize {
        core::mem::size_of::<u8>()
    }

    /// Booleans are unpacked if the byte used to represent them is non-zero.
    fn unpack<U: Unpacker>(unpacker: &mut U) -> Result<Self, UnpackError<Self::UnpackError, U::Error>> {
        Ok(u8::unpack(unpacker).map_err(UnpackError::infallible)? != 0)
    }
}
