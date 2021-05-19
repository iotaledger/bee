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

/// A `Unpacker` backed by a `&[u8]`.
pub struct SliceUnpacker<'u>(&'u [u8]);

impl<'u> SliceUnpacker<'u> {
    /// Create a new unpacker from a byte slice.
    pub fn new(slice: &'u [u8]) -> Self {
        Self(slice)
    }
}

/// Error type to be raised when `SliceUnpacker` does not have enough bytes to unpack something.
#[derive(Debug)]
pub struct UnexpectedEOF {
    /// The required number of bytes.
    pub required: usize,
    /// THe number of bytes the unpacker had.
    pub had: usize,
}

impl<'u> Unpacker for SliceUnpacker<'u> {
    type Error = UnexpectedEOF;

    fn unpack_bytes(&mut self, slice: &mut [u8]) -> Result<(), Self::Error> {
        let len = slice.len();

        if self.0.len() >= len {
            let (head, tail) = self.0.split_at(len);
            self.0 = tail;
            slice.copy_from_slice(head);
            Ok(())
        } else {
            Err(UnexpectedEOF {
                required: len,
                had: self.0.len(),
            })
        }
    }
}
