// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use super::{Packable, UnpackError};

use core::convert::Infallible;

/// A type that can unpack any value that implements `Packer`.
pub trait Unpacker: Sized {
    /// The error type that can be returned if some error occurs while unpacking.
    type Error;

    /// Unpack a sequence of bytes from the `Unpacker`.
    fn unpack_bytes(&mut self, slice: &mut [u8]) -> Result<(), Self::Error>;

    /// Unpack a packable value that has no semantic errors.
    ///
    /// Note that the unpacking process can still fail.
    fn unpack_infallible<P>(&mut self) -> Result<P, Self::Error>
    where
        P: Packable<Error = Infallible>,
    {
        P::unpack(self).map_err(|err| match err {
            UnpackError::Packable(err) => match err {},
            UnpackError::Unpacker(err) => err,
        })
    }
}
