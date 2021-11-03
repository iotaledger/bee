// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    error::{UnpackError, UnpackErrorExt},
    packer::Packer,
    unpacker::Unpacker,
    Packable,
};

use core::convert::Infallible;

impl Packable for bool {
    type UnpackError = Infallible;

    fn packed_len(&self) -> usize {
        core::mem::size_of::<u8>()
    }

    /// Booleans are packed as `u8` integers following Rust's data layout.
    fn pack<P: Packer>(&self, packer: &mut P) -> Result<(), P::Error> {
        (*self as u8).pack(packer)
    }

    /// Booleans are unpacked if the byte used to represent them is non-zero.
    fn unpack<U: Unpacker, const CHECK: bool>(
        unpacker: &mut U,
    ) -> Result<Self, UnpackError<Self::UnpackError, U::Error>> {
        Ok(u8::unpack::<_, CHECK>(unpacker).infallible()? != 0)
    }
}
